use std::collections::HashMap;
use crate::basic_types::{BasicError, Expression, SymbolValue};

#[derive(Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolValue>,
    parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    pub fn get_array_element(&self, name: &str, indices: &[usize]) -> Result<SymbolValue, BasicError> {
        let symbol = self.get_symbol(name).ok_or(BasicError::Runtime {
            message: format!("Array '{}' not found", name),
            line_number: None,
        })?;

        match symbol {
            SymbolValue::Array1DNumber(vec) => {
                if indices.len() != 1 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 1 index", name),
                        line_number: None,
                    });
                }
                let index = indices[0];
                if index >= vec.len() {
                    return Err(BasicError::Runtime {
                        message: "Array index out of bounds".to_string(),
                        line_number: None,
                    });
                }
                Ok(SymbolValue::Number(vec[index]))
            }

            SymbolValue::Array2DNumber(vec) => {
                if indices.len() != 2 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 2 indices", name),
                        line_number: None,
                    });
                }
                let row = indices[0];
                let col = indices[1];
                if row >= vec.len() || col >= vec[row].len() {
                    return Err(BasicError::Runtime {
                        message: "Array index out of bounds".to_string(),
                        line_number: None,
                    });
                }
                Ok(SymbolValue::Number(vec[row][col]))
            }

            SymbolValue::Array1DString(vec) => {
                if indices.len() != 1 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 1 index", name),
                        line_number: None,
                    });
                }
                let index = indices[0];
                if index >= vec.len() {
                    return Err(BasicError::Runtime {
                        message: "Array index out of bounds".to_string(),
                        line_number: None,
                    });
                }
                Ok(SymbolValue::String(vec[index].clone()))
            }

            SymbolValue::Array2DString(vec) => {
                if indices.len() != 2 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 2 indices", name),
                        line_number: None,
                    });
                }
                let row = indices[0];
                let col = indices[1];
                if row >= vec.len() || col >= vec[row].len() {
                    return Err(BasicError::Runtime {
                        message: "Array index out of bounds".to_string(),
                        line_number: None,
                    });
                }
                Ok(SymbolValue::String(vec[row][col].clone()))
            }

            _ => Err(BasicError::Runtime {
                message: format!("'{}' is not an array", name),
                line_number: None,
            }),
        }
    }
    pub fn create_array(&mut self, name: String, dimensions: Vec<usize>) -> Result<(), BasicError> {
        if self.symbols.contains_key(&name) {
            return Err(BasicError::Runtime {
                message: format!("Array '{}' already declared", name),
                line_number: None,
            });
        }

        let is_string = name.ends_with('$');

        match dimensions.len() {
            1 => {
                let size = dimensions[0];
                let array = if is_string {
                    SymbolValue::Array1DString(vec!["".to_string(); size])
                } else {
                    SymbolValue::Array1DNumber(vec![0.0; size])
                };
                self.symbols.insert(name, array);
                Ok(())
            }

            2 => {
                let rows = dimensions[0];
                let cols = dimensions[1];
                let array = if is_string {
                    SymbolValue::Array2DString(vec![vec!["".to_string(); cols]; rows])
                } else {
                    SymbolValue::Array2DNumber(vec![vec![0.0; cols]; rows])
                };
                self.symbols.insert(name, array);
                Ok(())
            }

            _ => Err(BasicError::Runtime {
                message: "Only 1D and 2D arrays are supported".to_string(),
                line_number: None,
            }),
        }
    }
    pub fn define_function(&mut self, name: String, param: Vec<String>, expr: Expression) -> Result<(), BasicError> {
        if self.symbols.contains_key(&name) {
            return Err(BasicError::Runtime {
                message: format!("Function '{}' already defined", name),
                line_number: None,
            });
        }

        self.symbols.insert(name, SymbolValue::FunctionDef { param, expr });
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
