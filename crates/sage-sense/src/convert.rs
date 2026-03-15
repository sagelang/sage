//! Conversion utilities for LSP types.

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

/// Convert a byte offset to an LSP Position (line/character).
/// LSP positions are zero-based, and character offsets are UTF-16 code units.
pub fn offset_to_position(offset: usize, source: &str) -> Position {
    let prefix = &source[..offset.min(source.len())];
    let line = prefix.matches('\n').count() as u32;
    let last_newline = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
    // LSP positions use UTF-16 code units
    let character = prefix[last_newline..].encode_utf16().count() as u32;
    Position { line, character }
}

/// Convert a byte range (start, end) to an LSP Range.
pub fn span_to_range(start: usize, end: usize, source: &str) -> Range {
    Range {
        start: offset_to_position(start, source),
        end: offset_to_position(end, source),
    }
}

/// Convert a LexErrorLocation to an LSP Diagnostic.
pub fn lex_error_to_diagnostic(
    error: &sage_parser::LexErrorLocation,
    source: &str,
) -> Diagnostic {
    Diagnostic {
        range: span_to_range(error.start, error.end, source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(tower_lsp::lsp_types::NumberOrString::String(
            "sage::lexer::E001".to_string(),
        )),
        source: Some("sage".to_string()),
        message: format!("invalid token `{}`", error.text),
        ..Default::default()
    }
}

/// Convert a sage-parser ParseError to an LSP Diagnostic.
pub fn parse_error_to_diagnostic(
    error: &sage_parser::ParseError,
    source: &str,
) -> Option<Diagnostic> {
    // chumsky's Simple error has a span
    let span = error.span();
    Some(Diagnostic {
        range: span_to_range(span.start, span.end, source),
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(tower_lsp::lsp_types::NumberOrString::String(
            "sage::parser::syntax".to_string(),
        )),
        source: Some("sage".to_string()),
        message: format!("{}", error),
        ..Default::default()
    })
}

/// Convert a sage-checker CheckError to an LSP Diagnostic.
pub fn check_error_to_diagnostic(
    error: &sage_checker::CheckError,
    source: &str,
) -> Option<Diagnostic> {
    use miette::Diagnostic as MietteDiagnostic;

    // Get the span from miette labels
    let labels: Vec<_> = error.labels().map(|l| l.collect()).unwrap_or_default();
    let span = labels.first()?;
    let offset = span.offset();
    let len = span.len();

    // Determine severity
    let severity = error
        .severity()
        .map(|s| match s {
            miette::Severity::Error => DiagnosticSeverity::ERROR,
            miette::Severity::Warning => DiagnosticSeverity::WARNING,
            miette::Severity::Advice => DiagnosticSeverity::HINT,
        })
        .unwrap_or(DiagnosticSeverity::ERROR);

    // Get error code
    let code = error
        .code()
        .map(|c| tower_lsp::lsp_types::NumberOrString::String(c.to_string()));

    Some(Diagnostic {
        range: span_to_range(offset, offset + len, source),
        severity: Some(severity),
        code,
        source: Some("sage".to_string()),
        message: format!("{}", error),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_position_simple() {
        let source = "hello\nworld";
        assert_eq!(offset_to_position(0, source), Position { line: 0, character: 0 });
        assert_eq!(offset_to_position(5, source), Position { line: 0, character: 5 });
        assert_eq!(offset_to_position(6, source), Position { line: 1, character: 0 });
        assert_eq!(offset_to_position(11, source), Position { line: 1, character: 5 });
    }

    #[test]
    fn test_offset_to_position_utf8() {
        // UTF-8 multibyte: "日本語" is 9 bytes but 3 UTF-16 code units
        let source = "日本語";
        // First char
        assert_eq!(offset_to_position(0, source), Position { line: 0, character: 0 });
        // After first char (3 bytes, 1 UTF-16 unit)
        assert_eq!(offset_to_position(3, source), Position { line: 0, character: 1 });
        // After second char
        assert_eq!(offset_to_position(6, source), Position { line: 0, character: 2 });
    }

    #[test]
    fn test_span_to_range() {
        let source = "hello\nworld";
        let range = span_to_range(0, 5, source);
        assert_eq!(range.start, Position { line: 0, character: 0 });
        assert_eq!(range.end, Position { line: 0, character: 5 });
    }
}
