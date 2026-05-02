use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::{error::RuntimeError, value::Valor};

#[derive(Debug, Clone)]
pub struct Entorno {
    valores: HashMap<String, (Valor, bool)>, // (value, is_duro)
    padre: Option<Rc<RefCell<Entorno>>>,
}

impl Entorno {
    pub fn nuevo() -> Self {
        Entorno { valores: HashMap::new(), padre: None }
    }

    pub fn nuevo_hijo(padre: Rc<RefCell<Entorno>>) -> Self {
        Entorno { valores: HashMap::new(), padre: Some(padre) }
    }

    pub fn definir(&mut self, nombre: &str, valor: Valor, es_duro: bool) {
        self.valores.insert(nombre.to_string(), (valor, es_duro));
    }

    pub fn asignar(&mut self, nombre: &str, valor: Valor) -> Result<(), RuntimeError> {
        if let Some((entry, es_duro)) = self.valores.get_mut(nombre) {
            if *es_duro {
                return Err(RuntimeError::ConstanteInmutable(nombre.to_string()));
            }
            *entry = valor;
            return Ok(());
        }
        if let Some(padre) = &self.padre {
            return padre.borrow_mut().asignar(nombre, valor);
        }
        Err(RuntimeError::VarNoDefinida(nombre.to_string()))
    }

    pub fn obtener(&self, nombre: &str) -> Option<Valor> {
        if let Some((val, _)) = self.valores.get(nombre) {
            return Some(val.clone());
        }
        if let Some(padre) = &self.padre {
            return padre.borrow().obtener(nombre);
        }
        None
    }
}