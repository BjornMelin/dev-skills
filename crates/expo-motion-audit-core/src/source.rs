//! Source-position helpers and oxc `SourceType` selection.
//!
//! oxc spans are byte offsets into the source; oxc does not ship a built-in
//! offset-to-line/column converter, so we compute one here.

use oxc_span::SourceType;

/// Choose the oxc [`SourceType`] for a file extension.
///
/// `tsx`/`jsx` enable JSX; `ts`/`mts`/`cts` enable TypeScript (no JSX);
/// `mjs`/`cjs`/`js` are plain JavaScript modules with JSX enabled (a superset
/// that still parses non-JSX JavaScript). Unknown extensions fall back to a
/// permissive TSX-like type so we never silently skip a candidate file.
#[must_use]
pub fn source_type_for_extension(extension: &str) -> SourceType {
    match extension.to_ascii_lowercase().as_str() {
        "tsx" => SourceType::tsx(),
        "ts" | "mts" | "cts" => SourceType::ts(),
        "jsx" => SourceType::jsx(),
        "mjs" => SourceType::mjs(),
        "cjs" => SourceType::cjs(),
        // Plain `.js` can contain JSX in many React Native projects, so parse as JSX.
        "js" => SourceType::jsx(),
        _ => SourceType::tsx(),
    }
}

/// Pre-computed line-start byte offsets so repeated lookups in one file are
/// cheap. Built once per parsed file.
pub struct LineIndex {
    /// Byte offset of the start of each line (line 0 starts at byte 0).
    line_starts: Vec<u32>,
}

impl LineIndex {
    /// Build a line index from source text.
    #[must_use]
    pub fn new(source: &str) -> LineIndex {
        let mut line_starts = vec![0u32];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                // The next line begins at the byte after the newline.
                line_starts.push(u32::try_from(offset + 1).unwrap_or(u32::MAX));
            }
        }
        LineIndex { line_starts }
    }

    /// Convert a byte offset to a 1-based (line, column) pair. Column is a
    /// count of UTF-8 bytes from the start of the line plus one, which matches
    /// editor "byte column" reporting closely enough for an auditor.
    #[must_use]
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        // Binary search for the greatest line start <= offset.
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert) => insert.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let column = offset.saturating_sub(line_start) + 1;
        (u32::try_from(line + 1).unwrap_or(u32::MAX), column)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_col_basic() {
        let source = "a\nbc\nd";
        let index = LineIndex::new(source);
        assert_eq!(index.line_col(0), (1, 1)); // 'a'
        assert_eq!(index.line_col(2), (2, 1)); // 'b'
        assert_eq!(index.line_col(3), (2, 2)); // 'c'
        assert_eq!(index.line_col(5), (3, 1)); // 'd'
    }

    #[test]
    fn source_type_selection() {
        assert!(source_type_for_extension("tsx").is_jsx());
        assert!(source_type_for_extension("ts").is_typescript());
        assert!(!source_type_for_extension("ts").is_jsx());
        assert!(source_type_for_extension("jsx").is_jsx());
    }
}
