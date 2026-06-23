//! Config-file analysis: `babel.config.js` and `app.json`/`app.config.json`.
//!
//! Babel config is parsed as CommonJS with oxc. The export may be either a
//! direct `module.exports = { ... }` object or a function form
//! `module.exports = function (api) { return { ... }; }`; both are handled. The
//! analysis locates the `plugins` array. `babel-preset-expo` in the `presets`
//! array auto-includes `react-native-worklets/plugin`, so a config that relies
//! on the preset (with no explicit worklets plugin) is treated as correct and
//! the "missing" finding is suppressed; explicit ordering rules (deprecated
//! plugin, worklets-present-but-not-last) still fire.
//!
//! App config is parsed as JSON with serde_json. The static `app.json` /
//! `app.config.json` forms are analyzed directly; the dynamic
//! `app.config.js`/`.ts`/`.cjs`/`.mjs` forms fail the JSON parse and are
//! reported as an informational `config.unable-to-analyze` low finding because
//! they cannot be resolved statically.
//!
//! Limitation: babel config resolution is structural, not evaluative. A
//! `plugins` property that is not an inline array (assembled dynamically via a
//! variable, spread, conditional, or computed) cannot be resolved and yields an
//! informational low finding rather than a false high-severity one.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::rules::ids;
use crate::source::LineIndex;
use crate::types::{Category, Confidence, Finding, Severity};

/// The plugin string Reanimated 4 / Worklets requires (must be last).
const WORKLETS_PLUGIN: &str = "react-native-worklets/plugin";
/// The old, deprecated Reanimated babel plugin string.
const DEPRECATED_REANIMATED_PLUGIN: &str = "react-native-reanimated/plugin";
/// The Expo babel preset, which auto-includes the worklets plugin.
const BABEL_PRESET_EXPO: &str = "babel-preset-expo";

/// Analyze a `babel.config.js` (or `babel.config.cjs`) source string.
///
/// Emits, in order of priority:
/// - [`ids::CONFIG_DEPRECATED_REANIMATED_PLUGIN`] (high) when the old
///   `react-native-reanimated/plugin` appears in the plugins array.
/// - [`ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST`] (high) when the
///   worklets plugin is absent (and `babel-preset-expo` is not present to supply
///   it), or present but not the last element.
/// - [`ids::CONFIG_UNABLE_TO_ANALYZE`] (low) when the export is too dynamic to
///   resolve statically, including a `plugins` property that is present but not
///   an inline array.
#[must_use]
pub fn analyze_babel_config(relative_path: &str, source: &str) -> Vec<Finding> {
    let allocator = Allocator::default();
    // Babel config is CommonJS.
    let parser_return = Parser::new(&allocator, source, SourceType::cjs()).parse();
    let line_index = LineIndex::new(source);
    if parser_return.panicked {
        return vec![unable_to_analyze(
            relative_path,
            &line_index,
            "babel.config could not be parsed.",
        )];
    }
    let program = parser_return.program;

    let Some(config_object) = find_module_exports_object(&program) else {
        return vec![unable_to_analyze(
            relative_path,
            &line_index,
            "unable to statically analyze babel.config (dynamic or non-object export).",
        )];
    };

    // `babel-preset-expo` auto-includes `react-native-worklets/plugin`, so a
    // config that relies on the preset (with no explicit worklets plugin) is
    // correct and must not be flagged as missing.
    let has_expo_preset = has_babel_preset_expo(config_object);

    let Some(plugins) = object_property_array(config_object, "plugins") else {
        // `plugins` is either absent, or present but not an inline array.
        if object_has_property(config_object, "plugins") {
            // Present but dynamic (identifier, conditional, spread, ...): cannot
            // resolve statically, so this is informational, not a hard miss.
            return vec![unable_to_analyze(
                relative_path,
                &line_index,
                "babel.config `plugins` is not an inline array; it cannot be analyzed statically.",
            )];
        }
        // Truly absent. If babel-preset-expo is present it supplies the worklets
        // plugin, so there is nothing to report.
        if has_expo_preset {
            return Vec::new();
        }
        // No plugins array at all: the worklets plugin is definitionally missing.
        return vec![missing_or_not_last(
            relative_path,
            &line_index,
            config_object.span.start,
            "babel.config has no `plugins` array; the worklets plugin is required and must be last.",
        )];
    };

    let plugin_names = collect_plugin_names(plugins);
    let mut findings = Vec::new();

    // Rule 11: deprecated reanimated plugin present.
    if plugin_names
        .iter()
        .any(|name| name.as_deref() == Some(DEPRECATED_REANIMATED_PLUGIN))
    {
        let (line, column) = line_index.line_col(plugins.span.start);
        findings.push(Finding {
            id: ids::CONFIG_DEPRECATED_REANIMATED_PLUGIN.to_string(),
            category: Category::Config,
            severity: Severity::High,
            confidence: Confidence::High,
            file: relative_path.to_string(),
            line,
            column,
            message: format!(
                "babel.config uses the deprecated `{DEPRECATED_REANIMATED_PLUGIN}`."
            ),
            suggestion: format!(
                "Replace `{DEPRECATED_REANIMATED_PLUGIN}` with `{WORKLETS_PLUGIN}` (it must be last)."
            ),
        });
    }

    // Rule 10: worklets plugin missing or not last.
    let last_is_worklets = plugin_names
        .last()
        .map(|name| name.as_deref() == Some(WORKLETS_PLUGIN))
        .unwrap_or(false);
    let has_worklets = plugin_names
        .iter()
        .any(|name| name.as_deref() == Some(WORKLETS_PLUGIN));

    if !has_worklets {
        // The explicit worklets plugin is absent. babel-preset-expo supplies it,
        // so only flag "missing" when the expo preset is not present.
        if !has_expo_preset {
            findings.push(missing_or_not_last(
                relative_path,
                &line_index,
                plugins.span.start,
                &format!(
                    "babel.config is missing `{WORKLETS_PLUGIN}`; it is required and must be last."
                ),
            ));
        }
    } else if !last_is_worklets {
        findings.push(missing_or_not_last(
            relative_path,
            &line_index,
            plugins.span.start,
            &format!("`{WORKLETS_PLUGIN}` is present but not the last plugin."),
        ));
    }

    // If we could not statically resolve some entries AND the worklets plugin
    // was not provably the last, hint that analysis was partial.
    if findings.is_empty() && plugin_names.iter().any(Option::is_none) {
        findings.push(unable_to_analyze(
            relative_path,
            &line_index,
            "babel.config plugins array contains entries that could not be resolved statically.",
        ));
    }

    findings
}

/// Analyze an `app.json` / `app.config.json` source string.
///
/// `project_uses_reanimated` should be `true` when any scanned source file
/// imports `react-native-reanimated`. The New-Architecture rule only fires when
/// the project actually uses Reanimated, since Reanimated 4 requires the New
/// Architecture.
///
/// Emits [`ids::CONFIG_NEW_ARCH_DISABLED`] (medium) when `expo.newArchEnabled`
/// is `false` or absent and the project uses Reanimated.
#[must_use]
pub fn analyze_app_config(
    relative_path: &str,
    source: &str,
    project_uses_reanimated: bool,
) -> Vec<Finding> {
    let line_index = LineIndex::new(source);
    let Ok(value) = serde_json::from_str::<serde_json::Value>(source) else {
        return vec![unable_to_analyze(
            relative_path,
            &line_index,
            "app config is not valid JSON (a dynamic app.config.js/ts cannot be analyzed statically).",
        )];
    };

    if !project_uses_reanimated {
        return Vec::new();
    }

    let new_arch = value
        .get("expo")
        .and_then(|expo| expo.get("newArchEnabled"));
    let disabled = match new_arch {
        Some(serde_json::Value::Bool(true)) => false,
        // false, null, or absent all count as "not enabled".
        _ => true,
    };
    if !disabled {
        return Vec::new();
    }

    vec![Finding {
        id: ids::CONFIG_NEW_ARCH_DISABLED.to_string(),
        category: Category::Config,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        file: relative_path.to_string(),
        line: 1,
        column: 1,
        message: "expo.newArchEnabled is false or absent while the project uses Reanimated."
            .to_string(),
        suggestion:
            "Set `expo.newArchEnabled` to true; Reanimated 4 requires the New Architecture."
                .to_string(),
    }]
}

// ---------------------------------------------------------------------------
// Babel config helpers
// ---------------------------------------------------------------------------

/// Find the object literal exported via `module.exports = ...`.
///
/// Handles the direct object form and the function form
/// `module.exports = function (api) { return { ... }; }` (or an arrow), where
/// the returned object literal is the config.
fn find_module_exports_object<'a>(
    program: &'a oxc_ast::ast::Program<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in &program.body {
        let Statement::ExpressionStatement(expression_statement) = statement else {
            continue;
        };
        let Expression::AssignmentExpression(assignment) = &expression_statement.expression else {
            continue;
        };
        if !assignment_target_is_module_exports(&assignment.left) {
            continue;
        }
        return object_or_function_return_object(&assignment.right);
    }
    None
}

/// Whether an assignment target is `module.exports`.
fn assignment_target_is_module_exports(target: &oxc_ast::ast::AssignmentTarget<'_>) -> bool {
    use oxc_ast::ast::AssignmentTarget;
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return false;
    };
    if member.property.name.as_str() != "exports" {
        return false;
    }
    matches!(
        member.object.without_parentheses(),
        Expression::Identifier(identifier) if identifier.name.as_str() == "module"
    )
}

/// The config object from either a direct object literal or a function that
/// returns one.
fn object_or_function_return_object<'a>(
    expression: &'a Expression<'a>,
) -> Option<&'a ObjectExpression<'a>> {
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => Some(object),
        Expression::FunctionExpression(function) => {
            let body = function.body.as_ref()?;
            first_returned_object(&body.statements)
        }
        Expression::ArrowFunctionExpression(arrow) => {
            if let Some(inner) = arrow.get_expression() {
                match inner.without_parentheses() {
                    Expression::ObjectExpression(object) => Some(object),
                    _ => None,
                }
            } else {
                first_returned_object(&arrow.body.statements)
            }
        }
        _ => None,
    }
}

/// First object literal returned by a `return { ... }` in a statement list.
fn first_returned_object<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a ObjectExpression<'a>> {
    for statement in statements {
        if let Statement::ReturnStatement(return_statement) = statement
            && let Some(argument) = &return_statement.argument
            && let Expression::ObjectExpression(object) = argument.without_parentheses()
        {
            return Some(object);
        }
    }
    None
}

/// Return the array-expression value of an object property by key name.
fn object_property_array<'a>(
    object: &'a ObjectExpression<'a>,
    key_name: &str,
) -> Option<&'a oxc_ast::ast::ArrayExpression<'a>> {
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if property_key_name(&property.key) == Some(key_name)
            && let Expression::ArrayExpression(array) = property.value.without_parentheses()
        {
            return Some(array);
        }
    }
    None
}

/// Whether an object literal has a property with the given key name, regardless
/// of the property's value shape.
fn object_has_property(object: &ObjectExpression<'_>, key_name: &str) -> bool {
    object.properties.iter().any(|property| {
        matches!(
            property,
            ObjectPropertyKind::ObjectProperty(property)
                if property_key_name(&property.key) == Some(key_name)
        )
    })
}

/// Whether the config's `presets` array contains an entry resolving to
/// `babel-preset-expo`. Presets may be a bare string (`"babel-preset-expo"`) or
/// a tuple (`["babel-preset-expo", options]`); both resolve to the first
/// string. `babel-preset-expo` auto-includes `react-native-worklets/plugin`.
fn has_babel_preset_expo(object: &ObjectExpression<'_>) -> bool {
    let Some(presets) = object_property_array(object, "presets") else {
        return false;
    };
    collect_plugin_names(presets)
        .iter()
        .any(|name| name.as_deref() == Some(BABEL_PRESET_EXPO))
}

/// Collect the resolved string names of each plugins-array element, in order.
///
/// A plugin entry can be a bare string (`"foo"`) or a tuple
/// (`["foo", options]`); both resolve to the string `"foo"`. An entry that
/// cannot be resolved statically (an identifier, a call, a spread) resolves to
/// `None`, which the caller treats as "unknown".
fn collect_plugin_names(array: &oxc_ast::ast::ArrayExpression<'_>) -> Vec<Option<String>> {
    array
        .elements
        .iter()
        .map(|element| match element {
            oxc_ast::ast::ArrayExpressionElement::StringLiteral(string) => {
                Some(string.value.as_str().to_string())
            }
            oxc_ast::ast::ArrayExpressionElement::ArrayExpression(inner) => {
                // Tuple form `["plugin", options]`: resolve to the first string.
                match inner.elements.first() {
                    Some(oxc_ast::ast::ArrayExpressionElement::StringLiteral(string)) => {
                        Some(string.value.as_str().to_string())
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
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

/// Build a `config.worklets-plugin-missing-or-not-last` finding at an offset.
fn missing_or_not_last(
    relative_path: &str,
    line_index: &LineIndex,
    offset: u32,
    message: &str,
) -> Finding {
    let (line, column) = line_index.line_col(offset);
    Finding {
        id: ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST.to_string(),
        category: Category::Config,
        severity: Severity::High,
        confidence: Confidence::High,
        file: relative_path.to_string(),
        line,
        column,
        message: message.to_string(),
        suggestion: format!(
            "Add `{WORKLETS_PLUGIN}` as the LAST entry of the babel `plugins` array."
        ),
    }
}

/// Build a low-severity informational `config.unable-to-analyze` finding.
fn unable_to_analyze(relative_path: &str, line_index: &LineIndex, message: &str) -> Finding {
    let (line, column) = line_index.line_col(0);
    Finding {
        id: ids::CONFIG_UNABLE_TO_ANALYZE.to_string(),
        category: Category::Config,
        severity: Severity::Low,
        confidence: Confidence::Low,
        file: relative_path.to_string(),
        line,
        column,
        message: message.to_string(),
        suggestion: "Review this config manually; it is too dynamic for static analysis."
            .to_string(),
    }
}
