use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    process,
};

use clap::Parser;
use piola::{
    interpreter::Interprete,
    lexer::tokenizar,
    parser::parsear,
};

#[derive(Parser)]
#[command(name = "piola", about = "El intérprete del lenguaje Piola")]
struct Cli {
    /// Archivo fuente a ejecutar (.cl)
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    match cli.file {
        Some(path) => run_file(path),
        None => run_repl(),
    }
}

fn run_file(path: PathBuf) {
    if path.extension().and_then(|e| e.to_str()) != Some("cl") {
        eprintln!("error: se esperaba un archivo .cl");
        process::exit(1);
    }

    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error leyendo '{}': {e}", path.display());
            process::exit(1);
        }
    };

    let tokens = match tokenizar(&src) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let stmts = match parsear(tokens) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let mut interp = Interprete::nuevo();
    if let Err(e) = interp.correr(&stmts) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run_repl() {
    println!("Piola v0.1 — escribe 'chao' para salir");

    let stdin = io::stdin();
    let mut interp = Interprete::nuevo();

    loop {
        print!(">>> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("error: {e}");
                break;
            }
        }

        let trimmed = line.trim();
        if trimmed == "chao" || trimmed == "exit" || trimmed == "quit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }

        let tokens = match tokenizar(trimmed) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let stmts = match parsear(tokens) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        match interp.correr(&stmts) {
            Ok(val) => {
                use piola::interpreter::value::Valor;
                if !matches!(val, Valor::Nada) {
                    println!("{val}");
                }
            }
            Err(e) => eprintln!("{e}"),
        }
    }
}