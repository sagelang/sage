//! Sage Playground Engine — parse and interpret Sage programs in the browser.
//!
//! Compiles the Sage parser and a tree-walking interpreter to WebAssembly,
//! exposing a `run_sage(source)` function via wasm-bindgen.

#![forbid(unsafe_code)]

mod interp;

use interp::Interpreter;
use wasm_bindgen::prelude::*;

/// Result of running a Sage program.
#[wasm_bindgen]
pub struct RunResult {
    /// Output lines (from print, trace, etc.)
    output: Vec<String>,
    /// The yield value (or empty if none).
    result: String,
    /// Error message (or empty if success).
    error: String,
    /// Whether execution succeeded.
    success: bool,
}

#[wasm_bindgen]
impl RunResult {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> Vec<String> {
        self.output.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn result(&self) -> String {
        self.result.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> String {
        self.error.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }
}

/// Initialize the playground engine (call once on page load).
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Run a Sage program and return the output.
///
/// Parses the source, finds the entry agent, and interprets it.
/// Returns a `RunResult` with output lines, the yield value, and any error.
#[wasm_bindgen]
pub fn run_sage(source: &str) -> RunResult {
    // Step 1: Lex
    let lex_result = match sage_parser::lex(source) {
        Ok(result) => result,
        Err(lex_error) => {
            let msg = lex_error
                .errors
                .iter()
                .map(|e| format!("unexpected '{}' at position {}", e.text, e.start))
                .collect::<Vec<_>>()
                .join("; ");
            return RunResult {
                output: vec![],
                result: String::new(),
                error: format!("Lex error: {msg}"),
                success: false,
            };
        }
    };

    // Step 2: Parse
    let source_arc: std::sync::Arc<str> = std::sync::Arc::from(source);
    let (program, parse_errors) = sage_parser::parse(lex_result.tokens(), source_arc);

    if !parse_errors.is_empty() {
        let msg = parse_errors
            .iter()
            .map(|e| format!("{e}"))
            .collect::<Vec<_>>()
            .join("\n");
        return RunResult {
            output: vec![],
            result: String::new(),
            error: format!("Parse error:\n{msg}"),
            success: false,
        };
    }

    let program = match program {
        Some(p) => p,
        None => {
            return RunResult {
                output: vec![],
                result: String::new(),
                error: "Failed to parse program".to_string(),
                success: false,
            }
        }
    };

    // Step 3: Interpret
    let mut interp = Interpreter::new();
    match interp.run(&program) {
        Ok(val) => RunResult {
            output: interp.output,
            result: val.to_display(),
            error: String::new(),
            success: true,
        },
        Err(e) => RunResult {
            output: interp.output,
            result: String::new(),
            error: e.to_string(),
            success: false,
        },
    }
}
