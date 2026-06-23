//! gsap-audit-core: static analysis library for auditing GSAP usage.
//!
//! This crate parses JS/TS/JSX/TSX source with oxc, runs semantic analysis for
//! scope/symbol resolution, and reports GSAP anti-patterns as owned, serde
//! [`Finding`] values. The binary crate `gsap-audit` is a thin CLI over this
//! library.

pub mod analyze;
pub mod output;
pub mod rules;
pub mod scan;
pub mod source;
pub mod types;

pub use analyze::analyze_source;
pub use output::{format_json, format_markdown, highest_severity};
pub use rules::{CATALOG, descriptor};
pub use scan::{ScanOptions, ScanOutcome, scan_root};
pub use source::source_type_for_extension;
pub use types::{Category, Confidence, Finding, RuleDescriptor, Severity};

/// Tool name used in output payloads.
pub const TOOL_NAME: &str = "gsap-audit";

/// Tool version, sourced from the crate's Cargo manifest.
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
