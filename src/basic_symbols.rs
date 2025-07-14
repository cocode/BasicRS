use std::collections::HashMap;
use crate::basic_dialect::ARRAY_OFFSET;
use crate::basic_types::{BasicError, Expression, SymbolValue, ArrayElementType, ArrayData};

#[derive(Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, SymbolValue>,
    parent: Option<Box<SymbolTable>>,
}

pub fn adjust(coord: usize) -> usize {
    return coord - ARRAY_OFFSET;
}

impl SymbolTable {
    /// Validates array indices against ARRAY_OFFSET and dimension bounds, returning adjusted indices
    fn validate_and_adjust_indices(&self, name: &str, indices: &[usize], dimensions: &[usize]) -> Result<Vec<usize>, BasicError> {
        // Check dimension count
        if indices.len() != dimensions.len() {
            return Err(BasicError::Runtime {
                message: format!("Array '{}' expects {} indices, got {}", name, dimensions.len(), indices.len()),
                basic_line_number: None,
                file_line_number: None,
            });
        }
        
        // Check ARRAY_OFFSET bounds and adjust
        let mut adjusted = Vec::new();
        for (i, (&index, &dim_size)) in indices.iter().zip(dimensions.iter()).enumerate() {
            if index < ARRAY_OFFSET {
                return Err(BasicError::Runtime {
                    message: format!("Array index {} out of bounds for '{}' dimension {}. Valid range: {} to {}", 
                        index, name, i, ARRAY_OFFSET, dim_size - 1 + ARRAY_OFFSET),
                    basic_line_number: None,
                    file_line_number: None,
                });
            }
            let adjusted_index = index - ARRAY_OFFSET;
            if adjusted_index >= dim_size {
                return Err(BasicError::Runtime {
                    message: format!("Array index {} out of bounds for '{}' dimension {}. Valid range: {} to {}", 
                        index, name, i, ARRAY_OFFSET, dim_size - 1 + ARRAY_OFFSET),
                    basic_line_number: None,
                    file_line_number: None,
                });
            }
            adjusted.push(adjusted_index);
        }
        Ok(adjusted)
    }
    
    /// Converts multi-dimensional indices to flat index using row-major order
    fn calculate_flat_index(indices: &[usize], dimensions: &[usize]) -> usize {
        let mut flat_index = 0;
        let mut stride = 1;
        
        // Calculate flat index in row-major order
        for i in (0..indices.len()).rev() {
            flat_index += indices[i] * stride;
            if i > 0 {
                stride *= dimensions[i];
            }
        }
        flat_index
    }

    pub fn get_array_element(&self, name: &str, indices: &[usize]) -> Result<SymbolValue, BasicError> {
        // Arrays are stored with [] suffix to separate from scalar variables
        let array_key = format!("{}[]", name);
        let symbol = self.get_symbol(&array_key).ok_or(BasicError::Runtime {
            message: format!("Array '{}' not found", name),
            basic_line_number: None,
            file_line_number: None,
        })?;

        match symbol {
            // New unified array type
            SymbolValue::Array { element_type, dimensions, data } => {
                let adjusted_indices = self.validate_and_adjust_indices(name, indices, dimensions)?;
                let flat_index = Self::calculate_flat_index(&adjusted_indices, dimensions);
                
                match (element_type, data) {
                    (ArrayElementType::Number, ArrayData::Numbers(vec)) => {
                        Ok(SymbolValue::Number(vec[flat_index]))
                    }
                    (ArrayElementType::String, ArrayData::Strings(vec)) => {
                        Ok(SymbolValue::String(vec[flat_index].clone()))
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("Array '{}' has mismatched element type and data", name),
                        basic_line_number: None,
                        file_line_number: None,
                    }),
                }
            }

            // Legacy array types - maintain backwards compatibility during transition
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
        
        // First, validate indices without borrowing symbols mutably
        let (adjusted_indices, flat_index) = {
            let symbol = self.symbols.get(&array_key).ok_or(BasicError::Runtime {
                message: format!("Array '{}' not found", name),
                basic_line_number: None,
                file_line_number: None,
            })?;
            
            match symbol {
                SymbolValue::Array { dimensions, .. } => {
                    let adjusted_indices = self.validate_and_adjust_indices(name, indices, dimensions)?;
                    let flat_index = Self::calculate_flat_index(&adjusted_indices, dimensions);
                    (adjusted_indices, flat_index)
                }
                _ => {
                    // For legacy arrays, we'll handle validation below
                    (Vec::new(), 0)
                }
            }
        };
        
        // Now get mutable access to update the array
        let symbol = self.symbols.get_mut(&array_key).ok_or(BasicError::Runtime {
            message: format!("Array '{}' not found", name),
            basic_line_number: None,
            file_line_number: None,
        })?;

        match symbol {
            // New unified array type
            SymbolValue::Array { element_type, data, .. } => {
                match (element_type, data, value) {
                    (ArrayElementType::Number, ArrayData::Numbers(vec), SymbolValue::Number(n)) => {
                        vec[flat_index] = n;
                        Ok(())
                    }
                    (ArrayElementType::String, ArrayData::Strings(vec), SymbolValue::String(s)) => {
                        vec[flat_index] = s;
                        Ok(())
                    }
                    (ArrayElementType::Number, _, _) => {
                        Err(BasicError::Runtime {
                            message: "Type mismatch: expected number for numeric array".to_string(),
                            basic_line_number: None,
                            file_line_number: None,
                        })
                    }
                    (ArrayElementType::String, _, _) => {
                        Err(BasicError::Runtime {
                            message: "Type mismatch: expected string for string array".to_string(),
                            basic_line_number: None,
                            file_line_number: None,
                        })
                    }
                }
            }

            // Legacy array types - maintain backwards compatibility during transition
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
        let total_elements: usize = dimensions.iter().product();

        // Create new unified array type
        let array = if is_string {
            SymbolValue::Array {
                element_type: ArrayElementType::String,
                dimensions: dimensions.clone(),
                data: ArrayData::Strings(vec!["".to_string(); total_elements]),
            }
        } else {
            SymbolValue::Array {
                element_type: ArrayElementType::Number,
                dimensions: dimensions.clone(),
                data: ArrayData::Numbers(vec![0.0; total_elements]),
            }
        };

        self.symbols.insert(array_key, array);
        Ok(())
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
            SymbolValue::Array { element_type: ArrayElementType::Number, dimensions, data: ArrayData::Numbers(v) } => {
                assert_eq!(*dimensions, vec![5]);
                assert_eq!(v.len(), 5);
            },
            _ => panic!("Expected 1D number array"),
        }
    }

    #[test]
    fn test_create_array_2d_string() {
        let mut table = SymbolTable::new();
        table.create_array("S$".to_string(), vec![2, 3]).unwrap();
        let val = table.get_symbol("S$[]").unwrap(); // Arrays stored with [] suffix
        match val {
            SymbolValue::Array { element_type: ArrayElementType::String, dimensions, data: ArrayData::Strings(v) } => {
                assert_eq!(*dimensions, vec![2, 3]);
                assert_eq!(v.len(), 6); // 2 * 3 = 6 total elements in flattened array
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
