//! Command-line interface for the Sage language.

use clap::{Parser, Subcommand};
use miette::{Diagnostic, IntoDiagnostic, Result, Severity, WrapErr};
use sage_checker::check;
use sage_interpreter::{Runtime, RuntimeConfig};
use sage_lexer::lex;
use sage_parser::parse;
use std::path::PathBuf;
use std::sync::Arc;

/// Sage - A programming language where agents are first-class citizens.
#[derive(Parser)]
#[command(name = "sage")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Sage program
    Run {
        /// Path to the .sg file to run
        file: PathBuf,

        /// Use mock LLM (for testing without API key)
        #[arg(long)]
        mock: bool,
    },

    /// Check a Sage program for errors without running it
    Check {
        /// Path to the .sg file to check
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, mock } => run_file(&file, mock).await,
        Commands::Check { file } => check_file(&file),
    }
}

/// Run a Sage program file.
async fn run_file(path: &PathBuf, mock: bool) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))?;

    let filename = path
        .file_name()
        .map_or_else(|| "unknown".to_string(), |s| s.to_string_lossy().into_owned());

    // Lex
    let lex_result = match lex(&source) {
        Ok(result) => result,
        Err(err) => {
            let report = miette::Report::new(err).with_source_code(source);
            return Err(report);
        }
    };

    // Parse
    let source_arc: Arc<str> = Arc::from(source.as_str());
    let (program, parse_errors) = parse(lex_result.tokens(), Arc::clone(&source_arc));

    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("Parse error: {err}");
        }
        miette::bail!("Parse errors in {filename}");
    }

    let program = program.ok_or_else(|| miette::miette!("Failed to parse program"))?;

    // Type check
    let check_result = check(&program);
    let mut has_errors = false;
    for err in &check_result.errors {
        let report = miette::Report::new(err.clone()).with_source_code(source.clone());
        eprintln!("{report:?}");
        // Only count actual errors, not warnings
        if err.severity().unwrap_or(Severity::Error) == Severity::Error {
            has_errors = true;
        }
    }
    if has_errors {
        miette::bail!("Type errors in {filename}");
    }

    // Run
    let runtime = if mock {
        Runtime::mock()
    } else {
        Runtime::new(RuntimeConfig::default())
    };

    let result = runtime
        .run(program)
        .await
        .map_err(|e| miette::Report::new(e).with_source_code(source))?;

    // Print result if it's not Unit
    if !result.is_unit() {
        println!("{result}");
    }

    Ok(())
}

/// Check a Sage program file without running it.
fn check_file(path: &PathBuf) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .into_diagnostic()
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))?;

    let filename = path
        .file_name()
        .map_or_else(|| "unknown".to_string(), |s| s.to_string_lossy().into_owned());

    // Lex
    let lex_result = match lex(&source) {
        Ok(result) => result,
        Err(err) => {
            let report = miette::Report::new(err).with_source_code(source);
            return Err(report);
        }
    };

    // Parse
    let source_arc: Arc<str> = Arc::from(source.as_str());
    let (program, parse_errors) = parse(lex_result.tokens(), Arc::clone(&source_arc));

    let mut has_errors = false;

    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("Parse error: {err}");
        }
        has_errors = true;
    }

    if let Some(program) = program {
        // Type check
        let check_result = check(&program);
        for err in &check_result.errors {
            let report = miette::Report::new(err.clone()).with_source_code(source.clone());
            eprintln!("{report:?}");
            // Only count actual errors, not warnings
            if err.severity().unwrap_or(Severity::Error) == Severity::Error {
                has_errors = true;
            }
        }
    }

    if has_errors {
        miette::bail!("Errors found in {filename}");
    }

    println!("No errors found in {filename}");
    Ok(())
}
