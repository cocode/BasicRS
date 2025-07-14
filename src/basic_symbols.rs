use std::collections::HashMap;
use crate::basic_dialect::ARRAY_OFFSET;
use crate::basic_types::{BasicError, Expression, SymbolValue};

#[derive(Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolValue>,
    parent: Option<Box<SymbolTable>>,
}

pub fn adjust(coord: usize) -> usize {
    return coord - ARRAY_OFFSET;
}

impl SymbolTable {

    pub fn get_array_element(&self, name: &str, indices: &[usize]) -> Result<SymbolValue, BasicError> {
        // Arrays are stored with [] suffix to separate from scalar variables
        let array_key = format!("{}[]", name);
        let symbol = self.get_symbol(&array_key).ok_or(BasicError::Runtime {
            message: format!("Array '{}' not found", name),
            basic_line_number: None,
            file_line_number: None,
        })?;

        match symbol {
            SymbolValue::Array1DNumber(vec) => {
                if indices.len() != 1 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 1 index", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let index = adjust(indices[0]);
                if index >= vec.len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                Ok(SymbolValue::Number(vec[index]))
            }

            SymbolValue::Array2DNumber(vec) => {
                if indices.len() != 2 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 2 indices", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET || indices[1] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let row = adjust(indices[0]);
                let col = adjust(indices[1]);

                if row >= vec.len() || col >= vec[row].len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                Ok(SymbolValue::Number(vec[row][col]))
            }

            SymbolValue::Array1DString(vec) => {
                if indices.len() != 1 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 1 index", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let index = adjust(indices[0]);
                if index >= vec.len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                Ok(SymbolValue::String(vec[index].clone()))
            }

            SymbolValue::Array2DString(vec) => {
                if indices.len() != 2 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 2 indices", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET || indices[1] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let row = adjust(indices[0]);
                let col = adjust(indices[1]);
                if row >= vec.len() || col >= vec[row].len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                Ok(SymbolValue::String(vec[row][col].clone()))
            }

            _ => Err(BasicError::Runtime {
                message: format!("'{}' is not an array", name),
                basic_line_number: None,
                file_line_number: None,
            }),
        }
    }

    pub fn set_array_element(&mut self, name: &str, indices: &[usize], value: SymbolValue) -> Result<(), BasicError> {
        // Arrays are stored with [] suffix to separate from scalar variables
        let array_key = format!("{}[]", name);
        let symbol = self.symbols.get_mut(&array_key).ok_or(BasicError::Runtime {
            message: format!("Array '{}' not found", name),
            basic_line_number: None,
            file_line_number: None,
        })?;

        match symbol {
            SymbolValue::Array1DNumber(vec) => {
                if indices.len() != 1 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 1 index", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let index = adjust(indices[0]);
                if index  >= vec.len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if let SymbolValue::Number(n) = value {
                    vec[index] = n;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "Type mismatch: expected number".to_string(),
                        basic_line_number: None,
                        file_line_number: None,
                    })
                }
            }

            SymbolValue::Array2DNumber(vec) => {
                if indices.len() != 2 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 2 indices", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET || indices[1] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let row = adjust(indices[0]);
                let col = adjust(indices[1]);
                if row >= vec.len() || col >= vec[row].len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if let SymbolValue::Number(n) = value {
                    vec[row][col] = n;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "Type mismatch: expected number".to_string(),
                        basic_line_number: None,
                        file_line_number: None,
                    })
                }
            }

            SymbolValue::Array1DString(vec) => {
                if indices.len() != 1 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 1 index", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let index = adjust(indices[0]);
                if index >= vec.len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index {} out of bounds for '{}'. Valid range: {} to {}", indices[0], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if let SymbolValue::String(s) = value {
                    vec[index] = s;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "Type mismatch: expected string".to_string(),
                        basic_line_number: None,
                        file_line_number: None,
                    })
                }
            }

            SymbolValue::Array2DString(vec) => {
                if indices.len() != 2 {
                    return Err(BasicError::Runtime {
                        message: format!("Array '{}' expects 2 indices", name),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if indices[0] < ARRAY_OFFSET || indices[1] < ARRAY_OFFSET {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let row = adjust(indices[0]);
                let col = adjust(indices[1]);
                if row >= vec.len() || col >= vec[row].len() {
                    return Err(BasicError::Runtime {
                        message: format!("Array index ({}, {}) out of bounds for '{}'. Valid row range: {}-{}, col range: {}-{}", indices[0], indices[1], name, ARRAY_OFFSET, vec.len() - 1 + ARRAY_OFFSET, ARRAY_OFFSET, vec[0].len() - 1 + ARRAY_OFFSET),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                if let SymbolValue::String(s) = value {
                    vec[row][col] = s;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "Type mismatch: expected string".to_string(),
                        basic_line_number: None,
                        file_line_number: None,
                    })
                }
            }

            _ => Err(BasicError::Runtime {
                message: format!("'{}' is not an array", name),
                basic_line_number: None,
                file_line_number: None,
            }),
        }
    }

    pub fn create_array(&mut self, name: String, dimensions: Vec<usize>) -> Result<(), BasicError> {
        // Arrays are stored with [] suffix to separate from scalar variables
        let array_key = format!("{}[]", name);
        if self.symbols.contains_key(&array_key) {
            return Err(BasicError::Runtime {
                message: format!("Array '{}' already declared", name),
                basic_line_number: None,
                file_line_number: None,
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
                self.symbols.insert(array_key, array);
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
                self.symbols.insert(array_key, array);
                Ok(())
            }

            _ => Err(BasicError::Runtime {
                message: "Only 1D and 2D arrays are supported".to_string(),
                basic_line_number: None,
                file_line_number: None,
            }),
        }
    }
    pub fn define_function(&mut self, name: String, param: Vec<String>, expr: Expression) -> Result<(), BasicError> {
        if self.symbols.contains_key(&name) {
            return Err(BasicError::Runtime {
                message: format!("Function '{}' already defined", name),
                basic_line_number: None,
                file_line_number: None,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{Expression, SymbolValue};

    #[test]
    fn test_basic_symbols() {
        let mut table = SymbolTable::new();
        table.put_symbol("A".to_string(), SymbolValue::Number(1.0));
        let val = table.get_symbol("A").unwrap();
        assert_eq!(SymbolValue::Number(1.0), *val);
        table.put_symbol("B".to_string(), SymbolValue::Number(99.0));

        let dump = table.dump();
        assert_eq!(dump.len(), 2);
        assert_eq!(dump["A"], SymbolValue::Number(1.0));
        assert_eq!(dump["B"], SymbolValue::Number(99.0));
    }

    #[test]
    fn test_create_array_1d_number() {
        let mut table = SymbolTable::new();
        table.create_array("A".to_string(), vec![5]).unwrap();
        let val = table.get_symbol("A[]").unwrap(); // Arrays stored with [] suffix
        match val {
            SymbolValue::Array1DNumber(v) => assert_eq!(v.len(), 5),
            _ => panic!("Expected 1D number array"),
        }
    }

    #[test]
    fn test_create_array_2d_string() {
        let mut table = SymbolTable::new();
        table.create_array("S$".to_string(), vec![2, 3]).unwrap();
        let val = table.get_symbol("S$[]").unwrap(); // Arrays stored with [] suffix
        match val {
            SymbolValue::Array2DString(v) => {
                assert_eq!(v.len(), 2);
                assert_eq!(v[0].len(), 3);
            }
            _ => panic!("Expected 2D string array"),
        }
    }

    #[test]
    fn test_get_array_element_valid() {
        let mut table = SymbolTable::new();
        table.create_array("A".to_string(), vec![3]).unwrap();
        let val = table.get_array_element("A", &[1]).unwrap();
        assert_eq!(val, SymbolValue::Number(0.0));
    }

    #[test]
    fn test_get_array_element_invalid_index() {
        let mut table = SymbolTable::new();
        table.create_array("A".to_string(), vec![2]).unwrap();
        let result = table.get_array_element("A", &[5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_array_element_invalid_low_index() {
        let mut table = SymbolTable::new();
        table.create_array("A".to_string(), vec![2]).unwrap();
        // Assumes ARRAY_OFFSET is 1, index 0 is invalid
        let result = table.get_array_element("A", &[0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_define_function() {
        let mut table = SymbolTable::new();
        let expr = Expression::new_number(42.0);
        table.define_function("F".to_string(), vec!["X".to_string()], expr.clone()).unwrap();

        let val = table.get_symbol("F").unwrap();
        match val {
            SymbolValue::FunctionDef { param, expr: e } => {
                assert_eq!(param, &vec!["X".to_string()]);
                assert_eq!(e, &expr);
            }
            _ => panic!("Expected FunctionDef"),
        }
    }

    #[test]
    fn test_nested_scope_lookup() {
        let mut root = SymbolTable::new();
        root.put_symbol("X".to_string(), SymbolValue::Number(5.0));
        let nested = root.get_nested_scope();
        assert_eq!(nested.get_symbol("X"), Some(&SymbolValue::Number(5.0)));
    }

    #[test]
    fn test_dump_merges_with_parent() {
        let mut parent = SymbolTable::new();
        parent.put_symbol("A".to_string(), SymbolValue::Number(1.0));

        let mut child = parent.get_nested_scope();
        child.put_symbol("B".to_string(), SymbolValue::Number(2.0));

        let dump = child.dump();
        assert_eq!(dump.len(), 2);
        assert_eq!(dump["A"], SymbolValue::Number(1.0));
        assert_eq!(dump["B"], SymbolValue::Number(2.0));
    }
}
