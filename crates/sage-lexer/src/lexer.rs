//! Lexer implementation and public API.

use crate::Token;
use logos::Logos;
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use std::sync::Arc;
use thiserror::Error;

/// A token with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    /// The token.
    pub token: Token,
    /// Start byte offset.
    pub start: usize,
    /// End byte offset (exclusive).
    pub end: usize,
}

impl Spanned {
    /// Create a new spanned token.
    #[must_use]
    pub fn new(token: Token, start: usize, end: usize) -> Self {
        Self { token, start, end }
    }

    /// Get the length of this token in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if this token span is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the span as a tuple.
    #[must_use]
    pub fn span(&self) -> (usize, usize) {
        (self.start, self.end)
    }
}

/// A single lexer error at a specific location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexErrorLocation {
    /// Start byte offset of the invalid token.
    pub start: usize,
    /// End byte offset of the invalid token.
    pub end: usize,
    /// The invalid text that couldn't be lexed.
    pub text: String,
}

/// Error type for lexer failures.
#[derive(Error, Debug)]
#[error("failed to lex source: {count} error(s) found")]
pub struct LexError {
    /// The source code being lexed.
    source_code: NamedSource<String>,

    /// All error locations found during lexing.
    pub errors: Vec<LexErrorLocation>,

    /// Number of errors for the error message.
    count: usize,
}

impl LexError {
    /// Create a new lex error.
    fn new(source: String, errors: Vec<LexErrorLocation>) -> Self {
        let count = errors.len();
        Self {
            source_code: NamedSource::new("<input>", source),
            errors,
            count,
        }
    }

    /// Create a new lex error with a filename.
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        let source = self.source_code.inner().clone();
        self.source_code = NamedSource::new(filename.into(), source);
        self
    }
}

impl Diagnostic for LexError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("sage::lexer::E001"))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("remove or replace invalid characters"))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source_code)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let labels = self.errors.iter().map(|e| {
            LabeledSpan::new_with_span(
                Some(format!("invalid token `{}`", e.text)),
                (e.start, e.end - e.start),
            )
        });
        Some(Box::new(labels))
    }
}

/// Result of lexing source code.
#[derive(Debug)]
pub struct LexResult {
    /// Successfully lexed tokens.
    pub tokens: Vec<Spanned>,
    /// The source code (for error reporting).
    pub source: Arc<str>,
}

impl LexResult {
    /// Get the tokens as a slice.
    #[must_use]
    pub fn tokens(&self) -> &[Spanned] {
        &self.tokens
    }

    /// Consume self and return just the tokens.
    #[must_use]
    pub fn into_tokens(self) -> Vec<Spanned> {
        self.tokens
    }
}

/// Lex source code into tokens.
///
/// This function tokenizes the entire source, collecting all errors rather than
/// stopping at the first one. If any errors are found, they are all reported.
///
/// # Arguments
///
/// * `source` - The source code to lex.
///
/// # Returns
///
/// * `Ok(LexResult)` - Successfully lexed tokens with source reference.
/// * `Err(LexError)` - One or more lex errors occurred.
///
/// # Errors
///
/// Returns `LexError` if the source contains invalid characters that cannot
/// be tokenized. All errors are collected and reported together.
///
/// # Example
///
/// ```
/// use sage_lexer::{lex, Token};
///
/// let result = lex("let x = 42").unwrap();
/// assert_eq!(result.tokens()[0].token, Token::KwLet);
/// ```
pub fn lex(source: &str) -> Result<LexResult, LexError> {
    let source_arc: Arc<str> = Arc::from(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    let lexer = Token::lexer(source);

    for (result, span) in lexer.spanned() {
        if let Ok(token) = result {
            tokens.push(Spanned::new(token, span.start, span.end));
        } else {
            let text = source[span.start..span.end].to_string();
            errors.push(LexErrorLocation {
                start: span.start,
                end: span.end,
                text,
            });
        }
    }

    if errors.is_empty() {
        Ok(LexResult {
            tokens,
            source: source_arc,
        })
    } else {
        Err(LexError::new(source.to_string(), errors))
    }
}

/// Lex source code, returning tokens even if there are errors.
///
/// This is useful for editor tooling where you want partial results.
/// Errors are collected but don't prevent returning valid tokens.
///
/// # Returns
///
/// A tuple of (tokens, errors). The tokens vector contains all valid tokens
/// found, and the errors vector contains all lex errors encountered.
#[must_use]
pub fn lex_partial(source: &str) -> (Vec<Spanned>, Vec<LexErrorLocation>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    let lexer = Token::lexer(source);

    for (result, span) in lexer.spanned() {
        if let Ok(token) = result {
            tokens.push(Spanned::new(token, span.start, span.end));
        } else {
            let text = source[span.start..span.end].to_string();
            errors.push(LexErrorLocation {
                start: span.start,
                end: span.end,
                text,
            });
        }
    }

    (tokens, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_tokens() {
        let result = lex("let x = 42").unwrap();
        let tokens = result.tokens();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token, Token::KwLet);
        assert_eq!(tokens[1].token, Token::Ident);
        assert_eq!(tokens[2].token, Token::Eq);
        assert_eq!(tokens[3].token, Token::IntLit);
    }

    #[test]
    fn lex_preserves_spans() {
        let result = lex("let x").unwrap();
        let tokens = result.tokens();

        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 3); // "let"
        assert_eq!(tokens[1].start, 4);
        assert_eq!(tokens[1].end, 5); // "x"
    }

    #[test]
    fn lex_empty_source() {
        let result = lex("").unwrap();
        assert!(result.tokens().is_empty());
    }

    #[test]
    fn lex_whitespace_only() {
        let result = lex("   \n\t  ").unwrap();
        assert!(result.tokens().is_empty());
    }

    #[test]
    fn lex_comments_only() {
        let result = lex("// this is a comment").unwrap();
        assert!(result.tokens().is_empty());
    }

    #[test]
    fn lex_error_invalid_char() {
        let err = lex("let # = 42").unwrap_err();

        assert_eq!(err.errors.len(), 1);
        assert_eq!(err.errors[0].text, "#");
        assert_eq!(err.errors[0].start, 4);
        assert_eq!(err.errors[0].end, 5);
    }

    #[test]
    fn lex_error_multiple_invalid() {
        let err = lex("let # x $ y").unwrap_err();

        assert_eq!(err.errors.len(), 2);
        assert_eq!(err.errors[0].text, "#");
        assert_eq!(err.errors[1].text, "$");
    }

    #[test]
    fn lex_partial_with_errors() {
        let (tokens, errors) = lex_partial("let # x = 42");

        // Should have valid tokens
        assert_eq!(tokens.len(), 4); // let, x, =, 42
        assert_eq!(tokens[0].token, Token::KwLet);

        // And the error
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, "#");
    }

    #[test]
    fn lex_agent_declaration() {
        let source = r#"
agent Researcher {
    belief topic: String

    on start {
        let result = infer("test")
        emit(result)
    }
}

run Researcher
"#;
        let result = lex(source).unwrap();
        let tokens = result.tokens();

        // Verify key tokens are present
        assert_eq!(tokens[0].token, Token::KwAgent);
        assert_eq!(tokens[1].token, Token::Ident);
        assert_eq!(tokens[2].token, Token::LBrace);
        assert_eq!(tokens[3].token, Token::KwBelief);
    }

    #[test]
    fn lex_result_into_tokens() {
        let result = lex("42").unwrap();
        let tokens = result.into_tokens();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::IntLit);
    }

    #[test]
    fn spanned_len() {
        let spanned = Spanned::new(Token::KwLet, 0, 3);
        assert_eq!(spanned.len(), 3);
        assert!(!spanned.is_empty());
    }

    #[test]
    fn spanned_span() {
        let spanned = Spanned::new(Token::KwLet, 5, 8);
        assert_eq!(spanned.span(), (5, 8));
    }

    #[test]
    fn lex_error_display() {
        let err = lex("#").unwrap_err();
        let display = format!("{err}");
        assert!(display.contains("failed to lex"));
    }

    #[test]
    fn lex_string_interpolation_markers() {
        // String literals with {ident} patterns should lex as single string tokens
        let result = lex(r#""Hello {name}!""#).unwrap();
        let tokens = result.tokens();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::StringLit);
    }

    #[test]
    fn lex_complex_expression() {
        let result = lex("a + b * c == d && e || !f").unwrap();
        let tokens = result.tokens();

        assert_eq!(tokens.len(), 12);
        assert_eq!(tokens[0].token, Token::Ident); // a
        assert_eq!(tokens[1].token, Token::Plus); // +
        assert_eq!(tokens[2].token, Token::Ident); // b
        assert_eq!(tokens[3].token, Token::Star); // *
        assert_eq!(tokens[4].token, Token::Ident); // c
        assert_eq!(tokens[5].token, Token::EqEq); // ==
        assert_eq!(tokens[6].token, Token::Ident); // d
        assert_eq!(tokens[7].token, Token::And); // &&
        assert_eq!(tokens[8].token, Token::Ident); // e
        assert_eq!(tokens[9].token, Token::Or); // ||
        assert_eq!(tokens[10].token, Token::Bang); // !
        assert_eq!(tokens[11].token, Token::Ident); // f
    }

    #[test]
    fn lex_list_type() {
        let result = lex("List<String>").unwrap();
        let tokens = result.tokens();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].token, Token::TyList);
        assert_eq!(tokens[1].token, Token::Lt);
        assert_eq!(tokens[2].token, Token::TyString);
        assert_eq!(tokens[3].token, Token::Gt);
    }

    #[test]
    fn lex_error_with_filename() {
        let err = lex("#").unwrap_err().with_filename("test.sg");
        // The error should still work
        assert_eq!(err.errors.len(), 1);
    }

    #[test]
    fn lex_error_is_diagnostic() {
        use miette::Diagnostic;

        let err = lex("#").unwrap_err();

        // Check that Diagnostic trait methods work
        assert!(err.code().is_some());
        assert!(err.help().is_some());
        assert!(err.source_code().is_some());
        assert!(err.labels().is_some());
    }
}
