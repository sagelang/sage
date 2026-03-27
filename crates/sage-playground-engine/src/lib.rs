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
            .map(sage_parser::format_error)
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

    // Guard: if the parser recovered from errors but produced an empty program
    // (no run statement, no agents), give a helpful error instead of the
    // confusing "No 'run' entry point" runtime message.
    if program.run_agent.is_none() && program.agents.is_empty() {
        return RunResult {
            output: vec![],
            result: String::new(),
            error: "No agents found. Every Sage program needs at least one agent \
                    with `run AgentName` at the end.\n\
                    Example:\n  \
                    agent Main {\n    \
                      on start {\n      \
                        yield(42);\n    \
                      }\n  \
                    }\n  \
                    run Main"
                .to_string(),
            success: false,
        };
    }

    // Guard: `run Agent { beliefs };` is not valid — catch this common mistake.
    // The parser's `run_stmt` only accepts `run Name;`, so `run Name { ... };`
    // silently fails (the `.or_not()` swallows the error), leaving run_agent
    // as None even though agents were parsed successfully.
    if program.run_agent.is_none() && !program.agents.is_empty() {
        // Check if the source contains a `run` followed by `{` (belief init attempt)
        let has_run_with_braces = {
            let trimmed = source.trim();
            // Look for `run <name> {` pattern in the source
            let mut found = false;
            for line in trimmed.lines() {
                let line = line.trim();
                if line.starts_with("run ") {
                    let after_run = line["run ".len()..].trim();
                    if after_run.contains('{') {
                        found = true;
                        break;
                    }
                }
            }
            found
        };

        let error = if has_run_with_braces {
            "Oswyn suggests: `run AgentName { ... }` is not valid syntax. \
             You cannot pass beliefs to `run`. Use `run AgentName;` instead, \
             and initialise values with `let` inside `on start`.\n\
             Example:\n  \
             agent Main {\n    \
               on start {\n      \
                 let max = 6;  // use let, not beliefs\n      \
                 yield(0);\n    \
               }\n  \
             }\n  \
             run Main;"
                .to_string()
        } else {
            "Missing `run` statement. Add `run AgentName;` at the end of your \
             program to specify the entry point.\n\
             Example: run Main;"
                .to_string()
        };

        return RunResult {
            output: vec![],
            result: String::new(),
            error,
            success: false,
        };
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Run source through the full playground pipeline (lex → parse → interpret)
    /// and return (success, output_lines, error_string).
    fn run(source: &str) -> (bool, Vec<String>, String) {
        let r = run_sage(source);
        (r.success, r.output, r.error)
    }

    #[test]
    fn hangman_code_runs() {
        let source = r#"
fn display_word(word: String, guessed: List<String>) -> String {
    let letters = split(word, "");
    let result: List<String> = [];
    for ch in letters {
        if contains(guessed, ch) {
            result = push(result, ch);
        } else {
            result = push(result, "_");
        }
    }
    return join(result, " ");
}

fn check_win(word: String, guessed: List<String>) -> Bool {
    let letters = split(word, "");
    for ch in letters {
        if !contains(guessed, ch) {
            return false;
        }
    }
    return true;
}

agent Hangman {
    on start {
        let word = "sage";
        let guessed: List<String> = ["s", "a"];
        print(display_word(word, guessed));
        yield(0);
    }
}

run Hangman;
"#;
        let (success, output, error) = run(source);
        eprintln!("success: {success}");
        eprintln!("output: {output:?}");
        eprintln!("error: {error}");
        assert!(success, "should succeed, got error: {error}");
        assert!(!output.is_empty(), "should have print output");
    }

    #[test]
    fn infinite_loop_caught() {
        let source = r#"
agent Main {
    on start {
        while true {
            print("looping");
        }
        yield(0);
    }
}
run Main;
"#;
        let (success, _output, error) = run(source);
        assert!(!success, "should fail on infinite loop");
        assert!(
            error.contains("Execution limit"),
            "should mention execution limit, got: {error}"
        );
    }

    #[test]
    fn run_with_beliefs_gives_helpful_error() {
        let source = r#"
agent Hangman {
    max_attempts: Int

    on start {
        yield(0);
    }
}

run Hangman { max_attempts: 6 };
"#;
        let (success, _output, error) = run(source);
        assert!(!success, "should fail for run with beliefs");
        assert!(
            error.contains("run AgentName;"),
            "should suggest correct run syntax, got: {error}"
        );
    }

    #[test]
    fn missing_run_gives_helpful_error() {
        let source = r#"
agent Main {
    on start {
        yield(0);
    }
}
"#;
        let (success, _output, error) = run(source);
        assert!(!success, "should fail without run statement");
        assert!(
            error.contains("Missing `run`"),
            "should mention missing run, got: {error}"
        );
    }
}
