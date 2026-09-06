//! AST + semantic analysis engine.
//!
//! The engine parses one file at a time into an arena-allocated oxc AST, runs
//! semantic analysis to get scope/symbol/reference data, then dispatches the
//! rule checks over the AST nodes. All output is owned [`Finding`] values so
//! nothing borrows from the arena once [`analyze_source`] returns.
//!
//! Lifetime note: the [`oxc_allocator::Allocator`] owns the AST arena and must
//! outlive every reference into the AST. We keep it as a local in
//! [`analyze_source`] and never return borrowed nodes, so callers are free of
//! the arena's lifetime.

use std::collections::{BTreeMap, BTreeSet};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportDeclarationSpecifier, ObjectExpression,
    ObjectPropertyKind, Program, PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_semantic::{AstNode, Semantic, SemanticBuilder};
use oxc_span::{GetSpan, SourceType, Span};

use crate::rules::ids;
use crate::source::LineIndex;
use crate::types::{Category, Confidence, Finding, Severity};

/// Known GSAP plugin identifiers used by the register/used-without-register
/// heuristics and the unscoped-selector plugin set.
const KNOWN_PLUGINS: &[&str] = &[
    "ScrollTrigger",
    "ScrollSmoother",
    "SplitText",
    "Flip",
    "Draggable",
    "Observer",
    "MotionPathPlugin",
    "DrawSVGPlugin",
    "MorphSVGPlugin",
    "InertiaPlugin",
    "CustomEase",
    "CustomWiggle",
    "CustomBounce",
    "ScrollToPlugin",
    "TextPlugin",
    "ScrambleTextPlugin",
    "Physics2DPlugin",
    "PhysicsPropsPlugin",
    "PixiPlugin",
];

const PLUGIN_VARS: &[(&str, &str)] = &[
    ("motionPath", "MotionPathPlugin"),
    ("drawSVG", "DrawSVGPlugin"),
    ("morphSVG", "MorphSVGPlugin"),
    ("text", "TextPlugin"),
    ("scrollTo", "ScrollToPlugin"),
    ("inertia", "InertiaPlugin"),
    ("scrambleText", "ScrambleTextPlugin"),
    ("physics2D", "Physics2DPlugin"),
    ("physicsProps", "PhysicsPropsPlugin"),
    ("pixi", "PixiPlugin"),
];

const DEV_ONLY_PLUGINS: &[&str] = &["GSDevTools", "MotionPathHelper"];

/// GSAP tween factory methods that take a vars object.
const TWEEN_METHODS: &[&str] = &["to", "from", "fromTo", "set"];

/// Timeline methods that return the timeline and can appear before a tween in a
/// fluent chain.
const TIMELINE_CHAIN_METHODS: &[&str] = &["add", "addLabel"];

/// Layout properties that force reflow when animated; transforms are preferred.
const LAYOUT_PROPS: &[&str] = &[
    "top",
    "left",
    "right",
    "bottom",
    "width",
    "height",
    "marginTop",
    "marginBottom",
    "marginLeft",
    "marginRight",
    "padding",
    "paddingTop",
    "paddingBottom",
    "paddingLeft",
    "paddingRight",
];

/// File-scoped facts gathered in a single pre-pass and shared by heuristic
/// rules that need a whole-file view (imports, registrations, directives).
#[allow(clippy::struct_excessive_bools)] // file-fact accumulator; named flags keep rule reads explicit
#[derive(Default)]
struct FileFacts {
    /// Identifiers passed to any `gsap.registerPlugin(...)` call in the file,
    /// from direct identifier arguments.
    registered: BTreeSet<String>,
    /// A `gsap.registerPlugin(...)` call passed an argument that cannot be
    /// resolved to a plugin name statically (a spread, a call, etc.). When set,
    /// the used-without-register check is suppressed for the
    /// file: we cannot prove a plugin was *not* registered.
    registration_unknown: bool,
    /// `useGSAP` is imported from `@gsap/react`.
    usegsap_bindings: BTreeSet<String>,
    /// Identifiers initialized to `useGSAP` config objects with a `scope` key.
    scoped_usegsap_configs: BTreeSet<String>,
    /// The file has a top-of-file `"use client"` directive.
    has_use_client: bool,
    /// The file uses GSAP member access or a bare GSAP plugin identifier.
    uses_gsap_surface: bool,
    /// Local identifiers bound to the GSAP object.
    gsap_bindings: BTreeSet<String>,
    /// Local bindings imported from the skill's configured GSAP module pattern
    /// (`lib/gsap`), where registration is centralized before re-export.
    configured_gsap_imports: BTreeSet<String>,
    /// Identifiers initialized from `gsap.timeline(...)`, with their resolved
    /// declarations so shadowed parameters and locals sharing the spelling
    /// are not treated as GSAP timelines.
    timeline_handles: Vec<TimelineHandle>,
    /// Local import aliases for known plugins, keyed by local binding.
    plugin_aliases: BTreeMap<String, String>,
    /// Local import aliases for dev-only helpers, keyed by local binding.
    dev_only_aliases: BTreeMap<String, String>,
    /// The file imports React, making capitalized functions plausible components.
    has_react_import: bool,
    /// The file contains JSX, which marks component files under the automatic
    /// JSX runtime even without an explicit React import.
    has_jsx: bool,
    /// The file calls `gsap.matchMedia(...)`.
    match_media_calls: Vec<MatchMediaCall>,
    /// `.revert()` call receivers, e.g. `mm` in `mm.revert()`. Compared by
    /// resolved declaration, not spelling, so cleanup in one scope cannot
    /// cover a shadowed binding in another.
    reverted_bindings: Vec<RevertRef>,
}

/// A `gsap.timeline()` result binding with its resolved declaration, so a
/// shadowed `tl` in another scope is never confused with the timeline.
struct TimelineHandle {
    name: String,
    declaration: Option<oxc_semantic::NodeId>,
}

/// Whether an identifier reference resolves to a recorded timeline handle:
/// same resolved declaration wins; spelling matches only when either side
/// is unresolvable.
fn identifier_is_timeline_handle(
    semantic: &Semantic<'_>,
    identifier: &oxc_ast::ast::IdentifierReference<'_>,
    facts: &FileFacts,
) -> bool {
    let declaration = reference_declaration(semantic, identifier);
    facts
        .timeline_handles
        .iter()
        .any(|handle| match (declaration, handle.declaration) {
            (Some(from), Some(to)) => from == to,
            _ => handle.name == identifier.name.as_str(),
        })
}
/// One `gsap.matchMedia(...)` call site and the cleanup covering it, if any.
/// A file-wide flag is wrong here: an unrelated `useGSAP(...)` elsewhere
/// cannot clean up this call's match-media context.
struct MatchMediaCall {
    /// Source span of the `gsap.matchMedia(...)` call, so each uncovered
    /// call gets its own finding instead of one file-level finding.
    span: Span,
    /// The call sits inside a `useGSAP(...)` callback, whose context reverts
    /// match-media state automatically.
    inside_usegsap: bool,
    /// The `mm` in `const mm = gsap.matchMedia()`, when directly assigned.
    binding: Option<MatchMediaBinding>,
}

/// A `gsap.matchMedia()` result binding with its resolved declaration, so a
/// shadowing `mm` in another scope is never confused with this one.
struct MatchMediaBinding {
    name: String,
    declaration: Option<oxc_semantic::NodeId>,
}

/// One `.revert()` receiver with its resolved declaration, if any.
struct RevertRef {
    name: String,
    declaration: Option<oxc_semantic::NodeId>,
}

/// Whether a revert receiver covers a match-media binding: same resolved
/// declaration wins; spelling matches only when either side is unresolvable.
fn revert_covers_binding(revert: &RevertRef, binding: &MatchMediaBinding) -> bool {
    match (revert.declaration, binding.declaration) {
        (Some(from), Some(to)) => from == to,
        _ => revert.name == binding.name,
    }
}

/// Parse and analyze a single source string, returning owned findings.
///
/// `relative_path` is used verbatim in finding output and for path-based rules
/// (SSR placement). `source_type` selects the oxc grammar.
#[must_use]
pub fn analyze_source(relative_path: &str, source: &str, source_type: SourceType) -> Vec<Finding> {
    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source, source_type).parse();
    // A panicked parse yields an empty program; emit nothing rather than noise.
    if parser_return.panicked {
        return Vec::new();
    }
    let program = parser_return.program;
    // `with_build_nodes(true)` is required: the default builder skips the full
    // AstNodes store, which would leave `semantic.nodes()` empty and disable
    // every node-walk rule.
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&program)
        .semantic;

    let line_index = LineIndex::new(source);
    let facts = collect_file_facts(&program, &semantic);

    let mut findings = Vec::new();
    let mut emit = |id: &str,
                    severity: Severity,
                    confidence: Confidence,
                    span: Span,
                    message: String,
                    suggestion: &str| {
        let descriptor = crate::rules::descriptor(id);
        let category = descriptor.map_or(Category::Core, |rule| rule.category);
        let (line, column) = line_index.line_col(span.start);
        findings.push(Finding {
            id: id.to_string(),
            category,
            severity,
            confidence,
            file: relative_path.to_string(),
            line,
            column,
            message,
            suggestion: suggestion.to_string(),
        });
    };

    // Node-level rules: walk every AST node once.
    for node in semantic.nodes() {
        check_node(node, &semantic, relative_path, &facts, &mut emit);
    }

    // File-level rules that do not hang off a single representative node.
    check_file_level(&program, relative_path, &facts, &line_index, &mut findings);

    findings.sort_by(|left, right| {
        (left.line, left.column, left.id.as_str()).cmp(&(
            right.line,
            right.column,
            right.id.as_str(),
        ))
    });
    findings
}

/// Pre-pass: gather whole-file facts used by several rules.
fn collect_file_facts<'a>(program: &Program<'a>, semantic: &Semantic<'a>) -> FileFacts {
    use oxc_ast::AstKind;

    let mut facts = FileFacts::default();

    // Directives are parsed at the top of the program body.
    for directive in &program.directives {
        if directive.expression.value.as_str() == "use client" {
            facts.has_use_client = true;
        }
    }

    // Walk every semantic node so usage and registration are detected anywhere
    // in the file, including inside function and component bodies.
    for node in semantic.nodes() {
        match node.kind() {
            AstKind::ImportDeclaration(import) => {
                record_gsap_import_bindings(import, &mut facts);
                record_usegsap_import_bindings(import, &mut facts);
                record_plugin_import_aliases(import, &mut facts);
                record_configured_gsap_imports(import, &mut facts);
                if !import.import_kind.is_type() && import.source.value.as_str() == "react" {
                    facts.has_react_import = true;
                }
            }
            AstKind::ImportExpression(import) => {
                record_configured_gsap_dynamic_import(import, semantic, node.id(), &mut facts);
            }
            AstKind::JSXElement(_) | AstKind::JSXFragment(_) => {
                facts.has_jsx = true;
            }
            AstKind::IdentifierReference(identifier)
                if plugin_name_for_identifier(identifier.name.as_str(), &facts).is_some()
                    && !reference_is_ts_type_position(semantic, node.id()) =>
            {
                facts.uses_gsap_surface = true;
            }
            AstKind::StaticMemberExpression(member) if member_object_is_gsap(member, &facts) => {
                facts.uses_gsap_surface = true;
            }
            AstKind::CallExpression(call) => {
                record_register(call, &mut facts);
                if is_usegsap_call(call, &facts) {
                    facts.uses_gsap_surface = true;
                }
                if is_gsap_member_call(call, &facts, "matchMedia") {
                    facts.match_media_calls.push(MatchMediaCall {
                        span: call.span,
                        inside_usegsap: call_inside_usegsap(semantic, node.id(), &facts),
                        binding: match_media_binding(semantic, node.id()),
                    });
                }
                if let Expression::StaticMemberExpression(member) =
                    call.callee.without_parentheses()
                    && member.property.name.as_str() == "revert"
                    && let Expression::Identifier(object) = member.object.without_parentheses()
                {
                    facts.reverted_bindings.push(RevertRef {
                        name: object.name.as_str().to_string(),
                        declaration: reference_declaration(semantic, object),
                    });
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                if let Some(identifier) = declarator.id.get_binding_identifier()
                    && let Some(init) = &declarator.init
                    && expression_is_gsap_timeline_call(init, &facts)
                {
                    facts.timeline_handles.push(TimelineHandle {
                        name: identifier.name.as_str().to_string(),
                        declaration: identifier
                            .symbol_id
                            .get()
                            .map(|symbol| semantic.scoping().symbol_declaration(symbol)),
                    });
                }
                if let Some(identifier) = declarator.id.get_binding_identifier()
                    && let Some(init) = &declarator.init
                    && let Expression::ObjectExpression(object) = init.without_parentheses()
                    && object_has_key(object, "scope")
                {
                    facts
                        .scoped_usegsap_configs
                        .insert(identifier.name.as_str().to_string());
                }
            }
            _ => {}
        }
    }

    facts
}

/// Whether an identifier reference resolves to an import of `imported`,
/// regardless of the local alias it was bound to.
fn identifier_imports_name(
    identifier: &oxc_ast::ast::IdentifierReference<'_>,
    semantic: &Semantic<'_>,
    imported: &str,
) -> bool {
    use oxc_ast::AstKind;

    let scoping = semantic.scoping();
    let Some(reference_id) = identifier.reference_id.get() else {
        return false;
    };
    let Some(symbol_id) = scoping.get_reference(reference_id).symbol_id() else {
        return false;
    };
    match semantic.nodes().kind(scoping.symbol_declaration(symbol_id)) {
        AstKind::ImportSpecifier(specifier) => specifier.imported.name() == imported,
        _ => false,
    }
}

/// Whether an identifier reference resolves to the setter of a `useState`
/// destructure, i.e. `setValue` in `const [value, setValue] = useState(...)`.
///
/// Resolved through the symbol table rather than by name. Two components in one
/// file can each declare a `setProgress`, and an unrelated prop or helper may
/// share the name; matching on text alone would report the wrong one.
fn identifier_is_use_state_setter(
    identifier: &oxc_ast::ast::IdentifierReference<'_>,
    semantic: &Semantic<'_>,
) -> bool {
    use oxc_ast::AstKind;
    use oxc_ast::ast::{BindingPattern, Expression};

    let scoping = semantic.scoping();
    let Some(reference_id) = identifier.reference_id.get() else {
        return false;
    };
    let Some(symbol_id) = scoping.get_reference(reference_id).symbol_id() else {
        return false;
    };
    let declaration_node = scoping.symbol_declaration(symbol_id);
    let AstKind::VariableDeclarator(declarator) = semantic.nodes().kind(declaration_node) else {
        return false;
    };
    let Some(init) = &declarator.init else {
        return false;
    };
    let Expression::CallExpression(call) = init.without_parentheses() else {
        return false;
    };
    // Resolve the hook through its import so an alias still counts:
    // `import { useState as useReactState } from "react"` binds the local name
    // to an `ImportSpecifier` whose imported name is the real hook.
    let is_use_state = match &call.callee {
        Expression::Identifier(callee) => {
            callee.name.as_str() == "useState"
                || identifier_imports_name(callee, semantic, "useState")
        }
        Expression::StaticMemberExpression(member) => member.property.name.as_str() == "useState",
        _ => false,
    };
    if !is_use_state {
        return false;
    }
    // The setter is specifically the second element. `const [value] =
    // useState()` declares no setter, and the value itself is not one.
    let BindingPattern::ArrayPattern(pattern) = &declarator.id else {
        return false;
    };
    pattern
        .elements
        .get(1)
        .and_then(Option::as_ref)
        .and_then(oxc_ast::ast::BindingPattern::get_binding_identifier)
        .is_some_and(|binding| binding.symbol_id.get() == Some(symbol_id))
}

/// JSX handler props that fire continuously while the user scrolls or moves.
///
/// `onClick` and friends are deliberately absent: a state update per click is
/// ordinary React. The defect is a state update per *frame*.
const CONTINUOUS_JSX_HANDLERS: &[&str] = &[
    "onScroll",
    "onWheel",
    "onPointerMove",
    "onMouseMove",
    "onTouchMove",
    "onDrag",
];

/// DOM events that fire continuously, as passed to `addEventListener`.
const CONTINUOUS_DOM_EVENTS: &[&str] = &[
    "scroll",
    "wheel",
    "pointermove",
    "mousemove",
    "touchmove",
    "drag",
];

/// Whether `node_id` sits inside a callback driven by a continuous motion
/// source, and if so what to name it in the message.
///
/// Walks outward to the nearest enclosing function, then classifies that
/// function by how it is passed. Bounded so a deeply nested expression cannot
/// walk the whole file.
fn continuous_motion_driver(
    semantic: &Semantic<'_>,
    node_id: oxc_semantic::NodeId,
    facts: &FileFacts,
) -> Option<String> {
    use oxc_ast::AstKind;
    use oxc_ast::ast::{Expression, PropertyKey};

    let nodes = semantic.nodes();
    let mut current = node_id;

    // Find the nearest enclosing function expression.
    let function_node = loop {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return None;
        }
        match nodes.kind(parent_id) {
            AstKind::ArrowFunctionExpression(_) | AstKind::Function(_) => break parent_id,
            _ => current = parent_id,
        }
    };

    // Classify by how that function is used.
    let mut current = function_node;
    for _ in 0..6 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return None;
        }
        match nodes.kind(parent_id) {
            // <div onScroll={() => setX(...)} />
            AstKind::JSXExpressionContainer(_) => {
                let attribute_id = nodes.parent_id(parent_id);
                if let AstKind::JSXAttribute(attribute) = nodes.kind(attribute_id)
                    && let Some(name) = attribute.name.as_identifier()
                    && CONTINUOUS_JSX_HANDLERS.contains(&name.name.as_str())
                {
                    return Some(format!("the `{}` handler", name.name));
                }
                return None;
            }
            // { onUpdate: () => setX(...) } inside a GSAP call.
            //
            // `onUpdate` alone is not enough: `editor.configure({ onUpdate })`
            // is an ordinary callback with no frame loop behind it, so the
            // enclosing call must be a GSAP one. `onRefresh` is deliberately
            // excluded — ScrollTrigger fires it on refresh, not per frame.
            AstKind::ObjectProperty(property) => {
                if let PropertyKey::StaticIdentifier(key) = &property.key
                    && key.name.as_str() == "onUpdate"
                    && enclosing_call_is_gsap(semantic, parent_id, facts)
                {
                    return Some("a GSAP `onUpdate` callback".to_string());
                }
                return None;
            }
            AstKind::CallExpression(call) => {
                match &call.callee {
                    // A bare `requestAnimationFrame(() => ...)` runs once.
                    // Only a callback that schedules another frame is a loop.
                    Expression::Identifier(identifier)
                        if identifier.name.as_str() == "requestAnimationFrame"
                            && function_reschedules_animation_frame(semantic, function_node) =>
                    {
                        return Some("a requestAnimationFrame loop".to_string());
                    }
                    Expression::StaticMemberExpression(member) => {
                        let property = member.property.name.as_str();
                        // el.addEventListener("scroll", fn)
                        if property == "addEventListener"
                            && let Some(Expression::StringLiteral(event)) =
                                call.arguments.first().and_then(|a| a.as_expression())
                            && CONTINUOUS_DOM_EVENTS.contains(&event.value.as_str())
                        {
                            return Some(format!("a `{}` listener", event.value));
                        }
                        // gsap.ticker.add(fn). The receiver must resolve to a
                        // known GSAP binding: `analytics.ticker.add(...)` is an
                        // unrelated API that happens to share a property name.
                        if property == "add"
                            && member_expression_mentions(member, "ticker")
                            && member_chain_roots_at_gsap(member, facts)
                        {
                            return Some("a gsap.ticker callback".to_string());
                        }
                        return None;
                    }
                    _ => return None,
                }
            }
            _ => current = parent_id,
        }
    }
    None
}

/// Rule: a `useState` setter called from a continuous motion source.
///
/// Driving scroll, pointer or ticker values through React state re-renders the
/// component on every frame. The fix is to write to a ref or a
/// `gsap.quickTo`/`quickSetter` and leave React out of the frame loop.
fn check_state_in_continuous_motion<'a, F>(
    call: &oxc_ast::ast::CallExpression<'a>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'a>,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    use oxc_ast::ast::Expression;

    let Expression::Identifier(callee) = call.callee.without_parentheses() else {
        return;
    };
    let setter = callee.name.as_str();
    if !identifier_is_use_state_setter(callee, semantic) {
        return;
    }
    let Some(driver) = continuous_motion_driver(semantic, node_id, facts) else {
        return;
    };

    // React bails out of a render when the next state equals the current one,
    // so a threshold like `setStuck(y > 100)` re-renders only when the boolean
    // flips. That is the prescribed pattern, not a per-frame render, and
    // failing the default gate on it would punish correct code. Report it as
    // advisory instead of claiming a cost it does not have.
    let stores_boolean = call
        .arguments
        .first()
        .and_then(|argument| argument.as_expression())
        .is_some_and(expression_is_boolean_valued);

    if stores_boolean {
        emit(
            ids::REACT_STATE_IN_CONTINUOUS_MOTION,
            Severity::Low,
            Confidence::Low,
            call.span,
            format!(
                "`{setter}` is called from {driver}, but stores a boolean, so React re-renders only when it flips."
            ),
            "Usually fine. Confirm the value is genuinely two-state; if it later stores a varying number, move it to a ref.",
        );
        return;
    }

    emit(
        ids::REACT_STATE_IN_CONTINUOUS_MOTION,
        Severity::High,
        Confidence::Medium,
        call.span,
        format!("`{setter}` is called from {driver}, re-rendering on every frame."),
        "Drive continuous motion outside React: write to a ref, or use gsap.quickTo/quickSetter and keep state for discrete changes only.",
    );
}

/// Whether an expression provably evaluates to a boolean.
///
/// Comparisons, `!x`, boolean literals and `a && b` / `a || b` over booleans.
/// Anything else is treated as potentially continuous.
fn expression_is_boolean_valued(expression: &oxc_ast::ast::Expression<'_>) -> bool {
    use oxc_ast::ast::{BinaryOperator, Expression, LogicalOperator, UnaryOperator};

    match expression.without_parentheses() {
        Expression::BooleanLiteral(_) => true,
        Expression::UnaryExpression(unary) => unary.operator == UnaryOperator::LogicalNot,
        Expression::BinaryExpression(binary) => matches!(
            binary.operator,
            BinaryOperator::GreaterThan
                | BinaryOperator::GreaterEqualThan
                | BinaryOperator::LessThan
                | BinaryOperator::LessEqualThan
                | BinaryOperator::Equality
                | BinaryOperator::StrictEquality
                | BinaryOperator::Inequality
                | BinaryOperator::StrictInequality
                | BinaryOperator::In
                | BinaryOperator::Instanceof
        ),
        Expression::LogicalExpression(logical) => {
            matches!(logical.operator, LogicalOperator::And | LogicalOperator::Or)
                && expression_is_boolean_valued(&logical.left)
                && expression_is_boolean_valued(&logical.right)
        }
        _ => false,
    }
}

/// Whether the call enclosing an object literal is a GSAP one.
///
/// Walks out from the property to the nearest `CallExpression` and checks its
/// callee against the file's known GSAP bindings, so a config object passed to
/// an unrelated library is not mistaken for a tween.
fn enclosing_call_is_gsap(
    semantic: &Semantic<'_>,
    property_id: oxc_semantic::NodeId,
    facts: &FileFacts,
) -> bool {
    use oxc_ast::AstKind;
    use oxc_ast::ast::Expression;

    let nodes = semantic.nodes();
    let mut current = property_id;
    for _ in 0..8 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        if let AstKind::CallExpression(call) = nodes.kind(parent_id) {
            return match &call.callee {
                // `ScrollTrigger.create({...})`, including an alias:
                // `import { ScrollTrigger as ST }` then `ST.create(...)`.
                // plugin_aliases already records the local binding, so resolve
                // through it rather than matching the literal property chain.
                Expression::StaticMemberExpression(member) => {
                    member_object_is_gsap(member, facts)
                        || member_object_resolves_to_plugin(member, facts)
                }
                Expression::Identifier(identifier) => {
                    let name = identifier.name.as_str();
                    facts.gsap_bindings.contains(name)
                        || facts.configured_gsap_imports.contains(name)
                        || plugin_name_for_identifier(name, facts).is_some()
                }
                _ => false,
            };
        }
        current = parent_id;
    }
    false
}

/// Whether a function schedules another animation frame from inside itself.
///
/// A `requestAnimationFrame` callback runs exactly once unless it reschedules,
/// so `requestAnimationFrame(() => setReady(true))` is a single state update,
/// not a per-frame loop, and must not be reported as one.
fn function_reschedules_animation_frame(
    semantic: &Semantic<'_>,
    function_id: oxc_semantic::NodeId,
) -> bool {
    use oxc_ast::AstKind;
    use oxc_ast::ast::Expression;

    let nodes = semantic.nodes();
    let Some(range) = node_span(semantic, function_id) else {
        return false;
    };
    // The callback's own name, when it has one. `requestAnimationFrame(function
    // tick() { ... requestAnimationFrame(tick) })` recurses; so does a named
    // arrow assigned to a binding that the inner call references.
    let self_name = match nodes.kind(function_id) {
        AstKind::Function(function) => function.id.as_ref().map(|id| id.name.as_str().to_string()),
        AstKind::ArrowFunctionExpression(_) => match nodes.kind(nodes.parent_id(function_id)) {
            AstKind::VariableDeclarator(declarator) => declarator
                .id
                .get_binding_identifier()
                .map(|binding| binding.name.as_str().to_string()),
            _ => None,
        },
        _ => None,
    };

    nodes.iter().any(|node| {
        let AstKind::CallExpression(call) = node.kind() else {
            return false;
        };
        let Expression::Identifier(identifier) = &call.callee else {
            return false;
        };
        if identifier.name.as_str() != "requestAnimationFrame" {
            return false;
        }
        if call.span.start <= range.0 || call.span.end > range.1 {
            return false;
        }
        // Presence of another frame request is not recurrence. A finite
        // sequence such as `requestAnimationFrame(() => { setX(1);
        // requestAnimationFrame(renderOnce) })` schedules exactly two frames.
        // Only a call that reschedules THIS callback is a loop.
        match call.arguments.first().and_then(|a| a.as_expression()) {
            Some(Expression::Identifier(argument)) => {
                self_name.as_deref() == Some(argument.name.as_str())
            }
            // Anything else cannot be proven to reschedule THIS named callback.
            // Inline arrow/function callbacks that re-request a frame (the
            // recursive-arrow idiom) land here too — an accepted false negative.
            _ => false,
        }
    })
}

/// The source span of a node, when it is one this rule cares about.
fn node_span(semantic: &Semantic<'_>, node_id: oxc_semantic::NodeId) -> Option<(u32, u32)> {
    use oxc_ast::AstKind;
    match semantic.nodes().kind(node_id) {
        AstKind::ArrowFunctionExpression(function) => {
            Some((function.span.start, function.span.end))
        }
        AstKind::Function(function) => Some((function.span.start, function.span.end)),
        _ => None,
    }
}

/// Whether a member expression's object is a known GSAP plugin, alias included.
fn member_object_resolves_to_plugin(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    facts: &FileFacts,
) -> bool {
    use oxc_ast::ast::Expression;
    match member.object.without_parentheses() {
        Expression::Identifier(identifier) => {
            plugin_name_for_identifier(identifier.name.as_str(), facts).is_some()
        }
        _ => false,
    }
}

/// Whether a member chain bottoms out in an identifier bound to GSAP.
///
/// `gsap.ticker.add(fn)` qualifies; `analytics.ticker.add(fn)` does not, even
/// though both contain a `ticker` property.
fn member_chain_roots_at_gsap(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    facts: &FileFacts,
) -> bool {
    use oxc_ast::ast::Expression;
    let mut current = member.object.without_parentheses();
    for _ in 0..6 {
        match current {
            Expression::StaticMemberExpression(inner) => {
                current = inner.object.without_parentheses();
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name.as_str();
                return facts.gsap_bindings.contains(name)
                    || facts.configured_gsap_imports.contains(name);
            }
            _ => return false,
        }
    }
    false
}

/// Whether a member expression chain mentions `name` anywhere in its object.
fn member_expression_mentions(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    name: &str,
) -> bool {
    use oxc_ast::ast::Expression;
    let mut current = member.object.without_parentheses();
    for _ in 0..4 {
        match current {
            Expression::StaticMemberExpression(inner) => {
                if inner.property.name.as_str() == name {
                    return true;
                }
                current = inner.object.without_parentheses();
            }
            Expression::Identifier(identifier) => return identifier.name.as_str() == name,
            _ => return false,
        }
    }
    false
}

fn record_configured_gsap_imports(
    import: &oxc_ast::ast::ImportDeclaration<'_>,
    facts: &mut FileFacts,
) {
    if import.import_kind.is_type() {
        return;
    }
    if !import_source_is_configured_gsap_module(import.source.value.as_str()) {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                if named.import_kind.is_type() {
                    continue;
                }
                let imported = named.imported.name();
                let imported_name = imported.as_str();
                if imported_name == "gsap"
                    || imported_name == "useGSAP"
                    || KNOWN_PLUGINS.contains(&imported_name)
                {
                    facts
                        .configured_gsap_imports
                        .insert(named.local.name.as_str().to_string());
                    facts
                        .configured_gsap_imports
                        .insert(imported_name.to_string());
                    if imported_name == "gsap" {
                        facts
                            .gsap_bindings
                            .insert(named.local.name.as_str().to_string());
                    }
                    if imported_name == "useGSAP" {
                        facts
                            .usegsap_bindings
                            .insert(named.local.name.as_str().to_string());
                    }
                    if KNOWN_PLUGINS.contains(&imported_name) {
                        facts.plugin_aliases.insert(
                            named.local.name.as_str().to_string(),
                            imported_name.to_string(),
                        );
                    }
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                facts
                    .configured_gsap_imports
                    .insert(default.local.name.as_str().to_string());
                facts.configured_gsap_imports.insert("gsap".to_string());
                facts
                    .gsap_bindings
                    .insert(default.local.name.as_str().to_string());
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
        }
    }
}

fn record_gsap_import_bindings(
    import: &oxc_ast::ast::ImportDeclaration<'_>,
    facts: &mut FileFacts,
) {
    if import.import_kind.is_type() || import.source.value.as_str() != "gsap" {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named)
                if named.import_kind.is_value()
                    && matches!(named.imported.name().as_str(), "gsap" | "default") =>
            {
                facts
                    .gsap_bindings
                    .insert(named.local.name.as_str().to_string());
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                facts
                    .gsap_bindings
                    .insert(default.local.name.as_str().to_string());
            }
            _ => {}
        }
    }
}

fn record_usegsap_import_bindings(
    import: &oxc_ast::ast::ImportDeclaration<'_>,
    facts: &mut FileFacts,
) {
    if import.import_kind.is_type() || import.source.value.as_str() != "@gsap/react" {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            continue;
        };
        if named.import_kind.is_value() && named.imported.name().as_str() == "useGSAP" {
            facts
                .usegsap_bindings
                .insert(named.local.name.as_str().to_string());
            facts.uses_gsap_surface = true;
        }
    }
}

fn record_plugin_import_aliases(
    import: &oxc_ast::ast::ImportDeclaration<'_>,
    facts: &mut FileFacts,
) {
    if import.import_kind.is_type() {
        return;
    }
    let source = import.source.value.as_str();
    if !import_source_is_gsap_package(source) {
        return;
    }
    let source_plugin = plugin_name_from_import_source(source);
    let source_dev_only = dev_only_name_from_import_source(source);
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(named) => {
                if named.import_kind.is_type() {
                    continue;
                }
                let imported = named.imported.name();
                let imported_name = imported.as_str();
                let local_name = named.local.name.as_str().to_string();
                if let Some(plugin) = plugin_name_for_known_or_default(imported_name, source_plugin)
                {
                    facts
                        .plugin_aliases
                        .insert(local_name.clone(), plugin.to_string());
                }
                if let Some(dev_only) =
                    dev_only_name_for_known_or_default(imported_name, source_dev_only)
                {
                    facts
                        .dev_only_aliases
                        .insert(local_name, dev_only.to_string());
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                if let Some(plugin) = source_plugin {
                    facts
                        .plugin_aliases
                        .insert(default.local.name.as_str().to_string(), plugin.to_string());
                }
                if let Some(dev_only) = source_dev_only {
                    facts.dev_only_aliases.insert(
                        default.local.name.as_str().to_string(),
                        dev_only.to_string(),
                    );
                }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(namespace) => {
                if let Some(dev_only) = source_dev_only {
                    facts.dev_only_aliases.insert(
                        namespace.local.name.as_str().to_string(),
                        dev_only.to_string(),
                    );
                }
            }
        }
    }
}

fn plugin_name_for_known_or_default<'a>(
    imported_name: &'a str,
    source_plugin: Option<&'a str>,
) -> Option<&'a str> {
    if KNOWN_PLUGINS.contains(&imported_name) {
        Some(imported_name)
    } else if imported_name == "default" {
        source_plugin
    } else {
        None
    }
}

/// Whether an `IdentifierReference` node sits in a TypeScript type-only
/// position (e.g. `let x: GSDevTools`, `function f(p: GSDevTools)`), where the
/// name is erased at build time and carries no runtime/value reference.
///
/// In oxc, a type-position name is an `IdentifierReference` whose enclosing node
/// is a TS type construct: `TSTypeReference` (`x: Foo`), `TSQualifiedName`
/// (`x: Ns.Foo`), or a `TSImportType` qualifier (`x: import('m').Foo`). A value
/// use such as `GSDevTools.create()` or an import specifier has a non-type
/// parent, so it is not skipped.
fn reference_is_ts_type_position(semantic: &Semantic<'_>, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    matches!(
        semantic.nodes().parent_kind(node_id),
        AstKind::TSTypeReference(_)
            | AstKind::TSQualifiedName(_)
            | AstKind::TSImportType(_)
            | AstKind::TSImportTypeQualifiedName(_)
    )
}

/// Whether a static member expression's object is the `gsap` identifier.
fn member_object_is_gsap(
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    facts: &FileFacts,
) -> bool {
    matches!(
        member.object.without_parentheses(),
        Expression::Identifier(object) if is_gsap_identifier(object.name.as_str(), facts)
    )
}

fn import_source_is_gsap_package(source: &str) -> bool {
    source == "gsap" || source.starts_with("gsap/")
}

fn import_source_is_gsap_trial(source: &str) -> bool {
    source == "gsap-trial" || source.starts_with("gsap-trial/")
}

fn import_source_is_configured_gsap_module(source: &str) -> bool {
    // Any local or workspace module whose final path segment is `gsap` is
    // treated as the project's configured GSAP entrypoint (the module that
    // owns `registerPlugin`). This covers `./gsap`, `lib/gsap`, `~/lib/gsap`
    // and scoped workspace re-exports such as `@scope/motion/web/gsap`.
    !import_source_is_gsap_package(source)
        && !import_source_is_gsap_trial(source)
        && source.rsplit('/').next() == Some("gsap")
}

/// Record a dynamic `import("<configured gsap module>")`.
///
/// Bindings from a dynamic import arrive through a `.then()` callback rather
/// than import specifiers, so the local names cannot be resolved the way the
/// static path does. Recording the module-level `gsap` marker is enough: it is
/// the same signal the static default-import branch sets, and it proves the
/// file routes through the entrypoint that owns registration.
fn record_configured_gsap_dynamic_import<'a>(
    import: &oxc_ast::ast::ImportExpression<'a>,
    semantic: &Semantic<'a>,
    node_id: oxc_semantic::NodeId,
    facts: &mut FileFacts,
) {
    let Expression::StringLiteral(source) = import.source.without_parentheses() else {
        return;
    };
    if !import_source_is_configured_gsap_module(source.value.as_str()) {
        return;
    }
    // A dynamic import only *orders* its registration side effect ahead of the
    // plugin use when the promise is consumed: either awaited, or chained with
    // `.then(...)` so the use sits in the callback. A bare floating
    // `void import("./gsap"); ScrollTrigger.create(...)` runs the use before the
    // module resolves, which is exactly the ordering bug this rule catches.
    //
    // Known limitation: a `.then()` chain is accepted file-wide, so a use placed
    // OUTSIDE the callback is not distinguished. Suppressing is the conservative
    // choice for a medium-confidence rule.
    if dynamic_import_is_sequenced(semantic, node_id) {
        facts.configured_gsap_imports.insert("gsap".to_string());
    }
}

/// Whether a dynamic import's promise is actually consumed, so its module
/// side effects are ordered ahead of the dependent code: `await import(...)` or
/// `import(...).then(...)`. A floating import is not sequenced.
fn dynamic_import_is_sequenced(semantic: &Semantic<'_>, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = node_id;
    // Bounded walk: an awaited import is at most a few transparent wrappers deep.
    for _ in 0..6 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        match nodes.kind(parent_id) {
            AstKind::AwaitExpression(_) => return true,
            // `import(...).then(...)` sequences the callback after resolution.
            AstKind::StaticMemberExpression(member) if member.property.name == "then" => {
                return true;
            }
            // Transparent wrappers: keep walking outward.
            AstKind::ParenthesizedExpression(_) | AstKind::TSAsExpression(_) => {
                current = parent_id;
            }
            _ => return false,
        }
    }
    false
}

fn plugin_name_from_import_source(source: &str) -> Option<&str> {
    let name = source.rsplit('/').next()?;
    KNOWN_PLUGINS.contains(&name).then_some(name)
}

fn dev_only_name_from_import_source(source: &str) -> Option<&str> {
    let name = source.rsplit('/').next()?;
    DEV_ONLY_PLUGINS.contains(&name).then_some(name)
}

fn plugin_name_for_identifier<'a>(name: &'a str, facts: &'a FileFacts) -> Option<&'a str> {
    if KNOWN_PLUGINS.contains(&name) {
        Some(name)
    } else {
        facts.plugin_aliases.get(name).map(String::as_str)
    }
}

fn dev_only_name_for_known_or_default<'a>(
    imported_name: &'a str,
    source_dev_only: Option<&'a str>,
) -> Option<&'a str> {
    if DEV_ONLY_PLUGINS.contains(&imported_name) {
        Some(imported_name)
    } else if imported_name == "default" {
        source_dev_only
    } else {
        None
    }
}

fn dev_only_name_for_identifier<'a>(name: &'a str, facts: &'a FileFacts) -> Option<&'a str> {
    if DEV_ONLY_PLUGINS.contains(&name) {
        Some(name)
    } else {
        facts.dev_only_aliases.get(name).map(String::as_str)
    }
}

fn is_gsap_identifier(name: &str, facts: &FileFacts) -> bool {
    name == "gsap" || facts.gsap_bindings.contains(name)
}

/// Record identifiers passed to `gsap.registerPlugin(...)`.
///
/// Handles these argument shapes:
/// - a bare identifier: `gsap.registerPlugin(ScrollTrigger)`,
/// - anything that cannot be resolved statically (a spread `...plugins`, a call
///   result): this sets `registration_unknown`,
///   which suppresses the used-without-register check for the whole file.
fn record_register(call: &CallExpression<'_>, facts: &mut FileFacts) {
    if !is_gsap_member_call(call, facts, "registerPlugin") {
        return;
    }
    for argument in &call.arguments {
        let Some(expression) = argument_expression(argument) else {
            // A spread argument (`...plugins`) cannot be resolved statically.
            facts.registration_unknown = true;
            continue;
        };
        match expression.without_parentheses() {
            Expression::Identifier(identifier) => {
                let name = plugin_name_for_identifier(identifier.name.as_str(), facts)
                    .unwrap_or_else(|| identifier.name.as_str());
                facts.registered.insert(name.to_string());
            }
            // GSAP does not flatten plugin arrays; `[ScrollTrigger]` is one
            // invalid plugin argument, not a successful ScrollTrigger register.
            Expression::ArrayExpression(_) => {}
            // Any other shape (call result, member access, spread) is opaque.
            _ => facts.registration_unknown = true,
        }
    }
}

/// Per-node rule dispatch.
fn check_node<'a, F>(
    node: &AstNode<'a>,
    semantic: &Semantic<'a>,
    relative_path: &str,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    use oxc_ast::AstKind;

    match node.kind() {
        // Rule 1: gsap-trial import.
        AstKind::ImportDeclaration(import)
            if import_source_is_gsap_trial(import.source.value.as_str()) =>
        {
            emit(
                ids::CORE_GSAP_TRIAL_IMPORT,
                Severity::High,
                Confidence::High,
                import.span,
                "Import from the obsolete `gsap-trial` package.".to_string(),
                "Import from `gsap`; every plugin is now free in the standard package.",
            );
        }
        AstKind::CallExpression(call) => {
            check_call(call, node.id(), semantic, facts, emit);
            check_state_in_continuous_motion(call, node.id(), semantic, facts, emit);
        }
        // Rule 2: dev-only helpers referenced in non-test source. Skip TS
        // type-only positions (e.g. `let x: GSDevTools`), which are erased at build time.
        AstKind::IdentifierReference(identifier)
            if dev_only_name_for_identifier(identifier.name.as_str(), facts).is_some()
                && !is_test_or_fixture_path(relative_path)
                && !reference_is_ts_type_position(semantic, node.id()) =>
        {
            let name = dev_only_name_for_identifier(identifier.name.as_str(), facts)
                .unwrap_or_else(|| identifier.name.as_str());
            emit(
                ids::PLUGINS_GSDEVTOOLS_IN_SOURCE,
                Severity::Medium,
                Confidence::Medium,
                identifier.span,
                format!("{name} referenced in source code."),
                "GSAP dev-only tools should be gated behind a dev flag or removed before shipping.",
            );
        }
        _ => {}
    }
}

/// Object-literal rules (markers, scrub+toggleActions conflict).
fn check_object_literal<F>(object: &ObjectExpression<'_>, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let mut has_markers_true = false;
    let mut markers_span = object.span;
    let mut has_scrub = false;
    let mut has_toggle_actions = false;

    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        let Some(key) = property_key_name(&property.key) else {
            continue;
        };
        match key {
            "markers" if expression_is_true(&property.value) => {
                has_markers_true = true;
                markers_span = property.span;
            }
            "scrub" => has_scrub = true,
            "toggleActions" => has_toggle_actions = true,
            _ => {}
        }
    }

    // Rule 3: markers: true.
    if has_markers_true {
        emit(
            ids::SCROLLTRIGGER_MARKERS_IN_PROD,
            Severity::Medium,
            Confidence::Medium,
            markers_span,
            "ScrollTrigger `markers: true` left enabled.".to_string(),
            "Remove `markers: true` (or guard it for development) before shipping.",
        );
    }

    // Rule 4: scrub + toggleActions conflict.
    if has_scrub && has_toggle_actions {
        emit(
            ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS,
            Severity::Medium,
            Confidence::High,
            object.span,
            "ScrollTrigger config sets both `scrub` and `toggleActions`.".to_string(),
            "Pick one: `scrub` ties progress to scroll; `toggleActions` plays on enter/leave.",
        );
    }
}

/// Call-expression rules.
fn check_call<'a, F>(
    call: &CallExpression<'a>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'a>,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    // Rule 6: lagSmoothing(0) / lagSmoothing(false).
    if is_ticker_lag_smoothing_disabled(call, facts) {
        emit(
            ids::PERFORMANCE_LAG_SMOOTHING_DISABLED,
            Severity::Medium,
            Confidence::High,
            call.span,
            "`gsap.ticker.lagSmoothing` is disabled.".to_string(),
            "Leave lag smoothing enabled unless you have measured a specific reason to disable it.",
        );
    }

    // Tween-factory rules: gsap.to/from/fromTo/set(...) and timeline variants.
    if let Some(method) = gsap_tween_method(call, facts) {
        check_tween_in_render(call, node_id, method, semantic, facts, emit);

        // Rule 5: GSAP-2 signature gsap.to(target, <number>, {...}).
        if matches!(method, "to" | "from" | "fromTo")
            && call.arguments.len() >= 2
            && argument_is_numeric_literal(&call.arguments[1])
        {
            emit(
                ids::CORE_GSAP2_SIGNATURE,
                Severity::Medium,
                Confidence::High,
                call.span,
                format!("`gsap.{method}` uses the GSAP-2 duration-as-second-argument signature."),
                "Move the duration into the vars object: gsap.to(target, { duration, ... }).",
            );
        }

        // Layout-prop and ScrollTrigger-config rules run over each vars object.
        // `fromTo` carries two vars objects (fromVars + toVars); scan both.
        for vars in tween_vars_objects(call, method) {
            // Rule 7: layout-prop animation in the vars object.
            if let Some(prop_span) = object_animates_layout_prop(vars) {
                emit(
                    ids::CORE_LAYOUT_PROP_ANIMATION,
                    Severity::Medium,
                    Confidence::Medium,
                    prop_span,
                    "Animating a layout property forces reflow.".to_string(),
                    "Animate transforms (x/y/scale/rotation) instead of top/left/width/height.",
                );
            }
            // Rules 3 & 4 only apply inside nested `scrollTrigger:` configs.
            check_nested_scrolltrigger_configs(vars, emit);

            if let Some(span) = object_will_change_span(vars) {
                emit(
                    ids::PERFORMANCE_WILL_CHANGE_PERMANENT,
                    Severity::Medium,
                    Confidence::Medium,
                    span,
                    "GSAP vars set `will-change` for the animation's full lifetime.".to_string(),
                    "will-change holds a compositor layer permanently; toggle it around the animation or scope it in CSS with removal.",
                );
            }
        }

        check_nested_timeline_scrolltrigger(call, method, facts, semantic, emit);
        check_missing_overwrite(call, node_id, method, semantic, facts, emit);
    }
    if let Some(vars) = gsap_timeline_vars_object(call, facts) {
        check_nested_scrolltrigger_configs(vars, emit);
        if let Some(span) = timeline_defaults_will_change_span(vars) {
            emit(
                ids::PERFORMANCE_WILL_CHANGE_PERMANENT,
                Severity::Medium,
                Confidence::Medium,
                span,
                "GSAP timeline defaults set `will-change` for the animation's full lifetime."
                    .to_string(),
                "will-change holds a compositor layer permanently; toggle it around the animation or scope it in CSS with removal.",
            );
        }
    }
    if is_gsap_member_call(call, facts, "timeline") {
        check_tween_in_render(call, node_id, "timeline", semantic, facts, emit);
    }

    // ScrollTrigger.create({...}): the argument is a ScrollTrigger config.
    if is_plugin_member_call(call, facts, "ScrollTrigger", "create")
        && let Some(config) = call.arguments.first().and_then(argument_expression)
        && let Expression::ObjectExpression(object) = config.without_parentheses()
    {
        check_scrolltrigger_config_object(object, emit);
    }
    // ScrollTrigger.batch(targets, {...}): the second argument is a config.
    if is_plugin_member_call(call, facts, "ScrollTrigger", "batch")
        && let Some(config) = call.arguments.get(1).and_then(argument_expression)
        && let Expression::ObjectExpression(object) = config.without_parentheses()
    {
        check_scrolltrigger_config_object(object, emit);
    }

    // Rule 8: plugin used without registration.
    check_plugin_used_without_register(call, facts, emit);

    // Rules 11 & 12 hang off useGSAP / gsap.context calls.
    if is_gsap_member_call(call, facts, "context") || is_usegsap_call(call, facts) {
        check_unscoped_selectors(call, facts, emit);
    }
    if is_gsap_member_call(call, facts, "context") {
        check_context_missing_revert(call, semantic, emit);
    }
}

/// ScrollTrigger belongs on a timeline itself, not on a child tween in a
/// fluent chain or on a stored timeline handle. A call-expression receiver is
/// the precise signal for the chained form (`tl.from(...).to(...)` or
/// `gsap.timeline().to(...)`); an identifier receiver counts when it resolves
/// to a recorded `gsap.timeline(...)` handle (`const tl = gsap.timeline()`).
fn check_nested_timeline_scrolltrigger<F>(
    call: &CallExpression<'_>,
    method: &str,
    facts: &FileFacts,
    semantic: &Semantic<'_>,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return;
    };
    let receiver_is_timeline = match member.object.without_parentheses() {
        Expression::CallExpression(_) => true,
        Expression::Identifier(identifier) => {
            identifier_is_timeline_handle(semantic, identifier, facts)
        }
        _ => false,
    };
    if !receiver_is_timeline {
        return;
    }
    for vars in tween_vars_objects(call, method) {
        if let Some(span) = object_key_span(vars, "scrollTrigger") {
            emit(
                ids::SCROLLTRIGGER_NESTED_TIMELINE_CHILD,
                Severity::Medium,
                Confidence::High,
                span,
                "ScrollTrigger only belongs on the top-level animation, never on timeline children."
                    .to_string(),
                "Move the scrollTrigger config to gsap.timeline({ scrollTrigger: ... }).",
            );
        }
    }
}

/// A tween created directly in a capitalized React component function runs on
/// every render. Hook callbacks and event handlers are deliberately excluded.
fn check_tween_in_render<F>(
    call: &CallExpression<'_>,
    node_id: oxc_semantic::NodeId,
    method: &str,
    semantic: &Semantic<'_>,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    if !facts.has_react_import && !facts.has_jsx {
        return;
    }
    let Some(function_id) = nearest_enclosing_function(semantic, node_id) else {
        return;
    };
    if function_is_hook_callback(semantic, function_id, facts)
        || function_is_event_handler(semantic, function_id)
    {
        return;
    }
    let Some(name) = enclosing_function_name(semantic, function_id) else {
        // Anonymous function directly exported as default and returning JSX
        // is a component (`export default () => { ... return <div />; }`); a
        // name test is impossible, so the default-export position plus JSX
        // decides, mirroring the expo-motion analyzer.
        if function_is_default_exported(semantic, function_id)
            && function_returns_jsx(semantic, function_id)
        {
            emit(
                ids::REACT_TWEEN_IN_RENDER,
                Severity::High,
                Confidence::Medium,
                call.span,
                format!(
                    "`{method}` creates a GSAP animation during a default-exported component render."
                ),
                "Create the animation in useGSAP, useLayoutEffect, or useEffect so it runs after render and cleans up.",
            );
        }
        return;
    };
    if !starts_with_ascii_uppercase(&name) {
        return;
    }
    emit(
        ids::REACT_TWEEN_IN_RENDER,
        Severity::High,
        Confidence::Medium,
        call.span,
        format!("`{method}` creates a GSAP animation during `{name}` render."),
        "Create the animation in useGSAP, useLayoutEffect, or useEffect so it runs after render and cleans up.",
    );
}

/// Event-driven tweens can stack when interactions arrive faster than they
/// finish. Only literal vars objects are checked; a spread may already carry an
/// overwrite policy and is therefore suppressed.
fn check_missing_overwrite<F>(
    call: &CallExpression<'_>,
    node_id: oxc_semantic::NodeId,
    method: &str,
    semantic: &Semantic<'_>,
    _facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    if !matches!(method, "to" | "from") {
        return;
    }
    let Some(function_id) = nearest_enclosing_function(semantic, node_id) else {
        return;
    };
    if !function_is_event_handler(semantic, function_id) {
        return;
    }
    let Some(vars) = tween_vars_objects(call, method).into_iter().next() else {
        return;
    };
    if object_has_key(vars, "overwrite") || object_has_spread(vars) {
        return;
    }
    emit(
        ids::CORE_MISSING_OVERWRITE,
        Severity::Low,
        Confidence::Medium,
        vars.span,
        format!("Event-handler `gsap.{method}` tween has no overwrite policy."),
        "Add `overwrite: \"auto\"` or use gsap.quickTo for repeated interactions.",
    );
}

fn nearest_enclosing_function(
    semantic: &Semantic<'_>,
    node_id: oxc_semantic::NodeId,
) -> Option<oxc_semantic::NodeId> {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = node_id;
    loop {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return None;
        }
        if matches!(
            nodes.kind(parent_id),
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        ) {
            return Some(parent_id);
        }
        current = parent_id;
    }
}

fn enclosing_function_name(
    semantic: &Semantic<'_>,
    function_id: oxc_semantic::NodeId,
) -> Option<String> {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    match nodes.kind(function_id) {
        AstKind::Function(function) => function
            .id
            .as_ref()
            .map(|identifier| identifier.name.as_str().to_string())
            .or_else(|| function_binding_name(nodes, function_id)),
        AstKind::ArrowFunctionExpression(_) => function_binding_name(nodes, function_id),
        _ => None,
    }
}

fn function_binding_name(
    nodes: &oxc_semantic::AstNodes<'_>,
    function_id: oxc_semantic::NodeId,
) -> Option<String> {
    use oxc_ast::AstKind;

    let parent_id = nodes.parent_id(function_id);
    match nodes.kind(parent_id) {
        AstKind::VariableDeclarator(declarator) => declarator
            .id
            .get_binding_identifier()
            .map(|identifier| identifier.name.as_str().to_string()),
        AstKind::ObjectProperty(property) => property_key_name(&property.key).map(str::to_string),
        // Transparent wrappers preserve the function identity, so the name
        // lives on the declarator above the call: `const Card = memo(() => ...)`
        // names the arrow `Card`.
        AstKind::CallExpression(call) if callee_is_transparent_wrapper(call) => {
            function_binding_name(nodes, parent_id)
        }
        _ => None,
    }
}

/// Whether a call preserves its function argument's identity for naming:
/// `memo` and `forwardRef` return the function/component.
fn callee_is_transparent_wrapper(call: &CallExpression<'_>) -> bool {
    match call.callee.without_parentheses() {
        Expression::Identifier(identifier) => {
            matches!(identifier.name.as_str(), "memo" | "forwardRef")
        }
        _ => false,
    }
}

fn function_is_hook_callback(
    semantic: &Semantic<'_>,
    function_id: oxc_semantic::NodeId,
    facts: &FileFacts,
) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = function_id;
    for _ in 0..6 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        match nodes.kind(parent_id) {
            AstKind::CallExpression(call) => {
                return match call.callee.without_parentheses() {
                    Expression::Identifier(identifier) => {
                        matches!(
                            identifier.name.as_str(),
                            "useEffect" | "useLayoutEffect" | "useGSAP"
                        ) || facts.usegsap_bindings.contains(identifier.name.as_str())
                    }
                    Expression::StaticMemberExpression(member) => matches!(
                        member.property.name.as_str(),
                        "useEffect" | "useLayoutEffect"
                    ),
                    _ => false,
                };
            }
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => return false,
            _ => current = parent_id,
        }
    }
    false
}

/// Whether a `gsap.matchMedia(...)` call node executes while a `useGSAP(...)`
/// context is active: its nearest enclosing function must be passed directly
/// as a `useGSAP(...)` argument. Lexical nesting alone is not enough: a call
/// inside a deferred callback (`setTimeout(() => gsap.matchMedia())`) runs
/// after the `useGSAP` callback returns, so the context never registers it.
/// Only ancestors count: a sibling `useGSAP(...)` elsewhere in the file cannot
/// clean up this call.
fn call_inside_usegsap(
    semantic: &Semantic<'_>,
    node_id: oxc_semantic::NodeId,
    facts: &FileFacts,
) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    // Nearest enclosing function: crossing it means deferred execution.
    let mut current = node_id;
    let function_id = loop {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        if matches!(
            nodes.kind(parent_id),
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
        ) {
            break parent_id;
        }
        current = parent_id;
    };
    // Covered only when that function is a direct `useGSAP(...)` argument.
    // (A `useGSAP` callee is always an identifier, so a function parented
    // directly under the call can only sit in argument position.)
    let call_id = nodes.parent_id(function_id);
    if call_id == function_id {
        return false;
    }
    if let AstKind::CallExpression(call) = nodes.kind(call_id)
        && is_usegsap_call(call, facts)
    {
        return true;
    }
    false
}

/// The `mm` in `const mm = gsap.matchMedia()`, when the call result is
/// directly assigned to a binding. Only that binding's own `.revert()` call
/// proves cleanup; any other object's revert is unrelated.
fn match_media_binding(
    semantic: &Semantic<'_>,
    node_id: oxc_semantic::NodeId,
) -> Option<MatchMediaBinding> {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    match nodes.kind(nodes.parent_id(node_id)) {
        AstKind::VariableDeclarator(declarator) => {
            declarator
                .id
                .get_binding_identifier()
                .map(|identifier| MatchMediaBinding {
                    name: identifier.name.as_str().to_string(),
                    declaration: identifier
                        .symbol_id
                        .get()
                        .map(|symbol| semantic.scoping().symbol_declaration(symbol)),
                })
        }
        _ => None,
    }
}

/// The declaration node an identifier reference resolves to, if any.
fn reference_declaration(
    semantic: &Semantic<'_>,
    identifier: &oxc_ast::ast::IdentifierReference<'_>,
) -> Option<oxc_semantic::NodeId> {
    let reference_id = identifier.reference_id.get()?;
    let symbol_id = semantic.scoping().get_reference(reference_id).symbol_id()?;
    Some(semantic.scoping().symbol_declaration(symbol_id))
}

fn function_is_event_handler(semantic: &Semantic<'_>, function_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    if enclosing_function_name(semantic, function_id)
        .is_some_and(|name| starts_with_event_handler_prefix(&name))
    {
        return true;
    }
    let nodes = semantic.nodes();
    let mut current = function_id;
    for _ in 0..6 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        match nodes.kind(parent_id) {
            AstKind::JSXAttribute(attribute) => {
                return attribute
                    .name
                    .as_identifier()
                    .is_some_and(|name| starts_with_event_handler_prefix(name.name.as_str()));
            }
            AstKind::Function(_) | AstKind::ArrowFunctionExpression(_) => return false,
            _ => current = parent_id,
        }
    }
    false
}

fn starts_with_event_handler_prefix(name: &str) -> bool {
    name.strip_prefix("on")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|character| character.is_ascii_uppercase())
}

fn starts_with_ascii_uppercase(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
}

/// Whether a function node is the declaration of an `export default`
/// (`export default function () {}` or `export default () => {}`).
fn function_is_default_exported(
    semantic: &Semantic<'_>,
    function_id: oxc_semantic::NodeId,
) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = function_id;
    for _ in 0..3 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        match nodes.kind(parent_id) {
            AstKind::ExportDefaultDeclaration(_) => return true,
            // `export default memo(() => {})`-style wrappers are not a bare
            // default export; only direct position counts.
            AstKind::CallExpression(_)
            | AstKind::VariableDeclarator(_)
            | AstKind::Function(_)
            | AstKind::ArrowFunctionExpression(_) => return false,
            _ => current = parent_id,
        }
    }
    false
}

/// Whether a function subtree contains a JSX element or fragment, marking it
/// as rendering UI rather than running plain logic.
fn function_returns_jsx(semantic: &Semantic<'_>, function_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    nodes.iter().any(|node| {
        if !matches!(
            node.kind(),
            AstKind::JSXElement(_) | AstKind::JSXFragment(_)
        ) {
            return false;
        }
        let mut current = node.id();
        loop {
            let parent_id = nodes.parent_id(current);
            if parent_id == current {
                return false;
            }
            if parent_id == function_id {
                return true;
            }
            // A nested function boundary means the JSX belongs to the inner
            // function, not the candidate component.
            if matches!(
                nodes.kind(parent_id),
                AstKind::Function(_) | AstKind::ArrowFunctionExpression(_)
            ) {
                return false;
            }
            current = parent_id;
        }
    })
}

/// Rule 8: a known plugin identifier is *used* (member object or bare ref in a
/// call argument context) but never registered in this file.
fn check_plugin_used_without_register<F>(call: &CallExpression<'_>, facts: &FileFacts, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    // If any registerPlugin argument could not be resolved statically (a spread
    // or computed value), we cannot prove the plugin was *not* registered, so
    // suppress the check for the whole file to avoid false positives.
    if facts.registration_unknown {
        return;
    }
    // Treat `<Plugin>.something(...)` calls and `gsap.something(<Plugin>)`
    // usage as "used". The simplest stable signal: a callee whose object is a
    // known plugin identifier, e.g. ScrollTrigger.create(...).
    if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses()
        && let Expression::Identifier(object) = member.object.without_parentheses()
        && let Some(name) = plugin_name_for_identifier(object.name.as_str(), facts)
        && !facts.registered.contains(name)
        && !facts.configured_gsap_imports.contains(name)
        && !facts.configured_gsap_imports.contains("gsap")
    {
        emit(
            ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER,
            Severity::High,
            Confidence::Medium,
            member.span,
            format!("`{name}` is used but never passed to gsap.registerPlugin in this file."),
            "Call gsap.registerPlugin(<Plugin>) once before using the plugin.",
        );
    }

    // ScrollTrigger is also "used" when a gsap tween/timeline passes a
    // `scrollTrigger:` config object, even though the callee is `gsap` rather
    // than `ScrollTrigger` — e.g. gsap.to(target, { scrollTrigger: { ... } }).
    if !facts.registered.contains("ScrollTrigger")
        && !facts.configured_gsap_imports.contains("ScrollTrigger")
        && !facts.configured_gsap_imports.contains("gsap")
        && let Some(span) = scrolltrigger_config_span(call, facts)
    {
        emit(
            ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER,
            Severity::High,
            Confidence::Medium,
            span,
            "ScrollTrigger is used (via a `scrollTrigger` config) but never passed to gsap.registerPlugin in this file."
                .to_string(),
            "Call gsap.registerPlugin(ScrollTrigger) once before using ScrollTrigger.",
        );
    }

    for (vars_key, plugin) in PLUGIN_VARS {
        if facts.registered.contains(*plugin)
            || facts.configured_gsap_imports.contains(*plugin)
            || facts.configured_gsap_imports.contains("gsap")
        {
            continue;
        }
        if let Some(span) = plugin_vars_key_span(call, facts, vars_key) {
            emit(
                ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER,
                Severity::High,
                Confidence::Medium,
                span,
                format!(
                    "`{vars_key}` vars use {plugin}, but `{plugin}` is never passed to gsap.registerPlugin in this file."
                ),
                "Call gsap.registerPlugin(<Plugin>) once before using plugin vars.",
            );
        }
    }
}

/// Rule 11: useGSAP/gsap.context callback uses string-literal selectors while
/// no scope is supplied. Uses argument structure (semantic-aware traversal of
/// the callback body via the node walk would double-report, so we inspect the
/// call's own arguments here).
fn check_unscoped_selectors<F>(call: &CallExpression<'_>, facts: &FileFacts, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let scoped = call_has_scope(call, facts);
    if scoped {
        return;
    }
    // The callback is the first argument; look for string-literal selectors in
    // gsap tween calls inside it.
    let Some(first) = call.arguments.first().and_then(argument_expression) else {
        return;
    };
    if let Some(span) = first_string_selector_in_callback(first, facts) {
        emit(
            ids::REACT_UNSCOPED_SELECTOR,
            Severity::Medium,
            Confidence::Medium,
            span,
            "String selector used inside useGSAP/gsap.context without a scope.".to_string(),
            "Pass a scope (useGSAP(cb, { scope: ref }) or gsap.context(cb, scopeRef)) or use refs.",
        );
    }
}

/// Rule 12: `const ctx = gsap.context(...)` whose binding is never `.revert()`-ed
/// and not returned.
///
/// Uses oxc_semantic to resolve the declared symbol and inspect its resolved
/// references. We find the enclosing `VariableDeclarator` of the call, take its
/// bound identifier symbol, and check whether any reference is the object of a
/// `.revert()` member call or appears in a `return`.
///
/// Limitation: this is same-scope by construction. If the context handle is
/// stored on an object/ref and reverted elsewhere (e.g. inside a returned
/// cleanup closure that reads it via a different binding), we cannot follow it
/// and may report a false positive; confidence is therefore medium.
fn check_context_missing_revert<'a, F>(
    call: &CallExpression<'a>,
    semantic: &Semantic<'a>,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    use oxc_ast::AstKind;

    // Find the node id of this call, then climb to a VariableDeclarator parent.
    let Some(call_node_id) = find_node_id_for_span(
        semantic,
        call.span,
        |kind| matches!(kind, AstKind::CallExpression(inner) if inner.span == call.span),
    ) else {
        return;
    };

    let nodes = semantic.nodes();
    let mut current = call_node_id;
    let mut declarator_name: Option<&str> = None;
    let mut declarator_span = call.span;
    // Climb a bounded number of parents to find the binding declarator.
    for _ in 0..6 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            break;
        }
        if let AstKind::VariableDeclarator(declarator) = nodes.kind(parent_id) {
            if let Some(identifier) = declarator.id.get_binding_identifier() {
                declarator_name = Some(identifier.name.as_str());
                declarator_span = declarator.span;
            }
            break;
        }
        current = parent_id;
    }

    let Some(name) = declarator_name else {
        if call_result_is_discarded_statement(nodes, call_node_id) {
            emit(
                ids::REACT_CONTEXT_MISSING_REVERT,
                Severity::High,
                Confidence::High,
                call.span,
                "gsap.context() result is discarded and cannot be reverted for cleanup."
                    .to_string(),
                "Store the context and return `() => ctx.revert()` so animations are torn down.",
            );
        }
        // Other unbound shapes may be returned directly, passed through, etc.
        return;
    };

    // Resolve the symbol for this binding name via the scoping table.
    let scoping = semantic.scoping();
    let mut reverted_or_returned = false;
    let mut found_symbol = false;
    for symbol_id in scoping.symbol_ids() {
        if scoping.symbol_name(symbol_id) != name {
            continue;
        }
        // Make sure this is the binding we found (match declaration span).
        let decl_node = scoping.symbol_declaration(symbol_id);
        let decl_kind = nodes.kind(decl_node);
        let decl_matches = match decl_kind {
            AstKind::VariableDeclarator(declarator) => declarator.span == declarator_span,
            _ => false,
        };
        if !decl_matches {
            continue;
        }
        found_symbol = true;
        for reference in scoping.get_resolved_references(symbol_id) {
            if reference_is_revert_or_return(nodes, reference.node_id()) {
                reverted_or_returned = true;
                break;
            }
        }
        break;
    }

    if found_symbol && !reverted_or_returned {
        emit(
            ids::REACT_CONTEXT_MISSING_REVERT,
            Severity::High,
            Confidence::Medium,
            declarator_span,
            format!("gsap.context() stored in `{name}` is never reverted or returned for cleanup."),
            "Return `() => ctx.revert()` (or call ctx.revert() in cleanup) so animations are torn down.",
        );
    }
}

/// File-level rules: SSR placement and useGSAP-not-registered.
fn check_file_level(
    program: &Program<'_>,
    relative_path: &str,
    facts: &FileFacts,
    line_index: &LineIndex,
    findings: &mut Vec<Finding>,
) {
    // Rule 9: useGSAP imported but never registered with registerPlugin.
    if !facts.usegsap_bindings.is_empty()
        && !facts.configured_gsap_imports.contains("gsap")
        && !facts.configured_gsap_imports.contains("useGSAP")
        && !facts
            .usegsap_bindings
            .iter()
            .any(|binding| facts.registered.contains(binding))
    {
        let span = program.span;
        let (line, column) = line_index.line_col(span.start);
        findings.push(Finding {
            id: ids::REACT_USEGSAP_NOT_REGISTERED.to_string(),
            category: Category::React,
            severity: Severity::Medium,
            confidence: Confidence::Medium,
            file: relative_path.to_string(),
            line,
            column,
            message: "useGSAP imported from @gsap/react but never registered.".to_string(),
            suggestion: "Call gsap.registerPlugin(useGSAP) once so the hook is recognized."
                .to_string(),
        });
    }

    // Rule 10: GSAP used in an App Router file without "use client".
    if is_under_app(relative_path)
        && (facts.uses_gsap_surface || !facts.usegsap_bindings.is_empty())
        && !facts.has_use_client
    {
        let span = program.span;
        let (line, column) = line_index.line_col(span.start);
        findings.push(Finding {
            id: ids::REACT_GSAP_IN_SSR.to_string(),
            category: Category::React,
            severity: Severity::Medium,
            confidence: Confidence::Medium,
            file: relative_path.to_string(),
            line,
            column,
            message: "GSAP used in an app/ file without a \"use client\" directive.".to_string(),
            suggestion: "Add \"use client\" at the top of the file; GSAP needs the browser."
                .to_string(),
        });
    }

    // One finding per uncovered call site: collapsing to a single file-level
    // finding lets a new leak hide behind an already-baselined one.
    for call in facts.match_media_calls.iter().filter(|call| {
        !call.inside_usegsap
            && call.binding.as_ref().is_none_or(|binding| {
                !facts
                    .reverted_bindings
                    .iter()
                    .any(|revert| revert_covers_binding(revert, binding))
            })
    }) {
        let (line, column) = line_index.line_col(call.span.start);
        findings.push(Finding {
            id: ids::REACT_MATCHMEDIA_MISSING_REVERT.to_string(),
            category: Category::React,
            severity: Severity::Medium,
            confidence: Confidence::Medium,
            file: relative_path.to_string(),
            line,
            column,
            message: "gsap.matchMedia() is used without revert cleanup or useGSAP auto-cleanup."
                .to_string(),
            suggestion: "matchMedia contexts leak on unmount outside useGSAP auto-cleanup; call mm.revert() in cleanup."
                .to_string(),
        });
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Path is under a Next.js App Router route root: `app/` or `src/app/`.
fn is_under_app(path: &str) -> bool {
    let segments = path
        .split(['/', '\\'])
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>();
    segments.first() == Some(&"app") || segments.windows(2).any(|pair| pair == ["src", "app"])
}

fn is_test_or_fixture_path(path: &str) -> bool {
    let mut segments = path
        .split(['/', '\\'])
        .filter(|segment| !segment.is_empty());
    let Some(file_name) = segments.next_back() else {
        return false;
    };
    if segments.any(|segment| matches!(segment, "__tests__" | "__fixtures__" | "fixtures")) {
        return true;
    }
    file_name.contains(".test.")
        || file_name.contains(".spec.")
        || file_name.starts_with("test.")
        || file_name.starts_with("spec.")
}

/// Return the static property name of an object key, if it is a plain
/// identifier or string key.
fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn object_has_key(object: &ObjectExpression<'_>, name: &str) -> bool {
    object.properties.iter().any(|property| {
        matches!(
            property,
            ObjectPropertyKind::ObjectProperty(inner) if property_key_name(&inner.key) == Some(name)
        )
    })
}

fn object_key_span(object: &ObjectExpression<'_>, name: &str) -> Option<Span> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(inner) = property else {
            return None;
        };
        (property_key_name(&inner.key) == Some(name)).then_some(inner.span)
    })
}

fn object_has_spread(object: &ObjectExpression<'_>) -> bool {
    object
        .properties
        .iter()
        .any(|property| matches!(property, ObjectPropertyKind::SpreadProperty(_)))
}

fn object_will_change_span(object: &ObjectExpression<'_>) -> Option<Span> {
    ["willChange", "will-change"]
        .into_iter()
        .find_map(|name| will_change_hint_span(object, name))
}

fn will_change_hint_span(object: &ObjectExpression<'_>, name: &str) -> Option<Span> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(inner) = property else {
            return None;
        };
        if property_key_name(&inner.key) != Some(name) {
            return None;
        }
        if !will_change_value_is_hint(&inner.value) {
            return None;
        }
        // `clearProps` in the same vars object removes the hint when the
        // tween completes, so the layer is not permanently retained.
        if object_clears_will_change(object) {
            return None;
        }
        Some(inner.span)
    })
}

/// Whether a vars object clears `willChange` on complete via `clearProps`:
/// `clearProps: "all"`, or a list naming `willChange`/`will-change`.
fn object_clears_will_change(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(inner) = property else {
            return false;
        };
        if property_key_name(&inner.key) != Some("clearProps") {
            return false;
        }
        match inner.value.without_parentheses() {
            Expression::StringLiteral(literal) => {
                let value = literal.value.as_str();
                value.trim() == "all"
                    || value
                        .split(',')
                        .map(str::trim)
                        .any(|part| matches!(part, "willChange" | "will-change"))
            }
            _ => false,
        }
    })
}

/// Whether a `will-change` vars value actually requests a compositor layer.
/// Reset keywords (`auto` and the CSS-wide keywords) release the layer, so
/// cleanup such as `gsap.set(el, { willChange: "auto" })` is not a finding.
/// Non-literal values still fire: an undecidable value may hold a layer.
fn will_change_value_is_hint(value: &Expression<'_>) -> bool {
    let reset = |text: &str| {
        matches!(
            text.trim().to_ascii_lowercase().as_str(),
            "auto" | "initial" | "inherit" | "unset" | "revert" | "revert-layer" | ""
        )
    };
    match value.without_parentheses() {
        Expression::StringLiteral(literal) => !reset(literal.value.as_str()),
        Expression::TemplateLiteral(literal) if literal.expressions.is_empty() => literal
            .quasis
            .first()
            .and_then(|quasi| quasi.value.cooked.as_ref())
            .is_none_or(|cooked| !reset(cooked.as_str())),
        _ => true,
    }
}

fn timeline_defaults_will_change_span(object: &ObjectExpression<'_>) -> Option<Span> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property_key_name(&property.key) != Some("defaults") {
            return None;
        }
        let Expression::ObjectExpression(defaults) = property.value.without_parentheses() else {
            return None;
        };
        object_will_change_span(defaults)
    })
}

/// Whether an expression is the boolean literal `true`.
fn expression_is_true(expression: &Expression<'_>) -> bool {
    matches!(expression.without_parentheses(), Expression::BooleanLiteral(boolean) if boolean.value)
}

/// Get the inner [`Expression`] of an [`Argument`], skipping spreads.
///
/// `Argument` inherits every `Expression` variant via oxc's enum-inheritance
/// macro, which generates the safe public [`Argument::as_expression`] accessor.
/// Spread elements return `None`.
fn argument_expression<'a>(argument: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
    argument.as_expression()
}

/// Whether an argument is a numeric literal.
fn argument_is_numeric_literal(argument: &Argument<'_>) -> bool {
    matches!(
        argument_expression(argument).map(Expression::without_parentheses),
        Some(Expression::NumericLiteral(_))
    )
}

fn is_gsap_member_call(call: &CallExpression<'_>, facts: &FileFacts, method: &str) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return false;
    };
    if member.property.name.as_str() != method {
        return false;
    }
    matches!(
        member.object.without_parentheses(),
        Expression::Identifier(identifier) if is_gsap_identifier(identifier.name.as_str(), facts)
    )
}

fn is_plugin_member_call(
    call: &CallExpression<'_>,
    facts: &FileFacts,
    plugin: &str,
    method: &str,
) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return false;
    };
    if member.property.name.as_str() != method {
        return false;
    }
    matches!(
        member.object.without_parentheses(),
        Expression::Identifier(identifier)
            if plugin_name_for_identifier(identifier.name.as_str(), facts) == Some(plugin)
    )
}

fn is_usegsap_call(call: &CallExpression<'_>, facts: &FileFacts) -> bool {
    matches!(
        call.callee.without_parentheses(),
        Expression::Identifier(identifier)
            if identifier.name.as_str() == "useGSAP"
                || facts.usegsap_bindings.contains(identifier.name.as_str())
    )
}

/// If the call is a GSAP tween factory (`gsap.to`, `tl.to`,
/// `gsap.timeline().to`, etc.), return the method.
fn gsap_tween_method<'a>(call: &'a CallExpression<'a>, facts: &FileFacts) -> Option<&'a str> {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return None;
    };
    let method = member.property.name.as_str();
    if TWEEN_METHODS.contains(&method) && expression_is_gsap_tween_owner(&member.object, facts) {
        Some(method)
    } else {
        None
    }
}

/// If the call uses ScrollTrigger implicitly via a `scrollTrigger:` config
/// object — `gsap.to/from/fromTo/set(target, { scrollTrigger: {...} })` or
/// `gsap.timeline({ scrollTrigger: {...} })` — return that property's span.
fn scrolltrigger_config_span<'a>(call: &'a CallExpression<'a>, facts: &FileFacts) -> Option<Span> {
    let vars_objects: Vec<&'a ObjectExpression<'a>> =
        if let Some(method) = gsap_tween_method(call, facts) {
            tween_vars_objects(call, method)
        } else if is_gsap_member_call(call, facts, "timeline") {
            gsap_timeline_vars_object(call, facts).into_iter().collect()
        } else {
            return None;
        };
    for vars in vars_objects {
        for property in &vars.properties {
            if let ObjectPropertyKind::ObjectProperty(inner) = property
                && property_key_name(&inner.key) == Some("scrollTrigger")
            {
                return Some(inner.span);
            }
        }
    }
    None
}

fn plugin_vars_key_span<'a>(
    call: &'a CallExpression<'a>,
    facts: &FileFacts,
    vars_key: &str,
) -> Option<Span> {
    let method = gsap_tween_method(call, facts)?;
    for vars in tween_vars_objects(call, method) {
        for property in &vars.properties {
            if let ObjectPropertyKind::ObjectProperty(inner) = property
                && property_key_name(&inner.key) == Some(vars_key)
            {
                return Some(inner.span);
            }
        }
    }
    None
}

fn gsap_timeline_vars_object<'a>(
    call: &'a CallExpression<'a>,
    facts: &FileFacts,
) -> Option<&'a ObjectExpression<'a>> {
    if !is_gsap_member_call(call, facts, "timeline") {
        return None;
    }
    match call
        .arguments
        .first()
        .and_then(argument_expression)
        .map(Expression::without_parentheses)
    {
        Some(Expression::ObjectExpression(object)) => Some(object),
        _ => None,
    }
}

fn expression_is_gsap_tween_owner(expression: &Expression<'_>, facts: &FileFacts) -> bool {
    match expression.without_parentheses() {
        Expression::Identifier(identifier) => {
            is_gsap_identifier(identifier.name.as_str(), facts)
                || facts
                    .timeline_handles
                    .iter()
                    .any(|handle| handle.name == identifier.name.as_str())
        }
        Expression::CallExpression(call) => {
            is_gsap_member_call(call, facts, "timeline")
                || gsap_tween_method(call, facts).is_some()
                || timeline_chain_method_returns_timeline(call, facts)
        }
        _ => false,
    }
}

fn timeline_chain_method_returns_timeline(call: &CallExpression<'_>, facts: &FileFacts) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return false;
    };
    TIMELINE_CHAIN_METHODS.contains(&member.property.name.as_str())
        && expression_is_gsap_tween_owner(&member.object, facts)
}

fn expression_is_gsap_timeline_call(expression: &Expression<'_>, facts: &FileFacts) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::CallExpression(call) if is_gsap_member_call(call, facts, "timeline")
    )
}

/// Return every vars object literal for a tween call.
///
/// `to`/`from`/`set(target, vars[, position])` carry one vars object.
/// `fromTo(target, fromVars, toVars[, position])` carries two: layout-prop and
/// config rules must scan BOTH, since either object can set offending properties.
fn tween_vars_objects<'a>(
    call: &'a CallExpression<'a>,
    method: &str,
) -> Vec<&'a ObjectExpression<'a>> {
    let needed = if method == "fromTo" { 2 } else { 1 };
    call.arguments
        .iter()
        .skip(1)
        .filter_map(|argument| {
            match argument_expression(argument).map(Expression::without_parentheses) {
                Some(Expression::ObjectExpression(object)) => Some(object.as_ref()),
                _ => None,
            }
        })
        .take(needed)
        .collect()
}

fn check_scrolltrigger_config_object<F>(object: &ObjectExpression<'_>, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    check_object_literal(object, emit);
}

fn check_nested_scrolltrigger_configs<F>(object: &ObjectExpression<'_>, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property_key_name(&property.key) == Some("scrollTrigger")
            && let Expression::ObjectExpression(nested) = property.value.without_parentheses()
        {
            check_scrolltrigger_config_object(nested, emit);
        }
    }
}

/// If an object literal animates a layout property, return that property's span.
fn object_animates_layout_prop(object: &ObjectExpression<'_>) -> Option<Span> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if let Some(key) = property_key_name(&property.key)
            && LAYOUT_PROPS.contains(&key)
        {
            return Some(property.span);
        }
    }
    None
}

/// Whether a call is `gsap.ticker.lagSmoothing(0|false)`.
fn is_ticker_lag_smoothing_disabled(call: &CallExpression<'_>, facts: &FileFacts) -> bool {
    let Expression::StaticMemberExpression(outer) = call.callee.without_parentheses() else {
        return false;
    };
    if outer.property.name.as_str() != "lagSmoothing" {
        return false;
    }
    // outer.object should be `gsap.ticker`.
    let Expression::StaticMemberExpression(inner) = outer.object.without_parentheses() else {
        return false;
    };
    if inner.property.name.as_str() != "ticker" {
        return false;
    }
    let is_gsap = matches!(
        inner.object.without_parentheses(),
        Expression::Identifier(identifier) if is_gsap_identifier(identifier.name.as_str(), facts)
    );
    if !is_gsap {
        return false;
    }
    let Some(first) = call.arguments.first().and_then(argument_expression) else {
        return false;
    };
    expression_is_disabled_lag_smoothing_arg(first)
}

/// Whether an argument expression disables lag smoothing: `0`, `-0`, or `false`.
fn expression_is_disabled_lag_smoothing_arg(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::NumericLiteral(number) => number.value == 0.0,
        Expression::BooleanLiteral(boolean) => !boolean.value,
        // `-0` parses as a unary negation (`-`) over the literal 0. Compare the
        // operator via its source string to avoid importing the operator enum
        // from the transitive `oxc_syntax` crate.
        Expression::UnaryExpression(unary) if unary.operator.as_str() == "-" => {
            matches!(
                unary.argument.without_parentheses(),
                Expression::NumericLiteral(number) if number.value == 0.0
            )
        }
        _ => false,
    }
}

/// Whether a useGSAP/gsap.context call supplies a scope.
///
/// - `gsap.context(cb, scope)` -> a present second argument is the scope.
/// - `useGSAP(cb, { scope })` -> a config object with a `scope` key.
fn call_has_scope(call: &CallExpression<'_>, facts: &FileFacts) -> bool {
    let Some(second) = call.arguments.get(1).and_then(argument_expression) else {
        return false;
    };
    // `useGSAP(cb, deps)` takes a dependency array (or a config object) as its
    // second argument, so a non-config second argument is NOT a scope. Only
    // `gsap.context(cb, scopeRef)` passes the scope element directly as the
    // second argument.
    let is_use_gsap = is_usegsap_call(call, facts);
    match second.without_parentheses() {
        // Config object (either call form): scoped only if it has a `scope` key.
        Expression::ObjectExpression(object) => object_has_key(object, "scope"),
        // A dependency array (useGSAP's useEffect-style overload) is never a scope.
        Expression::ArrayExpression(_) => false,
        Expression::Identifier(identifier) if is_use_gsap => facts
            .scoped_usegsap_configs
            .contains(identifier.name.as_str()),
        // gsap.context(cb, scopeRef): the bare second argument is the scope.
        // useGSAP(cb, depsVar): a non-object/non-array second argument is deps.
        _ => !is_use_gsap,
    }
}

/// Find the first string-literal selector passed to a gsap tween inside a
/// callback expression (an arrow or function expression). Returns its span.
fn first_string_selector_in_callback(callback: &Expression<'_>, facts: &FileFacts) -> Option<Span> {
    let mut found: Option<Span> = None;

    let mut inspect = |expression: &Expression<'_>| {
        if found.is_some() {
            return;
        }
        if let Expression::CallExpression(call) = expression
            && gsap_tween_method(call, facts).is_some()
            && let Some(first) = call.arguments.first().and_then(argument_expression)
            && let Expression::StringLiteral(string) = first.without_parentheses()
        {
            found = Some(string.span);
        }
    };

    match callback.without_parentheses() {
        Expression::ArrowFunctionExpression(arrow) => {
            if let Some(expression) = arrow.get_expression() {
                walk_expression(expression, &mut inspect);
            } else {
                for statement in &arrow.body.statements {
                    for_each_expression_in_statement(statement, &mut inspect);
                }
            }
        }
        Expression::FunctionExpression(function) => {
            let Some(body) = &function.body else {
                return None;
            };
            for statement in &body.statements {
                for_each_expression_in_statement(statement, &mut inspect);
            }
        }
        _ => return None,
    }

    found
}

/// Find a node id whose kind matches a predicate and whose span equals `span`.
fn find_node_id_for_span<'a, P>(
    semantic: &Semantic<'a>,
    span: Span,
    predicate: P,
) -> Option<oxc_semantic::NodeId>
where
    P: Fn(oxc_ast::AstKind<'a>) -> bool,
{
    for node in semantic.nodes() {
        if node.span() == span && predicate(node.kind()) {
            return Some(node.id());
        }
    }
    None
}

fn call_result_is_discarded_statement(
    nodes: &oxc_semantic::AstNodes<'_>,
    node_id: oxc_semantic::NodeId,
) -> bool {
    use oxc_ast::AstKind;

    matches!(nodes.parent_kind(node_id), AstKind::ExpressionStatement(_))
}

/// Whether the reference at `node_id` is torn down for cleanup: it is the
/// object of a `.revert()`/`.kill()` member call, or it is the bare returned
/// value (`return ctx;`).
///
/// The bare-return case is deliberately strict: only a return whose argument
/// IS the `ctx` identifier counts. A return that merely *contains* `ctx` in a
/// larger expression — `return <div>{ctx.data}</div>` or `return ctx ? a : b`
/// — does not tear the context down and must not suppress the finding. The
/// `.revert()`/`.kill()` arm already covers `return () => ctx.revert()`, since
/// the reference's ancestor chain includes that member call.
fn reference_is_revert_or_return(
    nodes: &oxc_semantic::AstNodes<'_>,
    node_id: oxc_semantic::NodeId,
) -> bool {
    use oxc_ast::AstKind;

    // The bare-return case requires the reference's *immediate* parent to be a
    // ReturnStatement (so the returned expression is exactly `ctx`).
    if matches!(nodes.parent_kind(node_id), AstKind::ReturnStatement(_)) {
        return true;
    }

    let member_id = nodes.parent_id(node_id);
    if let AstKind::StaticMemberExpression(member) = nodes.kind(member_id)
        && matches!(member.property.name.as_str(), "revert" | "kill")
    {
        let call_id = nodes.parent_id(member_id);
        if let AstKind::CallExpression(call) = nodes.kind(call_id)
            && call.callee.span() == member.span
        {
            return true;
        }
    }
    false
}

/// Apply a callback to every [`Expression`] reachable from a statement,
/// shallowly enough for our rules (we only need expressions in common
/// statement positions and nested calls/objects). This is intentionally
/// pragmatic rather than a full visitor.
fn for_each_expression_in_statement<'a>(
    statement: &'a Statement<'a>,
    callback: &mut dyn FnMut(&'a Expression<'a>),
) {
    match statement {
        Statement::ExpressionStatement(expression_statement) => {
            walk_expression(&expression_statement.expression, callback);
        }
        Statement::ReturnStatement(return_statement) => {
            if let Some(argument) = &return_statement.argument {
                walk_expression(argument, callback);
            }
        }
        Statement::VariableDeclaration(declaration) => {
            for declarator in &declaration.declarations {
                if let Some(init) = &declarator.init {
                    walk_expression(init, callback);
                }
            }
        }
        Statement::BlockStatement(block) => {
            for inner in &block.body {
                for_each_expression_in_statement(inner, callback);
            }
        }
        Statement::IfStatement(if_statement) => {
            walk_expression(&if_statement.test, callback);
            for_each_expression_in_statement(&if_statement.consequent, callback);
            if let Some(alternate) = &if_statement.alternate {
                for_each_expression_in_statement(alternate, callback);
            }
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(declaration) = &export.declaration {
                for_each_expression_in_declaration(declaration, callback);
            }
        }
        Statement::ExportDefaultDeclaration(export) => {
            if let Some(expression) = export.declaration.as_expression() {
                walk_expression(expression, callback);
            }
        }
        // Loops: recurse into the loop body so tweens inside them are seen.
        Statement::ForStatement(for_statement) => {
            for_each_expression_in_statement(&for_statement.body, callback);
        }
        Statement::ForInStatement(for_in) => {
            for_each_expression_in_statement(&for_in.body, callback);
        }
        Statement::ForOfStatement(for_of) => {
            for_each_expression_in_statement(&for_of.body, callback);
        }
        Statement::WhileStatement(while_statement) => {
            for_each_expression_in_statement(&while_statement.body, callback);
        }
        Statement::DoWhileStatement(do_while) => {
            for_each_expression_in_statement(&do_while.body, callback);
        }
        // Switch: recurse into every case's consequent statements.
        Statement::SwitchStatement(switch_statement) => {
            for case in &switch_statement.cases {
                for inner in &case.consequent {
                    for_each_expression_in_statement(inner, callback);
                }
            }
        }
        // Try/catch/finally: recurse into the block, handler body, finalizer.
        Statement::TryStatement(try_statement) => {
            for inner in &try_statement.block.body {
                for_each_expression_in_statement(inner, callback);
            }
            if let Some(handler) = &try_statement.handler {
                for inner in &handler.body.body {
                    for_each_expression_in_statement(inner, callback);
                }
            }
            if let Some(finalizer) = &try_statement.finalizer {
                for inner in &finalizer.body {
                    for_each_expression_in_statement(inner, callback);
                }
            }
        }
        Statement::LabeledStatement(labeled) => {
            for_each_expression_in_statement(&labeled.body, callback);
        }
        _ => {}
    }
}

/// Walk expressions inside a declaration statement (function/variable).
fn for_each_expression_in_declaration<'a>(
    declaration: &'a oxc_ast::ast::Declaration<'a>,
    callback: &mut dyn FnMut(&'a Expression<'a>),
) {
    use oxc_ast::ast::Declaration;
    match declaration {
        Declaration::VariableDeclaration(variable) => {
            for declarator in &variable.declarations {
                if let Some(init) = &declarator.init {
                    walk_expression(init, callback);
                }
            }
        }
        Declaration::FunctionDeclaration(function) => {
            if let Some(body) = &function.body {
                for statement in &body.statements {
                    for_each_expression_in_statement(statement, callback);
                }
            }
        }
        _ => {}
    }
}

/// Recursively visit expressions for the surface scan and selector scan. Covers
/// the shapes our rules care about (calls, members, objects, functions).
fn walk_expression<'a>(
    expression: &'a Expression<'a>,
    callback: &mut dyn FnMut(&'a Expression<'a>),
) {
    callback(expression);
    match expression.without_parentheses() {
        Expression::CallExpression(call) => {
            walk_expression(&call.callee, callback);
            for argument in &call.arguments {
                if let Some(inner) = argument_expression(argument) {
                    walk_expression(inner, callback);
                }
            }
        }
        Expression::StaticMemberExpression(member) => {
            walk_expression(&member.object, callback);
        }
        Expression::ObjectExpression(object) => {
            for property in &object.properties {
                if let ObjectPropertyKind::ObjectProperty(inner) = property {
                    walk_expression(&inner.value, callback);
                }
            }
        }
        Expression::ArrowFunctionExpression(arrow) => {
            for statement in &arrow.body.statements {
                for_each_expression_in_statement(statement, callback);
            }
        }
        Expression::FunctionExpression(function) => {
            if let Some(body) = &function.body {
                for statement in &body.statements {
                    for_each_expression_in_statement(statement, callback);
                }
            }
        }
        Expression::AwaitExpression(await_expression) => {
            walk_expression(&await_expression.argument, callback);
        }
        Expression::LogicalExpression(logical) => {
            walk_expression(&logical.left, callback);
            walk_expression(&logical.right, callback);
        }
        Expression::ConditionalExpression(conditional) => {
            walk_expression(&conditional.test, callback);
            walk_expression(&conditional.consequent, callback);
            walk_expression(&conditional.alternate, callback);
        }
        _ => {}
    }
}
