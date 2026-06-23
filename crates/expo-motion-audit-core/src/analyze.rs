//! AST + semantic analysis engine for Reanimated 4 / Worklets source.
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
//!
//! Heuristic limitations: several rules in this engine are deliberately
//! file-scoped heuristics rather than precise data-flow analyses. They detect
//! the *absence* of an expected token anywhere in the file (for example
//! `cancelAnimation` or a reduced-motion reference). This trades a small false
//! positive/negative rate for a stable, dependency-light static check that runs
//! without type information. Each such rule documents its specific limitation.

use std::collections::{BTreeMap, BTreeSet};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, AssignmentTarget, CallExpression, Expression, FunctionBody,
    ImportDeclarationSpecifier, JSXAttributeName, ObjectExpression, ObjectPropertyKind, Program,
    PropertyKey, Statement,
};
use oxc_parser::Parser;
use oxc_semantic::{AstNode, Semantic, SemanticBuilder};
use oxc_span::{GetSpan, SourceType, Span};

use crate::rules::ids;
use crate::source::LineIndex;
use crate::types::{Category, Confidence, Finding, Severity};

/// The module Reanimated ships from.
const REANIMATED_MODULE: &str = "react-native-reanimated";
/// The module the modern worklets/threading helpers ship from.
const WORKLETS_MODULE: &str = "react-native-worklets";

/// Animation factories that create or drive shared values.
const WITH_ANIMATIONS: &[&str] = &[
    "withTiming",
    "withSpring",
    "withDecay",
    "withDelay",
    "withRepeat",
    "withSequence",
    "withClamp",
];

/// Hooks whose first function argument is auto-workletized by the babel plugin
/// when written as an inline arrow.
const ANIMATED_HOOKS: &[&str] = &[
    "useAnimatedStyle",
    "useDerivedValue",
    "useAnimatedReaction",
    "useAnimatedProps",
    "useAnimatedScrollHandler",
];

/// Gesture callback methods that receive a worklet and run on the UI thread.
const GESTURE_WORKLET_METHODS: &[&str] = &[
    "onStart",
    "onUpdate",
    "onChange",
    "onEnd",
    "onBegin",
    "onFinalize",
    "onTouchesDown",
    "onTouchesMove",
    "onTouchesUp",
];

/// Gesture callbacks (and per-frame handlers) where bridging back to the JS
/// thread on every invocation is a performance hazard.
const HOT_PATH_METHODS: &[&str] = &["onUpdate", "onChange"];

/// Reduced-motion identifiers the accessibility/layout rules look for.
const REDUCED_MOTION_TOKENS: &[&str] = &[
    "useReducedMotion",
    "ReduceMotion",
    "AccessibilityInfo",
    "isReduceMotionEnabled",
];

/// Layout properties that should be animated via transforms instead.
const LAYOUT_PROPS: &[&str] = &[
    "width",
    "height",
    "top",
    "left",
    "right",
    "bottom",
    "margin",
    "marginTop",
    "marginBottom",
    "marginLeft",
    "marginRight",
    "marginHorizontal",
    "marginVertical",
];

/// File-scoped facts gathered in a single pre-pass and shared by heuristic
/// rules that need a whole-file view (imports, reduced-motion tokens, shared
/// value bindings).
#[derive(Default)]
struct FileFacts {
    /// Local bindings imported from `react-native-reanimated`, keyed by local
    /// name to the canonical imported name.
    reanimated_imports: BTreeMap<String, String>,
    /// The file references a reduced-motion API anywhere (token-level).
    has_reduced_motion_ref: bool,
    /// The file references `cancelAnimation` anywhere (token-level).
    has_cancel_animation_ref: bool,
    /// Local binding names initialized from `useSharedValue(...)`.
    shared_value_bindings: BTreeSet<String>,
    /// The file animates with `with*` factories or uses `entering=`/`exiting=`.
    uses_reanimated_animation: bool,
    /// A shared value is driven by a `with*` factory somewhere in the file.
    animates_shared_value: bool,
}

/// Parse and analyze a single source string, returning owned findings.
///
/// `relative_path` is used verbatim in finding output. `source_type` selects
/// the oxc grammar.
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
    let facts = collect_file_facts(source, &semantic);

    let mut findings = Vec::new();
    let mut emit = |id: &str,
                    severity: Severity,
                    confidence: Confidence,
                    span: Span,
                    message: String,
                    suggestion: &str| {
        let descriptor = crate::rules::descriptor(id);
        let category = descriptor.map_or(Category::ReanimatedCore, |rule| rule.category);
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
        check_node(node, &semantic, &facts, &mut emit);
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
    findings.dedup();
    findings
}

/// Pre-pass: gather whole-file facts used by several rules.
fn collect_file_facts<'a>(source: &str, semantic: &Semantic<'a>) -> FileFacts {
    use oxc_ast::AstKind;

    let mut facts = FileFacts::default();

    for node in semantic.nodes() {
        match node.kind() {
            AstKind::ImportDeclaration(import) => {
                record_imports(import, &mut facts);
            }
            AstKind::IdentifierReference(identifier) => {
                let name = identifier.name.as_str();
                if REDUCED_MOTION_TOKENS.contains(&name) {
                    facts.has_reduced_motion_ref = true;
                }
                if name == "cancelAnimation" {
                    facts.has_cancel_animation_ref = true;
                }
            }
            AstKind::CallExpression(call) => {
                if let Expression::Identifier(callee) = call.callee.without_parentheses() {
                    let name = callee.name.as_str();
                    if WITH_ANIMATIONS.contains(&name) {
                        facts.uses_reanimated_animation = true;
                    }
                }
            }
            AstKind::VariableDeclarator(declarator) => {
                if let Some(identifier) = declarator.id.get_binding_identifier()
                    && let Some(init) = &declarator.init
                    && expression_is_call_to(init, "useSharedValue")
                {
                    facts
                        .shared_value_bindings
                        .insert(identifier.name.as_str().to_string());
                }
            }
            AstKind::JSXAttribute(attribute) => {
                if let Some(name) = jsx_attribute_name(&attribute.name)
                    && matches!(name, "entering" | "exiting" | "layout")
                {
                    facts.uses_reanimated_animation = true;
                }
            }
            AstKind::AssignmentExpression(assignment)
                if assignment_drives_shared_value(assignment) =>
            {
                facts.animates_shared_value = true;
            }
            _ => {}
        }
    }

    // The reduced-motion / cancelAnimation token scan above only sees
    // `IdentifierReference` nodes; member-property names (`ReduceMotion.System`)
    // surface as the object identifier, which is covered. As a belt-and-braces
    // fallback for tokens that may only appear in import specifiers or JSX, also
    // scan the raw source for the literal tokens. This keeps the heuristic
    // robust without a second AST pass.
    if !facts.has_cancel_animation_ref && source.contains("cancelAnimation") {
        facts.has_cancel_animation_ref = true;
    }
    if !facts.has_reduced_motion_ref
        && REDUCED_MOTION_TOKENS
            .iter()
            .any(|token| source.contains(token))
    {
        facts.has_reduced_motion_ref = true;
    }

    facts
}

/// Record import bindings from Reanimated and Worklets modules.
fn record_imports(import: &oxc_ast::ast::ImportDeclaration<'_>, facts: &mut FileFacts) {
    if import.import_kind.is_type() {
        return;
    }
    let source = import.source.value.as_str();
    if source != REANIMATED_MODULE && source != WORKLETS_MODULE {
        return;
    }
    let Some(specifiers) = &import.specifiers else {
        return;
    };
    for specifier in specifiers {
        if let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier
            && named.import_kind.is_value()
        {
            let imported = named.imported.name();
            facts.reanimated_imports.insert(
                named.local.name.as_str().to_string(),
                imported.as_str().to_string(),
            );
        }
    }
}

/// Per-node rule dispatch.
fn check_node<'a, F>(node: &AstNode<'a>, semantic: &Semantic<'a>, facts: &FileFacts, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    use oxc_ast::AstKind;

    match node.kind() {
        AstKind::CallExpression(call) => {
            check_call(call, semantic, facts, emit);
        }
        // Rule 2: reassigning a useSharedValue binding directly (`sv = x`).
        AstKind::AssignmentExpression(assignment) => {
            check_shared_value_reassign(assignment, semantic, facts, emit);
        }
        // Rule 4: reading/writing a resolved shared value's `.value` on JS.
        AstKind::StaticMemberExpression(member) => {
            check_value_access_on_js(member, semantic, node.id(), facts, emit);
        }
        _ => {}
    }
}

/// Call-expression rules.
fn check_call<'a, F>(
    call: &CallExpression<'a>,
    semantic: &Semantic<'a>,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    // Rule 1: animated style returning/animating layout props.
    check_layout_prop_animation(call, facts, emit);

    // Rule 3: deprecated runOnJS/runOnUI from react-native-reanimated.
    check_deprecated_run_on(call, facts, emit);

    // Rule 6: extracted named function missing a 'worklet' directive.
    check_missing_worklet(call, semantic, facts, emit);

    // Rule 5: bridge call inside a gesture hot-path callback.
    check_bridge_in_hot_path(call, semantic, emit);

    // Rule 7: withRepeat(anim, -1, ...) in a file with no reduced-motion ref.
    check_infinite_repeat(call, facts, emit);
}

/// Rule 1: an animated style (the return object of `useAnimatedStyle`, or any
/// object literal passed to it) animates layout props. We detect the
/// `useAnimatedStyle(() => ({ ... }))` shape and inspect the returned object.
fn check_layout_prop_animation<F>(call: &CallExpression<'_>, _facts: &FileFacts, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    if callee_identifier(call) != Some("useAnimatedStyle") {
        return;
    }
    let Some(first) = call.arguments.first().and_then(argument_expression) else {
        return;
    };
    let Some(object) = arrow_or_function_return_object(first) else {
        return;
    };
    if let Some(span) = object_animates_layout_prop(object) {
        emit(
            ids::REANIMATED_CORE_LAYOUT_PROP_ANIMATION,
            Severity::Medium,
            Confidence::Medium,
            span,
            "Animated style animates a layout property, which forces native layout work."
                .to_string(),
            "Animate transforms (translateX/translateY/scale) instead of width/height/top/left/margin.",
        );
    }
}

/// Rule 2: `sv = x` where `sv` resolves to a `useSharedValue(...)` binding.
///
/// Uses oxc_semantic to resolve the assignment-target identifier to its symbol
/// and confirms the symbol's declaration is initialized from `useSharedValue`.
/// `sv.value = x` is a member-target assignment and is correctly NOT flagged.
fn check_shared_value_reassign<'a, F>(
    assignment: &oxc_ast::ast::AssignmentExpression<'a>,
    semantic: &Semantic<'a>,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left else {
        return;
    };
    // Fast path: the binding name was recorded as a shared value in the pre-pass.
    let name = target.name.as_str();
    let is_shared_value = facts.shared_value_bindings.contains(name)
        || identifier_resolves_to_shared_value(target, semantic);
    if !is_shared_value {
        return;
    }
    emit(
        ids::REANIMATED_CORE_SHARED_VALUE_REASSIGN,
        Severity::High,
        Confidence::High,
        assignment.span,
        format!("`{name}` is a shared value but is reassigned directly instead of via `.value`."),
        "Write `sv.value = ...`; reassigning the binding replaces the shared value object.",
    );
}

/// Rule 3: `runOnJS`/`runOnUI` used (deprecated in Reanimated 4).
fn check_deprecated_run_on<F>(call: &CallExpression<'_>, facts: &FileFacts, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let Some(name) = callee_identifier(call) else {
        return;
    };
    // Resolve through an alias imported from react-native-reanimated, and accept
    // the canonical names directly (covers the common unaliased import).
    let canonical = facts
        .reanimated_imports
        .get(name)
        .map(String::as_str)
        .unwrap_or(name);
    let replacement = match canonical {
        "runOnJS" => "scheduleOnRN",
        "runOnUI" => "scheduleOnUI",
        _ => return,
    };
    emit(
        ids::WORKLETS_THREADING_DEPRECATED_RUN_ON,
        Severity::High,
        Confidence::High,
        call.span,
        format!("`{canonical}` is deprecated in Reanimated 4."),
        match replacement {
            "scheduleOnRN" => "Use `scheduleOnRN` from `react-native-worklets`.",
            _ => "Use `scheduleOnUI` from `react-native-worklets`.",
        },
    );
}

/// Rule 4: a resolved shared value's `.value` is read/written on the JS thread.
///
/// "On the JS thread" means: at module scope, or inside a component render body
/// — i.e. NOT inside any worklet (a function with a `'worklet'` directive), an
/// animated hook callback, a gesture callback, an event handler, or a
/// `useEffect`. We climb the node's ancestor chain; if we reach the program
/// root without passing through one of those UI-thread/effect contexts, the
/// access runs on JS and is flagged.
///
/// Limitation: this does not model functions called indirectly. A `.value`
/// access inside a plain helper function that is itself only ever called from a
/// worklet is still flagged (medium confidence reflects this).
fn check_value_access_on_js<'a, F>(
    member: &oxc_ast::ast::StaticMemberExpression<'a>,
    semantic: &Semantic<'a>,
    node_id: oxc_semantic::NodeId,
    facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    if member.property.name.as_str() != "value" {
        return;
    }
    let Expression::Identifier(object) = member.object.without_parentheses() else {
        return;
    };
    let name = object.name.as_str();
    let is_shared_value = facts.shared_value_bindings.contains(name)
        || identifier_resolves_to_shared_value(object, semantic);
    if !is_shared_value {
        return;
    }
    if access_runs_on_ui_or_effect(semantic, node_id) {
        return;
    }
    emit(
        ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS,
        Severity::High,
        Confidence::Medium,
        member.span,
        format!(
            "`{name}.value` is read/written on the JS thread (outside any worklet, animated hook, or effect)."
        ),
        "Access shared values inside a worklet/animated hook, or read them in useEffect/useDerivedValue.",
    );
}

/// Rule 5: a JS-bridge call (`scheduleOnRN`/`runOnJS`) inside a gesture
/// `onUpdate`/`onChange` (a per-frame hot path).
fn check_bridge_in_hot_path<'a, F>(call: &CallExpression<'a>, semantic: &Semantic<'a>, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let Some(name) = callee_identifier(call) else {
        return;
    };
    if !matches!(name, "scheduleOnRN" | "runOnJS") {
        return;
    }
    let Some(call_node_id) = find_node_id_for_span(
        semantic,
        call.span,
        |kind| matches!(kind, oxc_ast::AstKind::CallExpression(inner) if inner.span == call.span),
    ) else {
        return;
    };
    if enclosing_hot_path_method(semantic, call_node_id).is_some() {
        emit(
            ids::WORKLETS_THREADING_BRIDGE_IN_HOT_PATH,
            Severity::Medium,
            Confidence::Medium,
            call.span,
            format!("`{name}` runs on every frame inside a gesture onUpdate/onChange callback."),
            "Throttle the bridge, or move per-frame work into the worklet and bridge only on end/state change.",
        );
    }
}

/// Rule 6: a NON-arrow named function passed to an animated hook or gesture
/// callback that lacks a `'worklet'` directive.
///
/// The babel worklets plugin auto-workletizes inline arrows in these positions,
/// so this rule deliberately targets *extracted named functions* (function
/// expressions with an identifier, e.g. `useDerivedValue(function compute() {
/// ... })`) which are NOT auto-workletized. Confidence is medium because a
/// project may workletize via a wrapper the static check cannot see.
fn check_missing_worklet<'a, F>(
    call: &CallExpression<'a>,
    _semantic: &Semantic<'a>,
    _facts: &FileFacts,
    emit: &mut F,
) where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    let is_target = callee_identifier(call)
        .map(|name| ANIMATED_HOOKS.contains(&name))
        .unwrap_or(false)
        || callee_is_gesture_worklet_method(call);
    if !is_target {
        return;
    }
    let Some(first) = call.arguments.first().and_then(argument_expression) else {
        return;
    };
    // Inline arrows are auto-workletized; only a named function expression is a
    // concern.
    if let Expression::FunctionExpression(function) = first.without_parentheses()
        && function.id.is_some()
        && let Some(body) = &function.body
        && !function_body_has_worklet_directive(body)
    {
        let label = function
            .id
            .as_ref()
            .map(|id| id.name.as_str())
            .unwrap_or("function");
        emit(
            ids::WORKLETS_THREADING_MISSING_WORKLET,
            Severity::Medium,
            Confidence::Medium,
            function.span,
            format!(
                "Named function `{label}` passed to an animated hook/gesture callback lacks a 'worklet' directive (the babel plugin only auto-workletizes inline arrows here)."
            ),
            "Add `'worklet';` as the first statement of the function, or inline it as an arrow.",
        );
    }
}

/// Rule 7: `withRepeat(anim, -1, ...)` in a file with no reduced-motion ref.
fn check_infinite_repeat<F>(call: &CallExpression<'_>, facts: &FileFacts, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    if callee_identifier(call) != Some("withRepeat") {
        return;
    }
    let Some(second) = call.arguments.get(1) else {
        return;
    };
    if !argument_is_negative_one(second) {
        return;
    }
    if facts.has_reduced_motion_ref {
        return;
    }
    emit(
        ids::LAYOUT_INFINITE_REPEAT_NO_REDUCED_MOTION,
        Severity::Medium,
        Confidence::Medium,
        call.span,
        "Infinite `withRepeat(anim, -1, ...)` in a file with no reduced-motion guard.".to_string(),
        "Gate looping animations behind useReducedMotion()/ReduceMotion so motion-sensitive users opt out.",
    );
}

/// File-level rules: missing reduced-motion (rule 8) and missing
/// cancelAnimation (rule 9).
fn check_file_level(
    program: &Program<'_>,
    relative_path: &str,
    facts: &FileFacts,
    line_index: &LineIndex,
    findings: &mut Vec<Finding>,
) {
    let span = program.span;
    let (line, column) = line_index.line_col(span.start);

    // Rule 8: the file animates with Reanimated but never references a
    // reduced-motion API. File-scoped heuristic: it cannot tell whether the
    // reduced-motion handling lives in a parent component, so confidence is
    // medium.
    if facts.uses_reanimated_animation && !facts.has_reduced_motion_ref {
        findings.push(Finding {
            id: ids::ACCESSIBILITY_MISSING_REDUCED_MOTION.to_string(),
            category: Category::Accessibility,
            severity: Severity::Medium,
            confidence: Confidence::Medium,
            file: relative_path.to_string(),
            line,
            column,
            message: "File uses Reanimated animations but never references a reduced-motion API."
                .to_string(),
            suggestion:
                "Respect useReducedMotion()/ReduceMotion (or AccessibilityInfo) before animating."
                    .to_string(),
        });
    }

    // Rule 9: a shared value is animated with `with*` but the file never
    // references `cancelAnimation`. File-scoped heuristic: it cannot prove the
    // animation outlives the component, so confidence is medium.
    if facts.animates_shared_value && !facts.has_cancel_animation_ref {
        findings.push(Finding {
            id: ids::LIFECYCLE_MISSING_CANCEL_ANIMATION.to_string(),
            category: Category::Lifecycle,
            severity: Severity::Medium,
            confidence: Confidence::Medium,
            file: relative_path.to_string(),
            line,
            column,
            message:
                "Shared value animated with `with*(...)` but the file never calls cancelAnimation."
                    .to_string(),
            suggestion:
                "Call cancelAnimation(sv) on unmount (e.g. in a useEffect cleanup) to avoid leaks."
                    .to_string(),
        });
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the static property name of an object key, if it is a plain
/// identifier or string key.
fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

/// The callee identifier name of a bare call `foo(...)`, if any.
fn callee_identifier<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match call.callee.without_parentheses() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

/// Whether the call's callee is `<gesture>.onUpdate(...)` style — a static
/// member call whose property is a gesture worklet method.
fn callee_is_gesture_worklet_method(call: &CallExpression<'_>) -> bool {
    matches!(
        call.callee.without_parentheses(),
        Expression::StaticMemberExpression(member)
            if GESTURE_WORKLET_METHODS.contains(&member.property.name.as_str())
    )
}

/// Get the inner [`Expression`] of an [`Argument`], skipping spreads.
fn argument_expression<'a>(argument: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
    argument.as_expression()
}

/// Whether an expression is a call to a named function `name(...)`.
fn expression_is_call_to(expression: &Expression<'_>, name: &str) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::CallExpression(call) if callee_identifier(call) == Some(name)
    )
}

/// Whether an argument is the numeric literal `-1` (a unary negation of `1`).
fn argument_is_negative_one(argument: &Argument<'_>) -> bool {
    let Some(expression) = argument_expression(argument) else {
        return false;
    };
    match expression.without_parentheses() {
        Expression::UnaryExpression(unary) if unary.operator.as_str() == "-" => {
            matches!(
                unary.argument.without_parentheses(),
                Expression::NumericLiteral(number) if number.value == 1.0
            )
        }
        _ => false,
    }
}

/// If a callback expression is `() => ({ ... })` or `() => { return { ... } }`
/// or `function () { return { ... } }`, return the returned object literal.
fn arrow_or_function_return_object<'a>(
    callback: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match callback.without_parentheses() {
        Expression::ArrowFunctionExpression(arrow) => {
            if let Some(expression) = arrow.get_expression() {
                object_of_expression(expression)
            } else {
                first_returned_object(&arrow.body)
            }
        }
        Expression::FunctionExpression(function) => {
            let body = function.body.as_ref()?;
            first_returned_object(body)
        }
        _ => None,
    }
}

/// The object literal of an expression (after unwrapping parentheses).
fn object_of_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

/// The object literal returned by the first `return { ... }` in a function body.
fn first_returned_object<'a>(body: &'a FunctionBody<'a>) -> Option<&'a ObjectExpression<'a>> {
    for statement in &body.statements {
        if let Statement::ReturnStatement(return_statement) = statement
            && let Some(argument) = &return_statement.argument
            && let Some(object) = object_of_expression(argument)
        {
            return Some(object);
        }
    }
    None
}

/// If an object literal animates a layout property, return that property's span.
fn object_animates_layout_prop(object: &ObjectExpression<'_>) -> Option<Span> {
    // `transform: [{ translateX: ... }]` is fine; only flag direct layout keys.
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

/// Whether a function body starts with a `'worklet'` directive.
fn function_body_has_worklet_directive(body: &FunctionBody<'_>) -> bool {
    body.directives
        .iter()
        .any(|directive| directive.expression.value.as_str() == "worklet")
}

/// Static attribute name of a JSX attribute, if it is a plain identifier.
fn jsx_attribute_name<'a>(name: &'a JSXAttributeName<'a>) -> Option<&'a str> {
    match name {
        JSXAttributeName::Identifier(identifier) => Some(identifier.name.as_str()),
        JSXAttributeName::NamespacedName(_) => None,
    }
}

/// Whether an assignment writes to `<sharedValue>.value`, signaling that a
/// shared value is being driven. We accept any `something.value = with*(...)`
/// or `something.value = <expr>` where the RHS is a `with*` factory call, which
/// is the common "animate this shared value" shape.
fn assignment_drives_shared_value(assignment: &oxc_ast::ast::AssignmentExpression<'_>) -> bool {
    let is_value_target = matches!(
        &assignment.left,
        AssignmentTarget::StaticMemberExpression(member)
            if member.property.name.as_str() == "value"
    );
    if !is_value_target {
        return false;
    }
    expression_is_with_animation(&assignment.right)
}

/// Whether an expression is a `with*` animation factory call.
fn expression_is_with_animation(expression: &Expression<'_>) -> bool {
    match expression.without_parentheses() {
        Expression::CallExpression(call) => callee_identifier(call)
            .map(|name| WITH_ANIMATIONS.contains(&name))
            .unwrap_or(false),
        _ => false,
    }
}

/// Resolve an identifier reference to its symbol and check whether the symbol's
/// declaration is a `VariableDeclarator` initialized from `useSharedValue(...)`.
fn identifier_resolves_to_shared_value(
    identifier: &oxc_ast::ast::IdentifierReference<'_>,
    semantic: &Semantic<'_>,
) -> bool {
    use oxc_ast::AstKind;

    let scoping = semantic.scoping();
    let Some(reference_id) = identifier.reference_id.get() else {
        return false;
    };
    let Some(symbol_id) = scoping.get_reference(reference_id).symbol_id() else {
        return false;
    };
    let declaration_node = scoping.symbol_declaration(symbol_id);
    if let AstKind::VariableDeclarator(declarator) = semantic.nodes().kind(declaration_node)
        && let Some(init) = &declarator.init
    {
        return expression_is_call_to(init, "useSharedValue");
    }
    false
}

/// Whether the node at `node_id` runs on the UI thread or inside an effect.
///
/// We climb ancestors; the access is considered NOT on the JS thread if any
/// ancestor is a worklet (function body with a `'worklet'` directive), an
/// animated-hook/gesture callback, an event handler, or a `useEffect`/
/// `useLayoutEffect`/`useAnimatedReaction` call's callback argument.
///
/// JSX event handlers (`onPress={() => { sv.value = ... }}`) run at event time
/// on the JS thread, so writing/reading a shared value there is fine. We treat
/// an access as off the render path when it is inside a function that is the
/// value of a JSX event-handler attribute. A bare expression in a JSX attribute
/// with no intervening function (e.g. `style={{ width: sv.value }}`) is read
/// during render and stays on the JS render path.
fn access_runs_on_ui_or_effect(semantic: &Semantic<'_>, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = node_id;
    // Whether we have climbed through a function boundary on the way up. A JSX
    // event handler only exempts the access if a function intervenes.
    let mut passed_through_function = false;
    loop {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            // Reached the root without finding a UI-thread context.
            return false;
        }
        match nodes.kind(parent_id) {
            // A function carrying a `'worklet'` directive runs on the UI thread.
            AstKind::Function(function) => {
                if let Some(body) = &function.body
                    && function_body_has_worklet_directive(body)
                {
                    return true;
                }
                passed_through_function = true;
            }
            AstKind::ArrowFunctionExpression(_) => {
                passed_through_function = true;
            }
            AstKind::FunctionBody(body) => {
                if function_body_has_worklet_directive(body) {
                    return true;
                }
            }
            // A function/arrow passed as an argument to an animated hook, gesture
            // callback, event handler, or effect runs off the JS render path.
            AstKind::CallExpression(call) if call_is_ui_or_effect_context(call) => {
                return true;
            }
            // A JSX event-handler prop (`onPress={() => { ... }}`) runs at event
            // time on the JS thread, but only when a function intervenes.
            AstKind::JSXAttribute(attribute)
                if passed_through_function && jsx_attribute_is_event_handler(attribute) =>
            {
                return true;
            }
            _ => {}
        }
        current = parent_id;
    }
}

/// Whether a JSX attribute is an event handler: its name starts with `on`
/// followed by an uppercase letter (e.g. `onPress`, `onChange`, `onScroll`,
/// `onLongPress`).
fn jsx_attribute_is_event_handler(attribute: &oxc_ast::ast::JSXAttribute<'_>) -> bool {
    let Some(name) = jsx_attribute_name(&attribute.name) else {
        return false;
    };
    let mut chars = name.chars();
    chars.next() == Some('o')
        && chars.next() == Some('n')
        && chars.next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Whether a call expression establishes a UI-thread or effect context for its
/// function argument: animated hooks, gesture worklet methods, `useEffect`/
/// `useLayoutEffect`, or an explicit `runOnUI`/`scheduleOnUI` wrapper.
fn call_is_ui_or_effect_context(call: &CallExpression<'_>) -> bool {
    if let Some(name) = callee_identifier(call)
        && (ANIMATED_HOOKS.contains(&name)
            || matches!(
                name,
                "useEffect" | "useLayoutEffect" | "runOnUI" | "scheduleOnUI"
            ))
    {
        return true;
    }
    callee_is_gesture_worklet_method(call)
}

/// If `node_id` is inside a gesture `onUpdate`/`onChange` callback, return its
/// node id (the enclosing call). Used by the hot-path rule.
fn enclosing_hot_path_method(
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
        if let AstKind::CallExpression(call) = nodes.kind(parent_id)
            && matches!(
                call.callee.without_parentheses(),
                Expression::StaticMemberExpression(member)
                    if HOT_PATH_METHODS.contains(&member.property.name.as_str())
            )
        {
            return Some(parent_id);
        }
        current = parent_id;
    }
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
