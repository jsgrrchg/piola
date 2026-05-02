use std::fmt;

#[derive(Debug)]
pub enum PiolaError {
    Lexico(String),
    Sintaxis(String),
    Runtime(String),
}

impl fmt::Display for PiolaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PiolaError::Lexico(msg) => write!(f, "Error léxico: {msg}"),
            PiolaError::Sintaxis(msg) => write!(f, "Error de sintaxis: {msg}"),
            PiolaError::Runtime(msg) => write!(f, "Error en tiempo de ejecución: {msg}"),
        }
    }
}