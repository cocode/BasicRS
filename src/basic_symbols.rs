use std::collections::HashMap;
use crate::basic_types::{SymbolValue};

#[derive(Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolValue>,
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
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
