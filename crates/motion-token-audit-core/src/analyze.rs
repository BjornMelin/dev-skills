//! Token discovery plus CSS and JS/TS stack checks.

use std::collections::BTreeMap;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpression, CallExpression, Expression, IdentifierReference, JSXAttributeName,
    JSXElementName, JSXMemberExpressionObject, ObjectExpression, ObjectPropertyKind, PropertyKey,
};
use oxc_parser::Parser;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{GetSpan, SourceType, Span};

use crate::rules::{descriptor, ids};
use crate::source::LineIndex;
use crate::types::{Category, Confidence, Coverage, Finding, Severity};

const STACKS: &[(&str, Category)] = &[
    ("css", Category::TokensCss),
    ("reanimated", Category::TokensReanimated),
    ("gsap", Category::TokensGsap),
    ("react", Category::TokensReact),
    ("tailwind", Category::TokensTailwind),
    ("motion", Category::TokensMotion),
    ("r3f", Category::TokensR3f),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Bezier([i32; 4]);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Spring {
    stiffness: i32,
    damping: i32,
    mass: i32,
}

#[derive(Clone, Debug, Default)]
pub struct MotionTokens {
    durations: BTreeMap<String, u32>,
    easings: BTreeMap<String, Bezier>,
    springs: BTreeMap<String, Spring>,
}

impl MotionTokens {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.durations.is_empty() && self.easings.is_empty() && self.springs.is_empty()
    }

    pub fn merge(&mut self, other: MotionTokens) {
        self.durations.extend(other.durations);
        self.easings.extend(other.easings);
        self.springs.extend(other.springs);
    }

    #[must_use]
    pub fn has_duration_ms(&self, value: u32) -> bool {
        self.durations.values().any(|known| *known == value)
    }

    #[must_use]
    fn has_easing(&self, value: Bezier) -> bool {
        self.easings.values().any(|known| *known == value)
    }

    #[must_use]
    fn has_spring(&self, value: Spring) -> bool {
        self.springs.values().any(|known| *known == value)
    }

    /// Whether a Tailwind class name references a known token: named
    /// durations/delays (`duration-fast`), named easings (`ease-out`), or
    /// arbitrary values. Variant prefixes (`hover:`) are stripped first.
    #[must_use]
    fn has_tailwind_token_class(&self, class: &str) -> bool {
        let base = class.rsplit(':').next().unwrap_or(class);
        if let Some(name) = base
            .strip_prefix("duration-")
            .or_else(|| base.strip_prefix("delay-"))
        {
            return !name.starts_with('[') && self.durations.contains_key(name);
        }
        if let Some(name) = base.strip_prefix("ease-") {
            return !name.starts_with('[') && self.easings.contains_key(name);
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct AnalyzeOutcome {
    pub findings: Vec<Finding>,
    pub coverage: Vec<Coverage>,
}

#[must_use]
pub fn empty_coverage() -> Vec<Coverage> {
    STACKS
        .iter()
        .map(|(stack, _)| Coverage::new(stack))
        .collect()
}

pub fn merge_coverage(into: &mut [Coverage], other: &[Coverage]) {
    for entry in other {
        if let Some(target) = into.iter_mut().find(|target| target.stack == entry.stack) {
            target.tokenized_references += entry.tokenized_references;
            target.hardcoded_literals += entry.hardcoded_literals;
            target.drift += entry.drift;
            target.orphan += entry.orphan;
        }
    }
}

#[must_use]
pub fn discover_tokens(source: &str, source_type: SourceType) -> MotionTokens {
    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source, source_type).parse();
    if parser_return.panicked {
        return MotionTokens::default();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parser_return.program)
        .semantic;

    let mut tokens = MotionTokens::default();
    for node in semantic.nodes() {
        let oxc_ast::AstKind::VariableDeclarator(declarator) = node.kind() else {
            continue;
        };
        let Some(identifier) = declarator.id.get_binding_identifier() else {
            continue;
        };
        if !matches!(identifier.name.as_str(), "motion" | "motionTokens") {
            continue;
        }
        let Some(init) = &declarator.init else {
            continue;
        };
        let Some(object) = object_expression(init) else {
            continue;
        };
        let has_any = object_property_object(object, "duration").is_some()
            || object_property_object(object, "easing").is_some()
            || object_property_object(object, "spring").is_some();
        if !has_any {
            continue;
        }
        collect_object_tokens(object, &mut tokens);
    }
    tokens
}

#[must_use]
pub fn discover_css_tokens(source: &str) -> MotionTokens {
    let mut tokens = MotionTokens::default();
    for line in source.lines() {
        for (name, value) in css_custom_properties(line, "--motion-duration-") {
            if let Some(ms) = parse_duration(value) {
                tokens.durations.insert(name.to_string(), ms);
            }
        }
        for (name, value) in css_custom_properties(line, "--motion-ease-") {
            if let Some(bezier) = parse_cubic_bezier(value) {
                tokens.easings.insert(name.to_string(), bezier);
            }
        }
    }
    tokens
}

#[must_use]
pub fn analyze_css(relative_path: &str, source: &str, tokens: &MotionTokens) -> AnalyzeOutcome {
    let mut findings = Vec::new();
    let mut coverage = empty_coverage();
    coverage_for(Category::TokensCss, &mut coverage).tokenized_references +=
        source.matches("var(--motion-").count();

    for (line_index, raw_line) in source.lines().enumerate() {
        let line = strip_motion_custom_properties(raw_line);
        let line_number = u32::try_from(line_index + 1).unwrap_or(u32::MAX);
        if line.contains("transition") || line.contains("animation") {
            for (column, ms, raw) in duration_literals(&line) {
                emit_duration(
                    &mut findings,
                    &mut coverage,
                    tokens,
                    &LiteralContext {
                        id: ids::CSS_DURATION_LITERAL,
                        category: Category::TokensCss,
                        file: relative_path,
                        line: line_number,
                        column,
                        value_ms: ms,
                        raw: &raw,
                        unit: "ms",
                    },
                );
            }
        }
        for (column, bezier, raw) in cubic_bezier_literals(&line) {
            emit_easing(
                &mut findings,
                &mut coverage,
                tokens,
                &EasingContext {
                    id: ids::CSS_EASING_LITERAL,
                    category: Category::TokensCss,
                    location: Location {
                        file: relative_path,
                        line: line_number,
                        column,
                    },
                    bezier,
                    raw: &raw,
                },
            );
        }
    }

    AnalyzeOutcome { findings, coverage }
}

#[must_use]
pub fn analyze_source(
    relative_path: &str,
    source: &str,
    source_type: SourceType,
    tokens: &MotionTokens,
) -> AnalyzeOutcome {
    let allocator = Allocator::default();
    let parser_return = Parser::new(&allocator, source, source_type).parse();
    if parser_return.panicked {
        return AnalyzeOutcome {
            findings: Vec::new(),
            coverage: empty_coverage(),
        };
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parser_return.program)
        .semantic;
    let line_index = LineIndex::new(source);
    let mut findings = Vec::new();
    let mut coverage = empty_coverage();
    count_js_token_references(source, &semantic, &mut coverage);

    for node in semantic.nodes() {
        match node.kind() {
            oxc_ast::AstKind::CallExpression(call) => {
                check_reanimated_call(
                    call,
                    relative_path,
                    &line_index,
                    tokens,
                    &mut findings,
                    &mut coverage,
                );
                check_gsap_call(
                    call,
                    relative_path,
                    &line_index,
                    tokens,
                    &mut findings,
                    &mut coverage,
                );
            }
            oxc_ast::AstKind::ObjectExpression(object) => {
                let handled = check_motion_jsx_object(
                    object,
                    node.id(),
                    &semantic,
                    relative_path,
                    &line_index,
                    tokens,
                    &mut findings,
                    &mut coverage,
                );
                if !handled {
                    check_motion_react_object(
                        object,
                        relative_path,
                        &line_index,
                        tokens,
                        &mut findings,
                        &mut coverage,
                    );
                }
            }
            oxc_ast::AstKind::StringLiteral(string) => {
                if literal_in_class_position(&semantic, node.id()) {
                    check_tailwind_literal(
                        string.span,
                        source,
                        relative_path,
                        &line_index,
                        tokens,
                        &mut findings,
                        &mut coverage,
                    );
                }
            }
            oxc_ast::AstKind::TemplateLiteral(template) => {
                // Scan only the static quasis: the full template span
                // overlaps StringLiteral visits for embedded expressions
                // (e.g. `${cond ? "duration-[200ms]" : ""}`), which would
                // double findings and coverage counts.
                if !literal_in_class_position(&semantic, node.id()) {
                    continue;
                }
                for quasi in &template.quasis {
                    check_tailwind_literal(
                        quasi.span,
                        source,
                        relative_path,
                        &line_index,
                        tokens,
                        &mut findings,
                        &mut coverage,
                    );
                }
            }
            _ => {}
        }
    }

    findings.sort_by(|left, right| {
        (left.line, left.column, left.id.as_str()).cmp(&(
            right.line,
            right.column,
            right.id.as_str(),
        ))
    });
    AnalyzeOutcome { findings, coverage }
}

fn collect_object_tokens(object: &ObjectExpression<'_>, tokens: &mut MotionTokens) {
    if let Some(duration) = object_property_object(object, "duration") {
        for (name, value) in numeric_properties(duration) {
            if let Some(ms) = round_ms(value) {
                tokens.durations.insert(name, ms);
            }
        }
    }
    if let Some(easing) = object_property_object(object, "easing") {
        for property in &easing.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let Some(name) = property_key_name(&property.key) else {
                continue;
            };
            let Expression::ArrayExpression(array) = property.value.without_parentheses() else {
                continue;
            };
            if let Some(bezier) = bezier_from_array(array) {
                tokens.easings.insert(name.to_string(), bezier);
            }
        }
    }
    if let Some(spring) = object_property_object(object, "spring") {
        for property in &spring.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let Some(name) = property_key_name(&property.key) else {
                continue;
            };
            let Expression::ObjectExpression(config) = property.value.without_parentheses() else {
                continue;
            };
            if let Some(value) = spring_from_object(config) {
                tokens.springs.insert(name.to_string(), value);
            }
        }
    }
}

fn check_reanimated_call(
    call: &CallExpression<'_>,
    relative_path: &str,
    line_index: &LineIndex,
    tokens: &MotionTokens,
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
) {
    if callee_name(call) == Some("withTiming")
        && let Some(config) = call.arguments.get(1).and_then(argument_expression)
        && let Expression::ObjectExpression(object) = config.without_parentheses()
        && let Some((value, span)) = object_numeric_property(object, "duration")
        && let Some(ms) = round_ms(value)
    {
        let (line, column) = line_index.line_col(span.start);
        emit_duration(
            findings,
            coverage,
            tokens,
            &LiteralContext {
                id: ids::REANIMATED_DURATION_LITERAL,
                category: Category::TokensReanimated,
                file: relative_path,
                line,
                column,
                value_ms: ms,
                raw: &format!("{value}"),
                unit: "ms",
            },
        );
    }

    if callee_name(call) == Some("withDelay")
        && let Some(expression) = call.arguments.first().and_then(argument_expression)
        && let Some(value) = numeric_expression(expression)
        && let Some(ms) = round_ms(value)
    {
        let (line, column) = line_index.line_col(expression.span().start);
        emit_duration(
            findings,
            coverage,
            tokens,
            &LiteralContext {
                id: ids::REANIMATED_DURATION_LITERAL,
                category: Category::TokensReanimated,
                file: relative_path,
                line,
                column,
                value_ms: ms,
                raw: &format!("{value}"),
                unit: "ms",
            },
        );
    }

    if is_static_member_call(call, "Easing", "bezier") {
        let values: Vec<f64> = call
            .arguments
            .iter()
            .filter_map(argument_expression)
            .filter_map(numeric_expression)
            .collect();
        if values.len() == 4
            && let Some(bezier) = quantize_bezier(&values)
        {
            let (line, column) = line_index.line_col(call.span.start);
            emit_easing(
                findings,
                coverage,
                tokens,
                &EasingContext {
                    id: ids::REANIMATED_EASING_LITERAL,
                    category: Category::TokensReanimated,
                    location: Location {
                        file: relative_path,
                        line,
                        column,
                    },
                    bezier,
                    raw: "Easing.bezier(...)",
                },
            );
        }
    }

    if callee_name(call) == Some("withSpring")
        && let Some(config) = call.arguments.get(1).and_then(argument_expression)
        && let Expression::ObjectExpression(object) = config.without_parentheses()
        && has_spring_literal(object)
    {
        coverage_for(Category::TokensReanimated, coverage).hardcoded_literals += 1;
        let drift = spring_from_object(object).is_some_and(|spring| tokens.has_spring(spring));
        let entry = coverage_for(Category::TokensReanimated, coverage);
        if drift {
            entry.drift += 1;
        } else {
            entry.orphan += 1;
        }
        let (line, column) = line_index.line_col(object.span.start);
        findings.push(classified_finding(
            ids::REANIMATED_SPRING_LITERAL,
            Category::TokensReanimated,
            Location {
                file: relative_path,
                line,
                column,
            },
            drift,
            "Inline Reanimated spring config.",
            "Reference motion.spring instead of duplicating stiffness/damping/mass.",
        ));
    }
}

fn check_gsap_call(
    call: &CallExpression<'_>,
    relative_path: &str,
    line_index: &LineIndex,
    tokens: &MotionTokens,
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
) {
    if !is_gsap_tween_call(call) {
        return;
    }
    for argument in &call.arguments {
        let Some(Expression::ObjectExpression(object)) =
            argument_expression(argument).map(Expression::without_parentheses)
        else {
            continue;
        };
        if let Some((value, span)) = object_numeric_property(object, "duration")
            && let Some(ms) = round_ms(value * 1000.0)
        {
            let (line, column) = line_index.line_col(span.start);
            emit_duration(
                findings,
                coverage,
                tokens,
                &LiteralContext {
                    id: ids::GSAP_DURATION_LITERAL,
                    category: Category::TokensGsap,
                    file: relative_path,
                    line,
                    column,
                    value_ms: ms,
                    raw: &format!("{value}"),
                    unit: "ms",
                },
            );
        }
        if let Some((ease, span)) = object_string_property(object, "ease") {
            coverage_for(Category::TokensGsap, coverage).hardcoded_literals += 1;
            let (line, column) = line_index.line_col(span.start);
            findings.push(classified_finding(
                ids::GSAP_EASING_LITERAL,
                Category::TokensGsap,
                Location {
                    file: relative_path,
                    line,
                    column,
                },
                false,
                &format!("Hardcoded GSAP ease `{ease}`."),
                "Reference the shared motion easing token vocabulary.",
            ));
            coverage_for(Category::TokensGsap, coverage).orphan += 1;
        }
    }
}

/// Motion 12 JSX props express durations in seconds and cubic-bezier easings as
/// four-number arrays. Only objects directly attached to a `motion.*` prop are
/// considered, avoiding generic objects that happen to contain `transition`.
#[allow(clippy::too_many_arguments)]
fn check_motion_jsx_object(
    object: &ObjectExpression<'_>,
    node_id: oxc_semantic::NodeId,
    semantic: &Semantic<'_>,
    relative_path: &str,
    line_index: &LineIndex,
    tokens: &MotionTokens,
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
) -> bool {
    let Some(attribute) = motion_jsx_attribute_for_object(semantic, node_id) else {
        return false;
    };
    match attribute {
        "transition" => emit_motion_transition_literals(
            object,
            relative_path,
            line_index,
            tokens,
            findings,
            coverage,
        ),
        "animate" | "exit" | "initial" => {
            // The outer object holds animation targets, not transition
            // options: a target named `duration` or `ease` is not timing.
            // Only the nested `transition` object carries timing options.
            if let Some(transition) = object_property_object(object, "transition") {
                emit_motion_transition_literals(
                    transition,
                    relative_path,
                    line_index,
                    tokens,
                    findings,
                    coverage,
                );
            }
        }
        _ => {}
    }
    true
}

fn emit_motion_transition_literals(
    object: &ObjectExpression<'_>,
    relative_path: &str,
    line_index: &LineIndex,
    tokens: &MotionTokens,
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
) {
    if let Some((value, span)) = object_numeric_property(object, "duration")
        && let Some(ms) = round_ms(value * 1000.0)
    {
        let (line, column) = line_index.line_col(span.start);
        let raw = format!("{value}");
        let context = LiteralContext {
            id: ids::MOTION_DURATION_LITERAL,
            category: Category::TokensMotion,
            file: relative_path,
            line,
            column,
            value_ms: ms,
            raw: &raw,
            unit: "ms",
        };
        emit_duration(findings, coverage, tokens, &context);
    }
    if let Some((bezier, span)) = object_array_property(object, "ease")
        .and_then(|(array, span)| bezier_from_array(array).map(|bezier| (bezier, span)))
    {
        let (line, column) = line_index.line_col(span.start);
        let context = EasingContext {
            id: ids::MOTION_EASING_LITERAL,
            category: Category::TokensMotion,
            location: Location {
                file: relative_path,
                line,
                column,
            },
            bezier,
            raw: "motion transition ease array",
        };
        emit_easing(findings, coverage, tokens, &context);
    }
    // Value-specific transition objects (`transition={{ opacity: { duration:
    // 0.2 } }}`) carry the same hardcoded timings one level down.
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            continue;
        };
        if matches!(property_key_name(&property.key), Some("duration" | "ease")) {
            continue;
        }
        if let Some(nested) = object_expression(&property.value) {
            emit_motion_transition_literals(
                nested,
                relative_path,
                line_index,
                tokens,
                findings,
                coverage,
            );
        }
    }
}

fn motion_jsx_attribute_for_object<'a>(
    semantic: &'a Semantic<'a>,
    node_id: oxc_semantic::NodeId,
) -> Option<&'a str> {
    if let Some(attribute) = direct_motion_jsx_attribute(semantic, node_id) {
        return Some(attribute);
    }
    // Reusable config (`const transition = { duration: 0.2 };
    // <motion.div transition={transition} />`): the declaration has no JSX
    // ancestor, so resolve it through references to a motion prop instead.
    resolve_transition_prop_reference(semantic, node_id)
}

fn direct_motion_jsx_attribute<'a>(
    semantic: &'a Semantic<'a>,
    node_id: oxc_semantic::NodeId,
) -> Option<&'a str> {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = node_id;
    for _ in 0..6 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return None;
        }
        match nodes.kind(parent_id) {
            AstKind::JSXAttribute(attribute) => {
                let name = jsx_attribute_name(&attribute.name)?;
                if !matches!(name, "transition" | "animate" | "exit" | "initial") {
                    return None;
                }
                return jsx_attribute_belongs_to_motion(semantic, parent_id).then_some(name);
            }
            // A nested object is handled through its direct motion prop object.
            AstKind::ObjectProperty(_) | AstKind::ObjectExpression(_) => return None,
            _ => current = parent_id,
        }
    }
    None
}

/// Resolve an object literal held in a variable to the motion prop that
/// consumes it, if any. Only direct declarator initializers qualify, so a
/// nested object is never reported twice (once directly, once by reference).
fn resolve_transition_prop_reference<'a>(
    semantic: &'a Semantic<'a>,
    node_id: oxc_semantic::NodeId,
) -> Option<&'a str> {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let parent_id = nodes.parent_id(node_id);
    if parent_id == node_id {
        return None;
    }
    let AstKind::VariableDeclarator(declarator) = nodes.kind(parent_id) else {
        return None;
    };
    let symbol_id = declarator.id.get_binding_identifier()?.symbol_id.get()?;
    semantic
        .scoping()
        .get_resolved_reference_ids(symbol_id)
        .iter()
        .find_map(|reference_id| reference_motion_prop(semantic, *reference_id))
}

/// Whether a variable reference sits in a motion JSX transition-style prop
/// value, returning that prop name.
fn reference_motion_prop<'a>(
    semantic: &'a Semantic<'a>,
    reference_id: oxc_semantic::ReferenceId,
) -> Option<&'a str> {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = semantic.scoping().get_reference(reference_id).node_id();
    for _ in 0..5 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return None;
        }
        if let AstKind::JSXAttribute(attribute) = nodes.kind(parent_id) {
            let name = jsx_attribute_name(&attribute.name)?;
            if !matches!(name, "transition" | "animate" | "exit" | "initial") {
                return None;
            }
            return jsx_attribute_belongs_to_motion(semantic, parent_id).then_some(name);
        }
        current = parent_id;
    }
    None
}

fn jsx_attribute_belongs_to_motion(
    semantic: &Semantic<'_>,
    attribute_id: oxc_semantic::NodeId,
) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = attribute_id;
    for _ in 0..3 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        if let AstKind::JSXOpeningElement(_) = nodes.kind(parent_id) {
            return jsx_element_is_motion_member(semantic, parent_id);
        }
        current = parent_id;
    }
    false
}

fn jsx_attribute_name<'a>(name: &'a JSXAttributeName<'a>) -> Option<&'a str> {
    match name {
        JSXAttributeName::Identifier(identifier) => Some(identifier.name.as_str()),
        JSXAttributeName::NamespacedName(_) => None,
    }
}

fn jsx_element_is_motion_member(semantic: &Semantic<'_>, opening_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let AstKind::JSXOpeningElement(opening) = nodes.kind(opening_id) else {
        return false;
    };
    match &opening.name {
        JSXElementName::MemberExpression(member) => {
            let Some(root) = jsx_member_root_reference(&member.object) else {
                return false;
            };
            match motion_import_source(semantic, root) {
                // Imported binding: only Motion-family modules count, under any local
                // alias (`import { motion as m }` with `<m.div>`).
                Some(source) => is_motion_module(source),
                // Unresolved (global/UMD script): keep the legacy spelling behavior.
                // A binding that resolves to a local declaration is an unrelated
                // namespace that merely shares the name.
                None => root.name.as_str() == "motion" && !resolves_to_symbol(semantic, root),
            }
        }
        // Component stored in an identifier (`const MotionDiv = motion.div`):
        // resolves when the declarator initializes it from the Motion namespace.
        JSXElementName::IdentifierReference(identifier) => {
            jsx_identifier_is_motion_component(semantic, identifier.as_ref())
        }
        _ => false,
    }
}

/// Whether a JSX identifier component (e.g. `MotionDiv`) is initialized from
/// the imported Motion namespace (`const MotionDiv = motion.div`).
fn jsx_identifier_is_motion_component(
    semantic: &Semantic<'_>,
    identifier: &IdentifierReference<'_>,
) -> bool {
    use oxc_ast::AstKind;

    let Some(reference_id) = identifier.reference_id.get() else {
        return false;
    };
    let Some(symbol_id) = semantic.scoping().get_reference(reference_id).symbol_id() else {
        return false;
    };
    let declaration = semantic.scoping().symbol_declaration(symbol_id);
    let nodes = semantic.nodes();
    // The declaration may be the declarator itself or the bound identifier,
    // so climb at most two levels to the enclosing declarator.
    let mut declarator_id = declaration;
    for _ in 0..2 {
        if matches!(nodes.kind(declarator_id), AstKind::VariableDeclarator(_)) {
            break;
        }
        let parent_id = nodes.parent_id(declarator_id);
        if parent_id == declarator_id {
            return false;
        }
        declarator_id = parent_id;
    }
    let AstKind::VariableDeclarator(declarator) = nodes.kind(declarator_id) else {
        return false;
    };
    let Some(init) = &declarator.init else {
        return false;
    };
    // The initializer must be `<namespace>.<member>` on a Motion import.
    // `motion(Component)` HOC shapes stay out of scope.
    let Expression::StaticMemberExpression(member) = init.without_parentheses() else {
        return false;
    };
    let Expression::Identifier(object) = member.object.without_parentheses() else {
        return false;
    };
    motion_import_source(semantic, object).is_some_and(is_motion_module)
}

/// Whether an import source provides the Motion JSX namespace.
fn is_motion_module(source: &str) -> bool {
    source == "motion" || source.starts_with("motion/") || source == "framer-motion"
}

/// The import source a JSX namespace identifier resolves to, if it resolves
/// to an import binding at all.
fn motion_import_source<'a>(
    semantic: &'a Semantic<'a>,
    identifier: &IdentifierReference<'_>,
) -> Option<&'a str> {
    use oxc_ast::AstKind;

    let reference_id = identifier.reference_id.get()?;
    let symbol_id = semantic.scoping().get_reference(reference_id).symbol_id()?;
    let declaration = semantic.scoping().symbol_declaration(symbol_id);
    let parent_id = semantic.nodes().parent_id(declaration);
    if parent_id == declaration {
        return None;
    }
    match semantic.nodes().kind(parent_id) {
        AstKind::ImportDeclaration(import) => Some(import.source.value.as_str()),
        _ => None,
    }
}

/// Whether a JSX namespace identifier resolves to any declared symbol.
fn resolves_to_symbol(semantic: &Semantic<'_>, identifier: &IdentifierReference<'_>) -> bool {
    identifier
        .reference_id
        .get()
        .and_then(|reference_id| semantic.scoping().get_reference(reference_id).symbol_id())
        .is_some()
}

fn jsx_member_root_reference<'a>(
    object: &'a JSXMemberExpressionObject<'a>,
) -> Option<&'a IdentifierReference<'a>> {
    match object {
        JSXMemberExpressionObject::IdentifierReference(identifier) => Some(identifier),
        JSXMemberExpressionObject::MemberExpression(member) => {
            jsx_member_root_reference(&member.object)
        }
        JSXMemberExpressionObject::ThisExpression(_) => None,
    }
}

/// Class-composition calls whose string arguments become classes.
const CLASS_COMPOSITION_CALLS: &[&str] = &[
    "clsx",
    "cn",
    "cx",
    "classNames",
    "twMerge",
    "twJoin",
    "cva",
    "tv",
];

/// Whether a string/template literal sits in class position: a
/// `className`/`class` JSX attribute, or an argument of a recognized
/// class-composition call. Unrelated strings (docs, fixtures, messages)
/// never become classes and must not produce Tailwind findings.
fn literal_in_class_position(semantic: &Semantic<'_>, node_id: oxc_semantic::NodeId) -> bool {
    use oxc_ast::AstKind;

    let nodes = semantic.nodes();
    let mut current = node_id;
    for _ in 0..8 {
        let parent_id = nodes.parent_id(current);
        if parent_id == current {
            return false;
        }
        match nodes.kind(parent_id) {
            AstKind::JSXAttribute(attribute) => {
                return matches!(
                    jsx_attribute_name(&attribute.name),
                    Some("className" | "class")
                );
            }
            AstKind::CallExpression(call) => {
                if let Expression::Identifier(identifier) = call.callee.without_parentheses()
                    && CLASS_COMPOSITION_CALLS.contains(&identifier.name.as_str())
                {
                    return true;
                }
            }
            _ => {}
        }
        current = parent_id;
    }
    false
}

/// Scan a string/template literal for Tailwind/NativeWind arbitrary motion
/// classes. The literal span keeps locations exact without scanning comments or
/// identifiers elsewhere in the file.
#[allow(clippy::too_many_arguments)]
fn check_tailwind_literal(
    span: Span,
    source: &str,
    relative_path: &str,
    line_index: &LineIndex,
    tokens: &MotionTokens,
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
) {
    let start = usize::try_from(span.start).unwrap_or(usize::MAX);
    let end = usize::try_from(span.end).unwrap_or(usize::MAX);
    let Some(literal) = source.get(start..end) else {
        return;
    };

    // Named token classes (`duration-fast`) count as tokenized references so
    // the Tailwind stack coverage reflects token usage, not just literals.
    // Strip one layer of quotes: the span covers the literal verbatim.
    let unquoted = literal
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .or_else(|| {
            literal
                .strip_prefix('\'')
                .and_then(|text| text.strip_suffix('\''))
        })
        .or_else(|| {
            literal
                .strip_prefix('`')
                .and_then(|text| text.strip_suffix('`'))
        })
        .unwrap_or(literal);
    let named_refs = unquoted
        .split_whitespace()
        .filter(|class| tokens.has_tailwind_token_class(class))
        .count();
    if named_refs > 0 {
        coverage_for(Category::TokensTailwind, coverage).tokenized_references += named_refs;
    }

    for prefix in ["duration-[", "delay-["] {
        for (offset, value, raw) in tailwind_ms_literals(literal, prefix) {
            let absolute = span
                .start
                .saturating_add(u32::try_from(offset).unwrap_or(u32::MAX));
            let (line, column) = line_index.line_col(absolute);
            let context = LiteralContext {
                id: ids::TAILWIND_DURATION_LITERAL,
                category: Category::TokensTailwind,
                file: relative_path,
                line,
                column,
                value_ms: value,
                raw: &raw,
                unit: "ms",
            };
            emit_duration(findings, coverage, tokens, &context);
        }
    }

    for (offset, bezier, raw) in tailwind_easing_literals(literal) {
        let absolute = span
            .start
            .saturating_add(u32::try_from(offset).unwrap_or(u32::MAX));
        let (line, column) = line_index.line_col(absolute);
        let context = EasingContext {
            id: ids::TAILWIND_EASING_LITERAL,
            category: Category::TokensTailwind,
            location: Location {
                file: relative_path,
                line,
                column,
            },
            bezier,
            raw: &raw,
        };
        emit_easing(findings, coverage, tokens, &context);
    }
}

/// Split a class string into `(byte_offset, token)` pairs on ASCII
/// whitespace. Offsets keep finding spans exact.
fn split_class_tokens(literal: &str) -> Vec<(usize, &str)> {
    let bytes = literal.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        let start = index;
        while index < bytes.len() && !bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if start < index {
            tokens.push((start, &literal[start..index]));
        }
    }
    tokens
}

/// Strip variant prefixes (`hover:`, `md:dark:`) to the utility itself.
fn strip_variant_prefixes(class: &str) -> &str {
    class.rsplit(':').next().unwrap_or(class)
}

/// Resolve a raw class token to the utility text and its byte offset:
/// surrounding quotes and variant prefixes are stripped with the offset
/// adjusted so finding spans stay exact.
fn utility_text(token_start: usize, token: &str) -> (usize, &str) {
    let is_quote = |candidate: char| candidate == '\'' || candidate == '"' || candidate == '`';
    let unquoted_leading = token.trim_start_matches(is_quote);
    let mut start = token_start + (token.len() - unquoted_leading.len());
    let unquoted = unquoted_leading.trim_end_matches(is_quote);
    let class = strip_variant_prefixes(unquoted);
    start += unquoted.len() - class.len();
    (start, class)
}

fn tailwind_ms_literals(literal: &str, prefix: &str) -> Vec<(usize, u32, String)> {
    let mut out = Vec::new();
    // Parse per whitespace-separated class whose utility starts with the
    // prefix: a substring match would misread `my-duration-[200ms]`, which
    // Tailwind never interprets as a duration utility.
    for (token_start, token) in split_class_tokens(literal) {
        let (class_start, class) = utility_text(token_start, token);
        let Some(rest) = class.strip_prefix(prefix) else {
            continue;
        };
        let number_start = class_start + prefix.len();
        // Bound the parse to the current class: the value ends at the first
        // `]` after the prefix. Searching past it lets one invalid class
        // swallow a later valid one (`duration-[var(--x)] duration-[200ms]`
        // would skip the hardcoded duration entirely).
        let rest = &literal[number_start..];
        let Some(bracket) = rest.find(']') else {
            continue;
        };
        let raw_end = number_start + bracket + 1;
        if let Some(number) = rest[..bracket].strip_suffix("ms")
            && !number.is_empty()
            && number.bytes().all(|byte| byte.is_ascii_digit())
            && let Ok(value) = number.parse::<u32>()
        {
            out.push((
                class_start,
                value,
                literal[class_start..raw_end].to_string(),
            ));
        }
    }
    out
}

fn tailwind_easing_literals(literal: &str) -> Vec<(usize, Bezier, String)> {
    let prefix = "ease-[cubic-bezier(";
    let mut out = Vec::new();
    // Same per-class discipline as `tailwind_ms_literals`: only utilities
    // starting with the prefix count, after variant stripping.
    for (token_start, token) in split_class_tokens(literal) {
        let (class_start, class) = utility_text(token_start, token);
        if class.strip_prefix(prefix).is_none() {
            continue;
        }
        // Bound the parse to the current class: the value ends at the first
        // `]` ahead.
        let rest = &literal[class_start + "ease-[".len()..];
        let Some(bracket) = rest.find(']') else {
            continue;
        };
        let raw_end = class_start + "ease-[".len() + bracket + 1;
        if let Some(bezier) = parse_cubic_bezier(&rest[..bracket]) {
            out.push((
                class_start,
                bezier,
                literal[class_start..raw_end].to_string(),
            ));
        }
    }
    out
}

fn check_motion_react_object(
    object: &ObjectExpression<'_>,
    relative_path: &str,
    line_index: &LineIndex,
    tokens: &MotionTokens,
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
) {
    let Some(transition) = object_property_object(object, "transition") else {
        return;
    };
    if let Some((value, span)) = object_numeric_property(transition, "duration")
        && let Some(ms) = round_ms(value * 1000.0)
    {
        let (line, column) = line_index.line_col(span.start);
        emit_duration(
            findings,
            coverage,
            tokens,
            &LiteralContext {
                id: ids::REACT_DURATION_LITERAL,
                category: Category::TokensReact,
                file: relative_path,
                line,
                column,
                value_ms: ms,
                raw: &format!("{value}"),
                unit: "ms",
            },
        );
    }
    if let Some((bezier, span)) = object_array_property(transition, "ease")
        .and_then(|(array, span)| bezier_from_array(array).map(|bezier| (bezier, span)))
    {
        let (line, column) = line_index.line_col(span.start);
        emit_easing(
            findings,
            coverage,
            tokens,
            &EasingContext {
                id: ids::REACT_EASING_LITERAL,
                category: Category::TokensReact,
                location: Location {
                    file: relative_path,
                    line,
                    column,
                },
                bezier,
                raw: "transition.ease",
            },
        );
    }
}

fn count_js_token_references(source: &str, semantic: &Semantic<'_>, coverage: &mut [Coverage]) {
    let shared_count = source.matches("motion.duration").count()
        + source.matches("motion.easing").count()
        + source.matches("motion.spring").count()
        + source.matches("motionTokens.").count();
    for category in js_stack_categories(source, has_motion_jsx(semantic)) {
        coverage_for(category, coverage).tokenized_references += shared_count;
    }
    coverage_for(Category::TokensReanimated, coverage).tokenized_references +=
        source.matches("reanimatedMotion.").count();
}

/// Whether the file renders Motion JSX under any namespace spelling,
/// including import aliases (`import { motion as m }` with `<m.div>`).
/// Coverage classification must follow the resolved binding like the JSX
/// rule does, not the literal `<motion.` text.
fn has_motion_jsx(semantic: &Semantic<'_>) -> bool {
    use oxc_ast::AstKind;

    semantic.nodes().iter().any(|node| {
        if let AstKind::JSXOpeningElement(_) = node.kind() {
            jsx_element_is_motion_member(semantic, node.id())
        } else {
            false
        }
    })
}

fn js_stack_categories(source: &str, motion_jsx: bool) -> Vec<Category> {
    let mut categories = Vec::new();
    if source.contains("withTiming")
        || source.contains("withDelay")
        || source.contains("withSpring")
        || source.contains("Easing.bezier")
        || source.contains("reanimatedMotion.")
    {
        categories.push(Category::TokensReanimated);
    }
    if source.contains("gsap.") {
        categories.push(Category::TokensGsap);
    }
    if source.contains("<motion.") || motion_jsx {
        categories.push(Category::TokensMotion);
    } else if source.contains("transition") {
        categories.push(Category::TokensReact);
    }
    if source.contains("@react-three/fiber") || source.contains("useFrame") {
        categories.push(Category::TokensR3f);
    }
    if source.contains("duration-[") || source.contains("delay-[") || source.contains("ease-[") {
        categories.push(Category::TokensTailwind);
    }
    if categories.is_empty() && source.contains("motionTokens.") {
        categories.push(Category::TokensReact);
    }
    categories
}

struct LiteralContext<'a> {
    id: &'a str,
    category: Category,
    file: &'a str,
    line: u32,
    column: u32,
    value_ms: u32,
    raw: &'a str,
    unit: &'a str,
}

#[derive(Clone, Copy)]
struct Location<'a> {
    file: &'a str,
    line: u32,
    column: u32,
}

struct EasingContext<'a> {
    id: &'a str,
    category: Category,
    location: Location<'a>,
    bezier: Bezier,
    raw: &'a str,
}

fn emit_duration(
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
    tokens: &MotionTokens,
    context: &LiteralContext<'_>,
) {
    coverage_for(context.category, coverage).hardcoded_literals += 1;
    let drift = tokens.has_duration_ms(context.value_ms);
    let entry = coverage_for(context.category, coverage);
    if drift {
        entry.drift += 1;
    } else {
        entry.orphan += 1;
    }
    findings.push(classified_finding(
        context.id,
        context.category,
        Location {
            file: context.file,
            line: context.line,
            column: context.column,
        },
        drift,
        &format!(
            "Hardcoded duration `{}` normalizes to {}{}.",
            context.raw, context.value_ms, context.unit
        ),
        "Reference the shared motion duration token instead of an inline literal.",
    ));
}

fn emit_easing(
    findings: &mut Vec<Finding>,
    coverage: &mut [Coverage],
    tokens: &MotionTokens,
    context: &EasingContext<'_>,
) {
    coverage_for(context.category, coverage).hardcoded_literals += 1;
    let drift = tokens.has_easing(context.bezier);
    let entry = coverage_for(context.category, coverage);
    if drift {
        entry.drift += 1;
    } else {
        entry.orphan += 1;
    }
    findings.push(classified_finding(
        context.id,
        context.category,
        context.location,
        drift,
        &format!("Hardcoded easing `{}`.", context.raw),
        "Reference the shared motion easing token instead of an inline literal.",
    ));
}

fn classified_finding(
    id: &str,
    category: Category,
    location: Location<'_>,
    drift: bool,
    message: &str,
    suggestion: &str,
) -> Finding {
    let kind = if drift { "drift" } else { "orphan" };
    Finding {
        id: id.to_string(),
        category,
        severity: if drift {
            Severity::Medium
        } else {
            Severity::Low
        },
        confidence: descriptor(id).map_or(Confidence::High, |rule| rule.confidence),
        file: location.file.to_string(),
        line: location.line,
        column: location.column,
        message: format!("{kind}: {message}"),
        suggestion: suggestion.to_string(),
    }
}

fn coverage_for(category: Category, coverage: &mut [Coverage]) -> &mut Coverage {
    let stack = STACKS
        .iter()
        .find(|(_, item)| *item == category)
        .map_or("r3f", |(stack, _)| *stack);
    coverage
        .iter_mut()
        .find(|entry| entry.stack == stack)
        .expect("coverage has every stack")
}

fn css_custom_properties<'a>(line: &'a str, prefix: &str) -> Vec<(&'a str, &'a str)> {
    let mut out = Vec::new();
    let mut start_at = 0;
    while let Some(offset) = line[start_at..].find(prefix) {
        let start = start_at + offset;
        let after_prefix = start + prefix.len();
        let next_semicolon = line[after_prefix..]
            .find(';')
            .map_or(line.len(), |offset| after_prefix + offset);
        let Some(name_offset) = line[after_prefix..next_semicolon].find(':') else {
            start_at = next_semicolon.saturating_add(1);
            continue;
        };
        let name_end = after_prefix + name_offset;
        let value_start = name_end + 1;
        let value_end = next_semicolon;
        out.push((
            line[after_prefix..name_end].trim(),
            line[value_start..value_end].trim(),
        ));
        start_at = value_end.saturating_add(1);
    }
    out
}

fn strip_motion_custom_properties(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut start_at = 0;
    while let Some(offset) = line[start_at..].find("--motion-") {
        let start = start_at + offset;
        let declaration_end = line[start..]
            .find(';')
            .map_or(line.len(), |offset| start + offset + 1);
        let has_colon = line[start..declaration_end].contains(':');
        if has_colon {
            out.push_str(&line[start_at..start]);
        } else {
            out.push_str(&line[start_at..declaration_end]);
        }
        start_at = declaration_end;
    }
    out.push_str(&line[start_at..]);
    out
}

fn duration_literals(line: &str) -> Vec<(u32, u32, String)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() && bytes[index] != b'.' {
            index += 1;
            continue;
        }
        let start = index;
        let mut seen_digit = false;
        while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
            if bytes[index].is_ascii_digit() {
                seen_digit = true;
            }
            index += 1;
        }
        if !seen_digit {
            continue;
        }
        let (unit, unit_len) = if line[index..].starts_with("ms") {
            ("ms", 2)
        } else if line[index..].starts_with('s') {
            ("s", 1)
        } else {
            continue;
        };
        let raw = &line[start..index + unit_len];
        if let Ok(number) = line[start..index].parse::<f64>() {
            let ms = if unit == "s" { number * 1000.0 } else { number };
            if let Some(value) = round_ms(ms) {
                out.push((
                    u32::try_from(start + 1).unwrap_or(u32::MAX),
                    value,
                    raw.to_string(),
                ));
            }
        }
        index += unit_len;
    }
    out
}

fn cubic_bezier_literals(line: &str) -> Vec<(u32, Bezier, String)> {
    let mut out = Vec::new();
    let mut start_at = 0;
    while let Some(offset) = line[start_at..].find("cubic-bezier(") {
        let start = start_at + offset;
        let value_start = start + "cubic-bezier(".len();
        let Some(end_offset) = line[value_start..].find(')') else {
            break;
        };
        let end = value_start + end_offset;
        let raw = &line[start..=end];
        if let Some(bezier) = parse_cubic_bezier(raw) {
            out.push((
                u32::try_from(start + 1).unwrap_or(u32::MAX),
                bezier,
                raw.to_string(),
            ));
        }
        start_at = end + 1;
    }
    out
}

fn parse_duration(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    if let Some(number) = trimmed.strip_suffix("ms") {
        return round_ms(number.trim().parse().ok()?);
    }
    if let Some(number) = trimmed.strip_suffix('s') {
        return round_ms(number.trim().parse::<f64>().ok()? * 1000.0);
    }
    None
}

fn parse_cubic_bezier(value: &str) -> Option<Bezier> {
    let inner = value
        .trim()
        .strip_prefix("cubic-bezier(")?
        .strip_suffix(')')?;
    let values: Vec<f64> = inner
        .split(',')
        .map(str::trim)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    quantize_bezier(&values)
}

fn quantize_bezier(values: &[f64]) -> Option<Bezier> {
    if values.len() != 4 {
        return None;
    }
    Some(Bezier([
        quantize(values[0]),
        quantize(values[1]),
        quantize(values[2]),
        quantize(values[3]),
    ]))
}

fn bezier_from_array(array: &ArrayExpression<'_>) -> Option<Bezier> {
    let values: Vec<f64> = array
        .elements
        .iter()
        .filter_map(|element| match element {
            oxc_ast::ast::ArrayExpressionElement::NumericLiteral(number) => Some(number.value),
            _ => None,
        })
        .collect();
    quantize_bezier(&values)
}

fn spring_from_object(object: &ObjectExpression<'_>) -> Option<Spring> {
    Some(Spring {
        stiffness: quantize(object_numeric_property(object, "stiffness")?.0),
        damping: quantize(object_numeric_property(object, "damping")?.0),
        mass: quantize(object_numeric_property(object, "mass")?.0),
    })
}

fn has_spring_literal(object: &ObjectExpression<'_>) -> bool {
    ["stiffness", "damping", "mass"]
        .iter()
        .any(|key| object_numeric_property(object, key).is_some())
}

fn numeric_properties(object: &ObjectExpression<'_>) -> Vec<(String, f64)> {
    object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            let name = property_key_name(&property.key)?;
            let value = numeric_expression(&property.value)?;
            Some((name.to_string(), value))
        })
        .collect()
}

fn object_property_object<'a>(
    object: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<&'a ObjectExpression<'a>> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property_key_name(&property.key) == Some(key)
            && let Some(object) = object_expression(&property.value)
        {
            return Some(object);
        }
        None
    })
}

fn object_expression<'a>(expression: &'a Expression<'a>) -> Option<&'a ObjectExpression<'a>> {
    match expression.without_parentheses() {
        Expression::ObjectExpression(object) => Some(object.as_ref()),
        Expression::TSAsExpression(inner) => object_expression(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => object_expression(&inner.expression),
        Expression::TSTypeAssertion(inner) => object_expression(&inner.expression),
        Expression::TSNonNullExpression(inner) => object_expression(&inner.expression),
        Expression::TSInstantiationExpression(inner) => object_expression(&inner.expression),
        _ => None,
    }
}

fn object_numeric_property(object: &ObjectExpression<'_>, key: &str) -> Option<(f64, Span)> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property_key_name(&property.key) == Some(key) {
            return numeric_expression(&property.value).map(|value| (value, property.value.span()));
        }
        None
    })
}

fn object_string_property<'a>(
    object: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<(&'a str, Span)> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property_key_name(&property.key) == Some(key)
            && let Expression::StringLiteral(string) = property.value.without_parentheses()
        {
            return Some((string.value.as_str(), property.value.span()));
        }
        None
    })
}

fn object_array_property<'a>(
    object: &'a ObjectExpression<'a>,
    key: &str,
) -> Option<(&'a ArrayExpression<'a>, Span)> {
    object.properties.iter().find_map(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return None;
        };
        if property_key_name(&property.key) == Some(key)
            && let Expression::ArrayExpression(array) = property.value.without_parentheses()
        {
            return Some((array.as_ref(), property.value.span()));
        }
        None
    })
}

fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn argument_expression<'a>(argument: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
    argument.as_expression()
}

fn numeric_expression(expression: &Expression<'_>) -> Option<f64> {
    match expression.without_parentheses() {
        Expression::NumericLiteral(number) => Some(number.value),
        Expression::UnaryExpression(unary) if unary.operator.as_str() == "-" => {
            numeric_expression(&unary.argument).map(|value| -value)
        }
        _ => None,
    }
}

fn callee_name<'a>(call: &'a CallExpression<'a>) -> Option<&'a str> {
    match call.callee.without_parentheses() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        _ => None,
    }
}

fn is_static_member_call(
    call: &CallExpression<'_>,
    object_name: &str,
    property_name: &str,
) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return false;
    };
    member.property.name.as_str() == property_name
        && matches!(
            member.object.without_parentheses(),
            Expression::Identifier(identifier) if identifier.name.as_str() == object_name
        )
}

fn is_gsap_tween_call(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() else {
        return false;
    };
    let method = member.property.name.as_str();
    matches!(method, "to" | "from" | "fromTo")
        && (is_identifier(&member.object, "gsap") || is_gsap_timeline_call(&member.object))
}

fn is_gsap_timeline_call(expression: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expression.without_parentheses() else {
        return false;
    };
    is_static_member_call(call, "gsap", "timeline")
}

fn is_identifier(expression: &Expression<'_>, name: &str) -> bool {
    matches!(
        expression.without_parentheses(),
        Expression::Identifier(identifier) if identifier.name.as_str() == name
    )
}

fn round_ms(value: f64) -> Option<u32> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    u32::try_from(value.round() as i64).ok()
}

fn quantize(value: f64) -> i32 {
    (value * 1000.0).round() as i32
}
