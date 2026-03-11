//! Source span types for error reporting.

use std::fmt;
use std::sync::Arc;

/// A span representing a range in source code.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Span {
    /// Byte offset of the start of the span.
    pub start: usize,
    /// Byte offset of the end of the span (exclusive).
    pub end: usize,
    /// The source text this span refers to.
    pub source: Arc<str>,
}

impl Span {
    /// Create a new span.
    #[must_use]
    pub fn new(start: usize, end: usize, source: Arc<str>) -> Self {
        Self { start, end, source }
    }

    /// Create a dummy span for testing or synthetic nodes.
    #[must_use]
    pub fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            source: Arc::from(""),
        }
    }

    /// Get the length of this span in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if this span is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the source text covered by this span.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.source[self.start..self.end]
    }

    /// Merge two spans into one that covers both.
    #[must_use]
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            source: Arc::clone(&self.source),
        }
    }

    /// Calculate line and column (1-indexed) for the start of this span.
    #[must_use]
    pub fn line_col(&self) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in self.source.char_indices() {
            if i >= self.start {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.line_col();
        let len = self.end - self.start;
        write!(f, "{line}:{col}..{len}")
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (line, col) = self.line_col();
        write!(f, "{line}:{col}")
    }
}

/// An identifier with its source span.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    /// The identifier name.
    pub name: String,
    /// The source span.
    pub span: Span,
}

impl Ident {
    /// Create a new identifier.
    #[must_use]
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    /// Create a dummy identifier for testing.
    #[must_use]
    pub fn dummy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            span: Span::dummy(),
        }
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ident({:?})", self.name)
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_text() {
        let source: Arc<str> = Arc::from("hello world");
        let span = Span::new(0, 5, Arc::clone(&source));
        assert_eq!(span.text(), "hello");
    }

    #[test]
    fn span_line_col() {
        let source: Arc<str> = Arc::from("line1\nline2\nline3");

        // Start of file
        let span = Span::new(0, 1, Arc::clone(&source));
        assert_eq!(span.line_col(), (1, 1));

        // Start of line 2
        let span = Span::new(6, 7, Arc::clone(&source));
        assert_eq!(span.line_col(), (2, 1));

        // Middle of line 3
        let span = Span::new(14, 15, Arc::clone(&source));
        assert_eq!(span.line_col(), (3, 3));
    }

    #[test]
    fn span_merge() {
        let source: Arc<str> = Arc::from("hello world");
        let span1 = Span::new(0, 5, Arc::clone(&source));
        let span2 = Span::new(6, 11, Arc::clone(&source));
        let merged = span1.merge(&span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 11);
    }

    #[test]
    fn ident_display() {
        let ident = Ident::dummy("foo");
        assert_eq!(format!("{ident}"), "foo");
    }
}
