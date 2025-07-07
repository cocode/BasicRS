use std::collections::HashMap;
use crate::basic_types::{BasicError, SymbolValue};

#[derive(Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolValue>,
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn create_array(&mut self, name: String, dimensions: Vec<usize>) -> Result<(), BasicError> {
        if self.symbols.contains_key(&name) {
            return Err(BasicError::Runtime {
                message: format!("Array '{}' already declared", name),
                line_number: None,
            });
        }

        let total_size: usize = dimensions.iter().product();

        let array = vec![SymbolValue::Number(0.0); total_size];

        self.symbols.insert(name, SymbolValue::Array(array));

        Ok(())
    }
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn get_nested_scope(&self) -> Self {
        SymbolTable {
            symbols: HashMap::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    pub fn get_symbol(&self, name: &str) -> Option<&SymbolValue> {
        if let Some(value) = self.symbols.get(name) {
            Some(value)
        } else if let Some(parent) = &self.parent {
            parent.get_symbol(name)
        } else {
            None
        }
    }

    pub fn put_symbol(&mut self, name: String, value: SymbolValue) {
        self.symbols.insert(name, value);
    }

    pub fn dump(&self) -> HashMap<String, SymbolValue> {
        let mut result = HashMap::new();
        for (name, value) in &self.symbols {
            result.insert(name.clone(), value.clone());
        }
        if let Some(parent) = &self.parent {
            for (name, value) in parent.dump() {
                if !result.contains_key(&name) {
                    result.insert(name, value);
                }
            }
        }
        result
    }
}
