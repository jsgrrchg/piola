use std::path::PathBuf;

use clap::Parser;
use miette::IntoDiagnostic;
use rustyline::{error::ReadlineError, DefaultEditor};

use piola::{
    error::PiolaError,
    interpreter::{value::Valor, Interprete},
    lexer::{tokenizar, Lexer},
    parser::parsear,
};

#[derive(Parser)]
#[command(name = "piola", about = "El intérprete del lenguaje Piola")]
struct Cli {
    /// Archivo fuente a ejecutar (.cl)
    file: Option<PathBuf>,
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    match cli.file {
        Some(path) => run_file(path),
        None => { run_repl(); Ok(()) }
    }
}

fn run_file(path: PathBuf) -> miette::Result<()> {
    if path.extension().and_then(|e| e.to_str()) != Some("cl") {
        miette::bail!("se esperaba un archivo .cl, encontré '{}'", path.display());
    }

    let src = std::fs::read_to_string(&path).into_diagnostic()?;

    let filename = path.to_string_lossy();

    let tokens = Lexer::new(&src).with_filename(&*filename).tokenizar()?;
    let stmts  = parsear(tokens, &src, &filename)?;

    let mut interp = Interprete::nuevo();
    let _ = interp.correr(&stmts)?;

    Ok(())
}

fn run_repl() {
    let mut rl = match DefaultEditor::new() {
        Ok(rl) => rl,
        Err(e) => { eprintln!("Error al inicializar el REPL: {e}"); return; }
    };

    let _ = rl.load_history(".piola_history");

    let mut interp = Interprete::nuevo();
    println!("Piola v0.1 — escribe 'chao' para salir");

    loop {
        match rl.readline(">>> ") {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }
                if matches!(trimmed, "chao" | "exit" | "quit") {
                    println!("¡Chao!");
                    break;
                }

                let _ = rl.add_history_entry(trimmed);

                let result = tokenizar(trimmed)
                    .and_then(|tokens| parsear(tokens, trimmed, "<repl>"))
                    .and_then(|stmts| interp.correr(&stmts));

                match result {
                    Ok(Valor::Nada) => {}
                    Ok(val)         => println!("{val}"),
                    Err(e)          => eprint_error(e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("(usa 'chao' para salir)");
            }
            Err(ReadlineError::Eof) => {
                println!("\n¡Chao!");
                break;
            }
            Err(e) => {
                eprintln!("Error del REPL: {e}");
                break;
            }
        }
    }

    let _ = rl.save_history(".piola_history");
}

fn eprint_error(err: PiolaError) {
    eprintln!("{:?}", miette::Report::new(err));
}