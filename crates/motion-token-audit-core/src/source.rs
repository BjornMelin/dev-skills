//! Source-position helpers and oxc `SourceType` selection.

use oxc_span::SourceType;

#[must_use]
pub fn source_type_for_extension(extension: &str) -> SourceType {
    match extension.to_ascii_lowercase().as_str() {
        "tsx" => SourceType::tsx(),
        "ts" | "mts" | "cts" => SourceType::ts(),
        "jsx" => SourceType::jsx(),
        "mjs" => SourceType::mjs(),
        "cjs" => SourceType::cjs(),
        "js" => SourceType::jsx(),
        _ => SourceType::tsx(),
    }
}

pub struct LineIndex {
    line_starts: Vec<u32>,
}

impl LineIndex {
    #[must_use]
    pub fn new(source: &str) -> LineIndex {
        let mut line_starts = vec![0u32];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(u32::try_from(offset + 1).unwrap_or(u32::MAX));
            }
        }
        LineIndex { line_starts }
    }

    #[must_use]
    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let line = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert) => insert.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let column = offset.saturating_sub(line_start) + 1;
        (u32::try_from(line + 1).unwrap_or(u32::MAX), column)
    }
}
