mod lexer;
mod parser;
mod typechecker;
mod interpreter;
mod error;

use std::fs;
use std::io::{self, Write};

use clap::{Parser as ClapParser, Subcommand};
use colored::Colorize;

use lexer::scanner::Scanner;
use parser::parser::Parser;
use interpreter::runtime::Runtime;
use interpreter::value::LingotValue;

#[derive(ClapParser)]
#[command(name = "lingot", version, about = "A concise, safe, and readable scripting language")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a .ling file
    Run {
        /// Path to the .ling file
        file: String,
    },
    /// Start the REPL
    Repl,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file } => run_file(&file),
        Commands::Repl => run_repl(),
    }
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            error::report::report_runtime_error(&format!("cannot read '{}': {}", path, e));
            std::process::exit(1);
        }
    };

    if let Err(e) = execute(&source) {
        error::report::report_runtime_error(&e);
        std::process::exit(1);
    }
}

fn run_repl() {
    println!("{} v{}", "lingot".bold().cyan(), env!("CARGO_PKG_VERSION"));
    println!("Type expressions to evaluate. Ctrl+C to exit.\n");

    let mut runtime = Runtime::new();

    loop {
        print!("{} ", ">>".green().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        match execute_with_runtime(&mut runtime, input) {
            Ok(Some(val)) => {
                match &val {
                    LingotValue::Void => {}
                    _ => println!("{}", val),
                }
            }
            Ok(None) => {}
            Err(e) => {
                error::report::report_runtime_error(&e);
            }
        }
    }
}

fn execute(source: &str) -> Result<(), String> {
    let mut runtime = Runtime::new();
    execute_with_runtime(&mut runtime, source)?;
    Ok(())
}

fn execute_with_runtime(
    runtime: &mut Runtime,
    source: &str,
) -> Result<Option<LingotValue>, String> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens().map_err(|e| e.to_string())?;

    let mut parser = Parser::new(tokens);
    let stmts = parser.parse().map_err(|e| e.to_string())?;

    let result = runtime.execute(&stmts)?;

    if result.ok {
        match result.value {
            LingotValue::Void => Ok(None),
            val => Ok(Some(val)),
        }
    } else {
        Err(result.error.unwrap_or_else(|| "unknown error".to_string()))
    }
}
