//! motion-token-audit-core: static analysis for cross-stack motion token drift.

pub mod analyze;
pub mod output;
pub mod rules;
pub mod scan;
pub mod source;
pub mod types;

pub use analyze::{
    MotionTokens, analyze_css, analyze_source, discover_css_tokens, discover_tokens,
};
pub use output::{format_json, format_markdown, highest_severity};
pub use rules::{CATALOG, descriptor};
pub use scan::{ScanOptions, ScanOutcome, scan_root};
pub use source::source_type_for_extension;
pub use types::{Category, Confidence, Coverage, Finding, RuleDescriptor, Severity};

pub const TOOL_NAME: &str = "motion-token-audit";
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
