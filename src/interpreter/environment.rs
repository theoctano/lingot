use std::collections::HashMap;

use crate::interpreter::value::LingotValue;

#[derive(Debug, Clone)]
struct Variable {
    value: LingotValue,
    is_dyn: bool,
    is_pub: bool,
}

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Variable>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(
        &mut self,
        name: &str,
        value: LingotValue,
        is_dyn: bool,
        is_pub: bool,
    ) -> Result<(), String> {
        let scope = self.scopes.last_mut()
            .ok_or_else(|| "internal error: no scope available".to_string())?;
        if scope.contains_key(name) {
            return Err(format!("'{}' is already defined in this scope", name));
        }
        scope.insert(name.to_string(), Variable { value, is_dyn, is_pub });
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<&LingotValue, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(&var.value);
            }
        }
        Err(format!("'{}' is not defined", name))
    }

    pub fn assign(&mut self, name: &str, value: LingotValue) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.get_mut(name) {
                if !var.is_dyn {
                    return Err(format!(
                        "'{}' is not dynamic, cannot reassign. Use 'let dyn' to make it reassignable",
                        name
                    ));
                }
                var.value = value;
                return Ok(());
            }
        }
        Err(format!("'{}' is not defined", name))
    }
}
