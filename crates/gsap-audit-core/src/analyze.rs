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
    "ScrollToPlugin",
    "TextPlugin",
];

const PLUGIN_VARS: &[(&str, &str)] = &[
    ("motionPath", "MotionPathPlugin"),
    ("drawSVG", "DrawSVGPlugin"),
    ("morphSVG", "MorphSVGPlugin"),
    ("text", "TextPlugin"),
    ("scrollTo", "ScrollToPlugin"),
    ("inertia", "InertiaPlugin"),
];

/// GSAP tween factory methods that take a vars object.
const TWEEN_METHODS: &[&str] = &["to", "from", "fromTo", "set"];

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
    /// The file has a top-of-file `"use client"` directive.
    has_use_client: bool,
    /// The file uses GSAP member access or a bare GSAP plugin identifier.
    uses_gsap_surface: bool,
    /// Local identifiers bound to the GSAP object.
    gsap_bindings: BTreeSet<String>,
    /// Local bindings imported from the skill's configured GSAP module pattern
    /// (`lib/gsap`), where registration is centralized before re-export.
    configured_gsap_imports: BTreeSet<String>,
    /// Identifiers initialized from `gsap.timeline(...)`.
    timeline_handles: BTreeSet<String>,
    /// Local import aliases for known plugins, keyed by local binding.
    plugin_aliases: BTreeMap<String, String>,
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
            }
            AstKind::VariableDeclarator(declarator) => {
                if let Some(identifier) = declarator.id.get_binding_identifier()
                    && let Some(init) = &declarator.init
                    && expression_is_gsap_timeline_call(init, &facts)
                {
                    facts
                        .timeline_handles
                        .insert(identifier.name.as_str().to_string());
                }
            }
            _ => {}
        }
    }

    facts
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
        let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier else {
            continue;
        };
        if named.import_kind.is_type() {
            continue;
        }
        let imported = named.imported.name();
        let imported_name = imported.as_str();
        if imported_name == "gsap" || KNOWN_PLUGINS.contains(&imported_name) {
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
            if KNOWN_PLUGINS.contains(&imported_name) {
                facts.plugin_aliases.insert(
                    named.local.name.as_str().to_string(),
                    imported_name.to_string(),
                );
            }
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
                let Some(plugin) = plugin_name_for_known_or_default(imported_name, source_plugin)
                else {
                    continue;
                };
                facts
                    .plugin_aliases
                    .insert(named.local.name.as_str().to_string(), plugin.to_string());
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                if let Some(plugin) = source_plugin {
                    facts
                        .plugin_aliases
                        .insert(default.local.name.as_str().to_string(), plugin.to_string());
                }
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(_) => {}
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
    !import_source_is_gsap_package(source)
        && !import_source_is_gsap_trial(source)
        && (source == "lib/gsap" || source.ends_with("/lib/gsap") || source == "./gsap")
}

fn plugin_name_from_import_source(source: &str) -> Option<&str> {
    let name = source.rsplit('/').next()?;
    KNOWN_PLUGINS.contains(&name).then_some(name)
}

fn plugin_name_for_identifier<'a>(name: &'a str, facts: &'a FileFacts) -> Option<&'a str> {
    if KNOWN_PLUGINS.contains(&name) {
        Some(name)
    } else {
        facts.plugin_aliases.get(name).map(String::as_str)
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
            check_call(call, semantic, facts, emit);
        }
        // Rule 2: GSDevTools referenced in non-test source. Skip TS type-only
        // positions (e.g. `let x: GSDevTools`), which are erased at build time.
        AstKind::IdentifierReference(identifier)
            if identifier.name.as_str() == "GSDevTools"
                && !is_test_or_fixture_path(relative_path)
                && !reference_is_ts_type_position(semantic, node.id()) =>
        {
            emit(
                ids::PLUGINS_GSDEVTOOLS_IN_SOURCE,
                Severity::Medium,
                Confidence::Medium,
                identifier.span,
                "GSDevTools referenced in source code.".to_string(),
                "GSDevTools is a dev-only tool; gate it behind a dev flag or remove it before shipping.",
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
            // Rules 3 & 4: markers / scrub+toggleActions only inside the GSAP
            // config (the vars object and its nested `scrollTrigger:` object).
            check_gsap_config_object(vars, emit);
        }
    }
    if let Some(vars) = gsap_timeline_vars_object(call, facts) {
        check_gsap_config_object(vars, emit);
    }

    // ScrollTrigger.create({...}): the argument is a ScrollTrigger config.
    if is_plugin_member_call(call, facts, "ScrollTrigger", "create")
        && let Some(config) = call.arguments.first().and_then(argument_expression)
        && let Expression::ObjectExpression(object) = config.without_parentheses()
    {
        check_gsap_config_object(object, emit);
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
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Path is under a Next.js App Router `app/` directory segment.
fn is_under_app(path: &str) -> bool {
    path.split(['/', '\\']).any(|segment| segment == "app")
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
                || facts.timeline_handles.contains(identifier.name.as_str())
        }
        Expression::CallExpression(call) => is_gsap_member_call(call, facts, "timeline"),
        _ => false,
    }
}

fn expression_is_gsap_timeline_call(expression: &Expression<'_>, facts: &FileFacts) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::CallExpression(call) if is_gsap_member_call(call, facts, "timeline")
    )
}

/// Return every vars object literal for a tween call.
///
/// `to`/`from`/`set(target, vars)` carry one vars object (the last argument).
/// `fromTo(target, fromVars, toVars)` carries two: layout-prop and config rules
/// must scan BOTH, since either object can set offending properties.
fn tween_vars_objects<'a>(
    call: &'a CallExpression<'a>,
    method: &str,
) -> Vec<&'a ObjectExpression<'a>> {
    let argument_count = call.arguments.len();
    let mut indices: Vec<usize> = Vec::new();
    match method {
        // fromTo(target, fromVars, toVars): the last two arguments.
        "fromTo" => {
            if let Some(to_index) = argument_count.checked_sub(1) {
                if let Some(from_index) = argument_count.checked_sub(2) {
                    indices.push(from_index);
                }
                indices.push(to_index);
            }
        }
        // to/from/set(target, vars): vars is the last argument.
        _ => {
            if let Some(index) = argument_count.checked_sub(1) {
                indices.push(index);
            }
        }
    }

    indices
        .into_iter()
        .filter_map(|index| {
            let argument = call.arguments.get(index)?;
            match argument_expression(argument).map(Expression::without_parentheses) {
                Some(Expression::ObjectExpression(object)) => Some(object.as_ref()),
                _ => None,
            }
        })
        .collect()
}

/// Run the GSAP/ScrollTrigger object-literal rules over a config object and its
/// nested `scrollTrigger:` config object.
///
/// This is the single entry point that gates Rules 3 (markers) and 4 (scrub +
/// toggleActions) to genuine GSAP context: a tween/timeline vars object, a
/// `scrollTrigger:` value, or a `ScrollTrigger.create(...)` argument. Unrelated
/// object literals elsewhere in the file are never scanned.
fn check_gsap_config_object<F>(object: &ObjectExpression<'_>, emit: &mut F)
where
    F: FnMut(&str, Severity, Confidence, Span, String, &str),
{
    check_object_literal(object, emit);

    // A `scrollTrigger:` property nests a ScrollTrigger config object that also
    // carries markers / scrub / toggleActions; scan it too.
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property_key_name(&property.key) == Some("scrollTrigger")
            && let Expression::ObjectExpression(nested) = property.value.without_parentheses()
        {
            check_object_literal(nested, emit);
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
        Expression::ObjectExpression(object) => object.properties.iter().any(|property| {
            matches!(
                property,
                ObjectPropertyKind::ObjectProperty(inner)
                    if property_key_name(&inner.key) == Some("scope")
            )
        }),
        // A dependency array (useGSAP's useEffect-style overload) is never a scope.
        Expression::ArrayExpression(_) => false,
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
        _ => {}
    }
}
