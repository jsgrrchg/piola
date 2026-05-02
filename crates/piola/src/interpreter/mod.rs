pub mod env;
pub mod error;
pub mod value;

use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Write},
    rc::Rc,
};

use crate::{
    ast::{Expr, OpBin, OpUn, Stmt},
    error::PiolaError,
};

use env::Entorno;
use error::RuntimeError;
use value::Valor;

pub struct Interprete {
    global: Rc<RefCell<Entorno>>,
}

impl Interprete {
    pub fn nuevo() -> Self {
        let global = Rc::new(RefCell::new(Entorno::nuevo()));

        // Register built-in functions
        {
            let mut env = global.borrow_mut();

            env.definir("altiro", Valor::Nativa(builtin_altiro), false);
            env.definir("largo", Valor::Nativa(builtin_largo), false);
            env.definir("cachar", Valor::Nativa(builtin_cachar), false);
            env.definir("pregunta", Valor::Nativa(builtin_pregunta), false);
        }

        Interprete { global }
    }

    pub fn correr(&mut self, stmts: &[Stmt]) -> Result<Valor, PiolaError> {
        self.eval_stmts(stmts, Rc::clone(&self.global))
            .map_err(|e| PiolaError::Runtime(e.to_string()))
    }

    fn eval_stmts(
        &mut self,
        stmts: &[Stmt],
        env: Rc<RefCell<Entorno>>,
    ) -> Result<Valor, RuntimeError> {
        let mut last = Valor::Nada;
        for stmt in stmts {
            last = self.eval_stmt(stmt, Rc::clone(&env))?;
        }
        Ok(last)
    }

    fn eval_stmt(
        &mut self,
        stmt: &Stmt,
        env: Rc<RefCell<Entorno>>,
    ) -> Result<Valor, RuntimeError> {
        match stmt {
            Stmt::Expresion(expr) => self.eval_expr(expr, env),

            Stmt::DeclWea { nombre, valor, es_duro } => {
                let v = self.eval_expr(valor, Rc::clone(&env))?;
                env.borrow_mut().definir(nombre, v, *es_duro);
                Ok(Valor::Nada)
            }

            Stmt::DeclPega { nombre, params, cuerpo } => {
                let funcion = Valor::Funcion {
                    params: params.clone(),
                    cuerpo: cuerpo.clone(),
                    entorno: Rc::clone(&env),
                };
                env.borrow_mut().definir(nombre, funcion, false);
                Ok(Valor::Nada)
            }

            Stmt::Cachai { cond, entonces, si_no } => {
                let condicion = self.eval_expr(cond, Rc::clone(&env))?;
                if condicion.es_verdadero() {
                    let child = Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                    self.eval_stmts(entonces, child)
                } else if let Some(rama_no) = si_no {
                    let child = Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                    self.eval_stmts(rama_no, child)
                } else {
                    Ok(Valor::Nada)
                }
            }

            Stmt::Mientras { cond, cuerpo } => {
                loop {
                    let condicion = self.eval_expr(cond, Rc::clone(&env))?;
                    if !condicion.es_verdadero() {
                        break;
                    }
                    let child = Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                    self.eval_stmts(cuerpo, child)?;
                }
                Ok(Valor::Nada)
            }

            Stmt::Para { var, iterable, cuerpo } => {
                let coleccion = self.eval_expr(iterable, Rc::clone(&env))?;
                match coleccion {
                    Valor::Lista(items) => {
                        let items_snapshot: Vec<Valor> = items.borrow().clone();
                        for item in items_snapshot {
                            let child =
                                Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                            child.borrow_mut().definir(var, item, false);
                            self.eval_stmts(cuerpo, child)?;
                        }
                    }
                    Valor::Texto(s) => {
                        for ch in s.chars() {
                            let child =
                                Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                            child
                                .borrow_mut()
                                .definir(var, Valor::Texto(ch.to_string()), false);
                            self.eval_stmts(cuerpo, child)?;
                        }
                    }
                    other => {
                        return Err(RuntimeError::TipoInvalido(format!(
                            "No podí iterar sobre un '{}', solo listas y textos.",
                            other.tipo_nombre()
                        )));
                    }
                }
                Ok(Valor::Nada)
            }

            Stmt::Ojo { cuerpo, error_var, manejo } => {
                let child = Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                match self.eval_stmts(cuerpo, child) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        let msg = e.to_string();
                        let manejo_env =
                            Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&env))));
                        manejo_env
                            .borrow_mut()
                            .definir(error_var, Valor::Texto(msg), false);
                        self.eval_stmts(manejo, manejo_env)
                    }
                }
            }
        }
    }

    fn eval_expr(
        &mut self,
        expr: &Expr,
        env: Rc<RefCell<Entorno>>,
    ) -> Result<Valor, RuntimeError> {
        match expr {
            Expr::Numero(n) => Ok(Valor::Numero(*n)),
            Expr::Texto(s) => Ok(Valor::Texto(s.clone())),
            Expr::Booleano(b) => Ok(Valor::Booleano(*b)),
            Expr::Nada => Ok(Valor::Nada),

            Expr::Ident(nombre, _span) => {
                env.borrow().obtener(nombre).ok_or_else(|| {
                    RuntimeError::VarNoDefinida(nombre.clone())
                })
            }

            Expr::Asignacion { nombre, valor, .. } => {
                let v = self.eval_expr(valor, Rc::clone(&env))?;
                env.borrow_mut().asignar(nombre, v.clone())?;
                Ok(v)
            }

            Expr::Unario { op, expr, .. } => {
                let v = self.eval_expr(expr, env)?;
                match op {
                    OpUn::No => Ok(Valor::Booleano(!v.es_verdadero())),
                    OpUn::Neg => match v {
                        Valor::Numero(n) => Ok(Valor::Numero(-n)),
                        other => Err(RuntimeError::TipoInvalido(format!(
                            "No podí negar un '{}'.",
                            other.tipo_nombre()
                        ))),
                    },
                }
            }

            Expr::Binario { izq, op, der, .. } => {
                // Short-circuit logical operators
                match op {
                    OpBin::Y => {
                        let lhs = self.eval_expr(izq, Rc::clone(&env))?;
                        if !lhs.es_verdadero() {
                            return Ok(Valor::Booleano(false));
                        }
                        let rhs = self.eval_expr(der, env)?;
                        return Ok(Valor::Booleano(rhs.es_verdadero()));
                    }
                    OpBin::O => {
                        let lhs = self.eval_expr(izq, Rc::clone(&env))?;
                        if lhs.es_verdadero() {
                            return Ok(Valor::Booleano(true));
                        }
                        let rhs = self.eval_expr(der, env)?;
                        return Ok(Valor::Booleano(rhs.es_verdadero()));
                    }
                    _ => {}
                }

                let lhs = self.eval_expr(izq, Rc::clone(&env))?;
                let rhs = self.eval_expr(der, env)?;
                self.eval_binario(op, lhs, rhs)
            }

            Expr::Llamada { callee, args, .. } => {
                let func = self.eval_expr(callee, Rc::clone(&env))?;
                let mut evaluated_args = Vec::with_capacity(args.len());
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg, Rc::clone(&env))?);
                }
                self.llamar(func, evaluated_args)
            }

            Expr::Indice { objeto, indice, .. } => {
                let obj = self.eval_expr(objeto, Rc::clone(&env))?;
                let idx = self.eval_expr(indice, env)?;
                self.eval_indice(obj, idx)
            }

            Expr::Lista(items, _) => {
                let mut vals = Vec::with_capacity(items.len());
                for item in items {
                    vals.push(self.eval_expr(item, Rc::clone(&env))?);
                }
                Ok(Valor::Lista(Rc::new(RefCell::new(vals))))
            }

            Expr::Mapa(pairs, _) => {
                let mut map = HashMap::new();
                for (key_expr, val_expr) in pairs {
                    let key = self.eval_expr(key_expr, Rc::clone(&env))?;
                    let val = self.eval_expr(val_expr, Rc::clone(&env))?;
                    let key_str = match key {
                        Valor::Texto(s) => s,
                        other => other.to_string(),
                    };
                    map.insert(key_str, val);
                }
                Ok(Valor::Mapa(Rc::new(RefCell::new(map))))
            }
        }
    }

    fn eval_binario(
        &self,
        op: &OpBin,
        lhs: Valor,
        rhs: Valor,
    ) -> Result<Valor, RuntimeError> {
        match op {
            OpBin::Suma => match (&lhs, &rhs) {
                (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(a + b)),
                (Valor::Texto(a), Valor::Texto(b)) => Ok(Valor::Texto(format!("{a}{b}"))),
                (Valor::Texto(a), Valor::Numero(b)) => {
                    let b_str = if b.fract() == 0.0 {
                        format!("{}", *b as i64)
                    } else {
                        b.to_string()
                    };
                    Ok(Valor::Texto(format!("{a}{b_str}")))
                }
                (Valor::Numero(a), Valor::Texto(b)) => {
                    let a_str = if a.fract() == 0.0 {
                        format!("{}", *a as i64)
                    } else {
                        a.to_string()
                    };
                    Ok(Valor::Texto(format!("{a_str}{b}")))
                }
                _ => Err(RuntimeError::TipoInvalido(format!(
                    "No podi sumar un '{}' con un '{}' pedazo de saco wea.",
                    lhs.tipo_nombre(),
                    rhs.tipo_nombre()
                ))),
            },
            OpBin::Resta => numeric_op(lhs, rhs, "-", |a, b| a - b),
            OpBin::Mul => numeric_op(lhs, rhs, "*", |a, b| a * b),
            OpBin::Div => {
                match (&lhs, &rhs) {
                    (Valor::Numero(_), Valor::Numero(b)) if *b == 0.0 => {
                        Err(RuntimeError::DivisionPorCero)
                    }
                    _ => numeric_op(lhs, rhs, "/", |a, b| a / b),
                }
            }
            OpBin::Mod => {
                match (&lhs, &rhs) {
                    (Valor::Numero(_), Valor::Numero(b)) if *b == 0.0 => {
                        Err(RuntimeError::DivisionPorCero)
                    }
                    _ => numeric_op(lhs, rhs, "%", |a, b| a % b),
                }
            }
            OpBin::Eq => Ok(Valor::Booleano(lhs == rhs)),
            OpBin::Neq => Ok(Valor::Booleano(lhs != rhs)),
            OpBin::Lt => compare_op(lhs, rhs, "<", |a, b| a < b),
            OpBin::Gt => compare_op(lhs, rhs, ">", |a, b| a > b),
            OpBin::Lte => compare_op(lhs, rhs, "<=", |a, b| a <= b),
            OpBin::Gte => compare_op(lhs, rhs, ">=", |a, b| a >= b),
            // Y and O are handled with short-circuit above
            OpBin::Y | OpBin::O => unreachable!(),
        }
    }

    fn llamar(&mut self, func: Valor, args: Vec<Valor>) -> Result<Valor, RuntimeError> {
        match func {
            Valor::Nativa(f) => f(args),
            Valor::Funcion { params, cuerpo, entorno } => {
                if args.len() != params.len() {
                    return Err(RuntimeError::NumArgInvalido(params.len(), args.len()));
                }
                let call_env = Rc::new(RefCell::new(Entorno::nuevo_hijo(Rc::clone(&entorno))));
                for (param, arg) in params.iter().zip(args) {
                    call_env.borrow_mut().definir(param, arg, false);
                }
                self.eval_stmts(&cuerpo, call_env)
            }
            other => Err(RuntimeError::NoLlamable(other.tipo_nombre().to_string())),
        }
    }

    fn eval_indice(&self, obj: Valor, idx: Valor) -> Result<Valor, RuntimeError> {
        match obj {
            Valor::Lista(items) => {
                let items = items.borrow();
                let i = match &idx {
                    Valor::Numero(n) => *n as i64,
                    other => {
                        return Err(RuntimeError::TipoInvalido(format!(
                            "Los índices de lista deben ser números, no '{}'.",
                            other.tipo_nombre()
                        )));
                    }
                };
                let len = items.len();
                let real_idx = if i < 0 {
                    let pos = len as i64 + i;
                    if pos < 0 {
                        return Err(RuntimeError::IndiceInvalido(i, len));
                    }
                    pos as usize
                } else {
                    i as usize
                };
                items.get(real_idx).cloned().ok_or(RuntimeError::IndiceInvalido(i, len))
            }
            Valor::Mapa(map) => {
                let map = map.borrow();
                let key = match idx {
                    Valor::Texto(s) => s,
                    other => other.to_string(),
                };
                map.get(&key).cloned().ok_or(RuntimeError::ClaveInexistente(key))
            }
            Valor::Texto(s) => {
                let i = match &idx {
                    Valor::Numero(n) => *n as i64,
                    other => {
                        return Err(RuntimeError::TipoInvalido(format!(
                            "Los índices de texto deben ser números, no '{}'.",
                            other.tipo_nombre()
                        )));
                    }
                };
                let chars: Vec<char> = s.chars().collect();
                let len = chars.len();
                let real_idx = if i < 0 {
                    let pos = len as i64 + i;
                    if pos < 0 {
                        return Err(RuntimeError::IndiceInvalido(i, len));
                    }
                    pos as usize
                } else {
                    i as usize
                };
                chars
                    .get(real_idx)
                    .map(|c| Valor::Texto(c.to_string()))
                    .ok_or(RuntimeError::IndiceInvalido(i, len))
            }
            other => Err(RuntimeError::TipoInvalido(format!(
                "No podí indexar un '{}'.",
                other.tipo_nombre()
            ))),
        }
    }
}

// ── Helper functions ──────────────────────────────────────────────────────────

fn numeric_op(
    lhs: Valor,
    rhs: Valor,
    op_sym: &str,
    f: impl Fn(f64, f64) -> f64,
) -> Result<Valor, RuntimeError> {
    match (lhs, rhs) {
        (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Numero(f(a, b))),
        (l, r) => Err(RuntimeError::TipoInvalido(format!(
            "No podi usar '{op_sym}' entre un '{}' y un '{}' pedazo de saco wea.",
            l.tipo_nombre(),
            r.tipo_nombre()
        ))),
    }
}

fn compare_op(
    lhs: Valor,
    rhs: Valor,
    op_sym: &str,
    f: impl Fn(f64, f64) -> bool,
) -> Result<Valor, RuntimeError> {
    match (lhs, rhs) {
        (Valor::Numero(a), Valor::Numero(b)) => Ok(Valor::Booleano(f(a, b))),
        (Valor::Texto(a), Valor::Texto(b)) => {
            let result = match op_sym {
                "<" => a < b,
                ">" => a > b,
                "<=" => a <= b,
                ">=" => a >= b,
                _ => unreachable!(),
            };
            Ok(Valor::Booleano(result))
        }
        (l, r) => Err(RuntimeError::TipoInvalido(format!(
            "No podi comparar un '{}' con un '{}' pedazo de saco wea.",
            l.tipo_nombre(),
            r.tipo_nombre()
        ))),
    }
}

// ── Built-in functions ───────────────────────────────────────────────────────

fn builtin_altiro(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    let parts: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", parts.join(" "));
    Ok(Valor::Nada)
}

fn builtin_largo(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::NumArgInvalido(1, args.len()));
    }
    match &args[0] {
        Valor::Texto(s) => Ok(Valor::Numero(s.chars().count() as f64)),
        Valor::Lista(items) => Ok(Valor::Numero(items.borrow().len() as f64)),
        other => Err(RuntimeError::TipoInvalido(format!(
            "largo() solo funciona con texto o lista, no con '{}'.",
            other.tipo_nombre()
        ))),
    }
}

fn builtin_cachar(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::NumArgInvalido(1, args.len()));
    }
    Ok(Valor::Texto(args[0].tipo_nombre().to_string()))
}

fn builtin_pregunta(args: Vec<Valor>) -> Result<Valor, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::NumArgInvalido(1, args.len()));
    }
    print!("{}", args[0]);
    io::stdout().flush().ok();
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        RuntimeError::TipoInvalido(format!("Error leyendo input: {e}"))
    })?;
    Ok(Valor::Texto(input.trim_end_matches('\n').trim_end_matches('\r').to_string()))
}