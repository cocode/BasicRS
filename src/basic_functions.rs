use crate::basic_types::{BasicError, IdentifierType, Token};
use crate::basic_function_registry::FUNCTION_REGISTRY;
use lazy_static::lazy_static;
use rand::prelude::*;
use rand::Rng;
use std::collections::HashMap;

/// Argument types for BASIC functions
#[derive(Clone, Debug, PartialEq)]
pub enum ArgType {
    Number,
    String,
}

impl ArgType {
    pub fn name(&self) -> &str {
        match self {
            ArgType::Number => "number",
            ArgType::String => "string",
        }
    }
}

/// Helper function to validate argument count for BASIC functions
fn validate_arg_count(args: &[String], expected_count: usize, function_name: &str) -> Result<(), BasicError> {
    if args.len() != expected_count {
        return Err(BasicError::Syntax {
            message: format!("{}() takes exactly {} argument{}", 
                function_name, 
                expected_count, 
                if expected_count == 1 { "" } else { "s" }
            ),
            basic_line_number: None,
            file_line_number: None,
        });
    }
    Ok(())
}

/// Helper function to validate and convert arguments based on their expected types
fn validate_and_convert_args(args: &[String], arg_types: &[ArgType], function_name: &str) -> Result<Vec<String>, BasicError> {
    validate_arg_count(args, arg_types.len(), function_name)?;
    
    let mut converted_args = Vec::new();
    
    for (i, (arg, expected_type)) in args.iter().zip(arg_types.iter()).enumerate() {
        match expected_type {
            ArgType::Number => {
                // Try to parse as number to validate
                arg.parse::<f64>().map_err(|_| BasicError::Syntax {
                    message: format!("Invalid {} argument for {}(): expected number, got '{}'", 
                        match i {
                            0 => "first".to_string(),
                            1 => "second".to_string(), 
                            2 => "third".to_string(),
                            n => format!("{}th", n + 1),
                        },
                        function_name, 
                        arg
                    ),
                    basic_line_number: None,
                    file_line_number: None,
                })?;
                converted_args.push(arg.clone());
            }
            ArgType::String => {
                // For strings, we expect them to be quoted or we accept them as-is
                converted_args.push(arg.clone());
            }
        }
    }
    
    Ok(converted_args)
}

#[derive(Clone)]
pub enum BasicFunction {
    Number {
        name: String,
        lambda: fn(&[String]) -> Result<String, BasicError>,
        arg_types: Vec<ArgType>,
    },
    String {
        name: String,
        lambda: fn(&[String]) -> Result<String, BasicError>,
        arg_types: Vec<ArgType>,
    },
}

impl BasicFunction {
    pub fn call(&self, args: Vec<Token>) -> Result<Token, BasicError> {
        match self {
            BasicFunction::Number {
                lambda,
                arg_types,
                name,
            } => {
                let arg_strings: Vec<String> = args
                    .into_iter()
                    .map(|t| match t {
                        Token::Number(n) => Ok(n.clone()),
                        Token::String(s) => Ok(s.clone()),
                        Token::Identifier(name, IdentifierType::Variable) => Ok(name.clone()),
                        _ => Err(BasicError::Runtime {
                            message: format!("Invalid token: {:?}", t),
                            basic_line_number: None,
                            file_line_number: None,
                        }),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                
                let validated_args = validate_and_convert_args(&arg_strings, arg_types, name)?;
                let result = lambda(&validated_args)?;
                Ok(Token::new_number(&result))
            }

            BasicFunction::String {
                lambda,
                arg_types,
                name,
            } => {
                let arg_strings: Vec<String> = args
                    .into_iter()
                    .map(|t| t.token().unwrap_or("").to_string())
                    .collect();
                
                let validated_args = validate_and_convert_args(&arg_strings, arg_types, name)?;
                let result = lambda(&validated_args)?;
                Ok(Token::new_string(&result))
            }
        }
    }
    
    pub fn arg_count(&self) -> usize {
        match self {
            BasicFunction::Number { arg_types, .. } => arg_types.len(),
            BasicFunction::String { arg_types, .. } => arg_types.len(),
        }
    }
    
    pub fn arg_types(&self) -> &[ArgType] {
        match self {
            BasicFunction::Number { arg_types, .. } => arg_types,
            BasicFunction::String { arg_types, .. } => arg_types,
        }
    }
}

// Helper function to create BasicFunction from registry
fn create_basic_function_from_registry(name: &str) -> Option<BasicFunction> {
    if let Some(func_def) = FUNCTION_REGISTRY.get_function(name) {
        let arg_types = func_def.arg_types.clone();
        let implementation = func_def.implementation;
        
        match func_def.function_type {
            crate::basic_function_registry::FunctionType::Number => {
                Some(BasicFunction::Number {
                    name: name.to_string(),
                    lambda: implementation,
                    arg_types,
                })
            }
            crate::basic_function_registry::FunctionType::String => {
                Some(BasicFunction::String {
                    name: name.to_string(),
                    lambda: implementation,
                    arg_types,
                })
            }
        }
    } else {
        None
    }
}

lazy_static! {
    pub static ref FUNCTIONS: HashMap<String, BasicFunction> = {
        let mut m = HashMap::new();
        
        // Populate from registry
        for func_name in FUNCTION_REGISTRY.get_function_names() {
            if let Some(basic_func) = create_basic_function_from_registry(func_name) {
                m.insert(func_name.to_string(), basic_func);
            }
        }



        m
    };
}

pub fn get_function(name: &str) -> Option<BasicFunction> {
    create_basic_function_from_registry(name)
}

pub struct PredefinedFunctions;

impl PredefinedFunctions {
    pub fn new() -> Self {
        PredefinedFunctions
    }

    pub fn functions(&self) -> Vec<String> {
        FUNCTION_REGISTRY.get_numeric_function_names()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn call(&self, name: &str, args: &[f64]) -> Option<f64> {
        if FUNCTION_REGISTRY.is_numeric_function(name) {
            // Convert f64 args to strings for the registry
            let string_args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
            
            // Call the registry function
            if let Ok(result) = FUNCTION_REGISTRY.call_function(name, &string_args) {
                // Parse the result back to f64
                result.parse::<f64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        let abs_fn = get_function("ABS").unwrap();
        match abs_fn {
            BasicFunction::Number { lambda, .. } => {
                assert_eq!(lambda(&vec!["-1".to_string()]).unwrap(), "1");
                assert_eq!(lambda(&vec!["1".to_string()]).unwrap(), "1");
            }
            _ => panic!("Expected number function"),
        }
    }

    #[test]
    fn test_chr() {
        let chr_fn = get_function("CHR$").unwrap();
        match chr_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(lambda(&vec!["65".to_string()]).unwrap(), "A");
                assert_eq!(lambda(&vec!["97".to_string()]).unwrap(), "a");
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_left() {
        let left_fn = get_function("LEFT$").unwrap();
        match left_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(
                    lambda(&vec!["\"Hello\"".to_string(), "2".to_string()]).unwrap(),
                    "He"
                );
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_len() {
        let len_fn = get_function("LEN").unwrap();
        match len_fn {
            BasicFunction::Number { lambda, .. } => {
                assert_eq!(lambda(&vec!["\"Hello\"".to_string()]).unwrap(), "5");
            }
            _ => panic!("Expected number function"),
        }
    }

    #[test]
    fn test_mid() {
        let mid_fn = get_function("MID$").unwrap();
        match mid_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(
                    lambda(&vec![
                        "\"Hello\"".to_string(),
                        "2".to_string(),
                        "2".to_string()
                    ]).unwrap(),
                    "el"
                );
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_right() {
        let right_fn = get_function("RIGHT$").unwrap();
        match right_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(
                    lambda(&vec!["\"Hello\"".to_string(), "2".to_string()]).unwrap(),
                    "lo"
                );
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_sgn() {
        let sgn_fn = get_function("SGN").unwrap();
        match sgn_fn {
            BasicFunction::Number { lambda, .. } => {
                assert_eq!(lambda(&vec!["-1".to_string()]).unwrap(), "-1");
                assert_eq!(lambda(&vec!["0".to_string()]).unwrap(), "0");
                assert_eq!(lambda(&vec!["1".to_string()]).unwrap(), "1");
            }
            _ => panic!("Expected number function"),
        }
    }

    #[test]
    fn test_rnd() {
        let rnd_fn = get_function("RND").unwrap();
        match rnd_fn {
            BasicFunction::Number { lambda, .. } => {
                // Test RND(1) - returns random number between 0 and 1
                let result = lambda(&vec!["1".to_string()]).unwrap();
                let value = result.parse::<f64>().unwrap();
                assert!(value >= 0.0 && value < 1.0);

                // Test RND(-1) - seeds and returns random number between 0 and 1
                let result = lambda(&vec!["-1".to_string()]).unwrap();
                let value = result.parse::<f64>().unwrap();
                assert!(value >= 0.0 && value < 1.0);

                // Test RND(0) - returns random number between 0 and 1
                let result = lambda(&vec!["0".to_string()]).unwrap();
                let value = result.parse::<f64>().unwrap();
                assert!(value >= 0.0 && value < 1.0);
            }
            _ => panic!("Expected number function"),
        }
    }
}
