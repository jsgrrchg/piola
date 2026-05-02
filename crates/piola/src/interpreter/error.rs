use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    TipoInvalido(String),
    VarNoDefinida(String),
    IndiceInvalido(i64, usize),
    ClaveInexistente(String),
    ConstanteInmutable(String),
    DivisionPorCero,
    NumArgInvalido(usize, usize),
    NoLlamable(String),
    /// Used internally for ojo/cago — carries the error message as a string value
    ErrorCatcheable(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TipoInvalido(msg) => write!(f, "{msg}"),
            RuntimeError::VarNoDefinida(name) => {
                write!(f, "La wea '{name}' no existe papito.")
            }
            RuntimeError::IndiceInvalido(idx, len) => {
                write!(
                    f,
                    "Te fuiste al chancho, el índice {idx} no existe en la lista (largo: {len})."
                )
            }
            RuntimeError::ClaveInexistente(key) => {
                write!(f, "La clave '{key}' no existe papito.")
            }
            RuntimeError::ConstanteInmutable(name) => {
                write!(f, "Oe, '{name}' es duro, no lo podí cambiar.")
            }
            RuntimeError::DivisionPorCero => {
                write!(f, "Weon, no se puede dividir por cero.")
            }
            RuntimeError::NumArgInvalido(exp, got) => {
                write!(f, "La pega espera {exp} args, le pasaste {got}.")
            }
            RuntimeError::NoLlamable(tipo) => {
                write!(f, "'{tipo}' no es una pega papito.")
            }
            RuntimeError::ErrorCatcheable(msg) => write!(f, "{msg}"),
        }
    }
}