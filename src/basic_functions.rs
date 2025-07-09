use crate::basic_types::{BasicError, Token};
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
                    .map(|t| t.token().unwrap_or("").to_string())
                    .collect();

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

lazy_static! {
    pub static ref FUNCTIONS: HashMap<String, BasicFunction> = {
        let mut m = HashMap::new();

        // Math functions
        m.insert("ABS".to_string(), BasicFunction::Number {
            name: "ABS".to_string(),
            lambda: |args| {
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(value.abs().to_string())
            },
            arg_types: vec![ArgType::Number],
        });

        m.insert("SQR".to_string(), BasicFunction::Number {
            name: "SQR".to_string(),
            lambda: |args| {
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok((value * value).to_string())
            },
            arg_types: vec![ArgType::Number],
        });

        m.insert("RND".to_string(), BasicFunction::Number {
            name: "RND".to_string(),
            lambda: |args| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                if value > 0.0 {
                    Ok(rng.gen_range(0.0..1.0).to_string())
                } else if value < 0.0 {
                    Ok(value.to_string())
                } else {
                    Ok(rng.gen_range(0.0..1.0).to_string())
                }
            },
            arg_types: vec![ArgType::Number],
        });

        // String functions
        m.insert("LEN".to_string(), BasicFunction::Number {
            name: "LEN".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                Ok(s.chars().count().to_string())
            },
            arg_types: vec![ArgType::String],
        });

        m.insert("LEFT$".to_string(), BasicFunction::String {
            name: "LEFT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n: usize = args[1].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!("\"{}\"", s.chars().take(n).collect::<String>()))
            },
            arg_types: vec![ArgType::String, ArgType::Number],
        });

        m.insert("RIGHT$".to_string(), BasicFunction::String {
            name: "RIGHT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n: usize = args[1].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!("\"{}\"", s.chars().rev().take(n).collect::<String>().chars().rev().collect::<String>()))
            },
            arg_types: vec![ArgType::String, ArgType::Number],
        });

        m.insert("MID$".to_string(), BasicFunction::String {
            name: "MID$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let start: usize = args[1].parse::<usize>().unwrap().saturating_sub(1); // Already validated by validate_and_convert_args
                let len: usize = args[2].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!("\"{}\"", s.chars().skip(start).take(len).collect::<String>()))
            },
            arg_types: vec![ArgType::String, ArgType::Number, ArgType::Number],
        });

        m.insert("CHR$".to_string(), BasicFunction::String {
            name: "CHR$".to_string(),
            lambda: |args| {
                let value: u8 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!("\"{}\"", char::from(value)))
            },
            arg_types: vec![ArgType::Number],
        });

        m.insert("SGN".to_string(), BasicFunction::Number {
            name: "SGN".to_string(),
            lambda: |args| {
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(if value > 0.0 { "1" }
                else if value < 0.0 { "-1" }
                else { "0" }.to_string())
            },
            arg_types: vec![ArgType::Number],
        });

        m
    };
}

pub fn get_function(name: &str) -> Option<BasicFunction> {
    match name {
        "ABS" => Some(BasicFunction::Number {
            name: "ABS".to_string(),
            lambda: |args| {
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(value.abs().to_string())
            },
            arg_types: vec![ArgType::Number],
        }),
        "CHR$" => Some(BasicFunction::String {
            name: "CHR$".to_string(),
            lambda: |args| {
                let value: u8 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!("\"{}\"", char::from(value)))
            },
            arg_types: vec![ArgType::Number],
        }),
        "LEFT$" => Some(BasicFunction::String {
            name: "LEFT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n: usize = args[1].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!("\"{}\"", s.chars().take(n).collect::<String>()))
            },
            arg_types: vec![ArgType::String, ArgType::Number],
        }),
        "LEN" => Some(BasicFunction::Number {
            name: "LEN".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                Ok(s.chars().count().to_string())
            },
            arg_types: vec![ArgType::String],
        }),
        "MID$" => Some(BasicFunction::String {
            name: "MID$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let start: usize = args[1].parse::<usize>().unwrap().saturating_sub(1); // Already validated by validate_and_convert_args
                let len: usize = args[2].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!(
                    "\"{}\"",
                    s.chars().skip(start).take(len).collect::<String>()
                ))
            },
            arg_types: vec![ArgType::String, ArgType::Number, ArgType::Number],
        }),
        "RIGHT$" => Some(BasicFunction::String {
            name: "RIGHT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n: usize = args[1].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(format!(
                    "\"{}\"",
                    s.chars()
                        .rev()
                        .take(n)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>()
                ))
            },
            arg_types: vec![ArgType::String, ArgType::Number],
        }),
        "RND" => Some(BasicFunction::Number {
            name: "RND".to_string(),
            lambda: |args| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                if value > 0.0 {
                    Ok(rng.gen_range(0.0..1.0).to_string())
                } else if value < 0.0 {
                    Ok(value.to_string())
                } else {
                    Ok(rng.gen_range(0.0..1.0).to_string())
                }
            },
            arg_types: vec![ArgType::Number],
        }),
        "SGN" => Some(BasicFunction::Number {
            name: "SGN".to_string(),
            lambda: |args| {
                let value: f64 = args[0].parse().unwrap(); // Already validated by validate_and_convert_args
                Ok(if value > 0.0 {
                    "1"
                } else if value < 0.0 {
                    "-1"
                } else {
                    "0"
                }
                .to_string())
            },
            arg_types: vec![ArgType::Number],
        }),
        _ => None,
    }
}

pub struct PredefinedFunctions;

impl PredefinedFunctions {
    pub fn new() -> Self {
        PredefinedFunctions
    }

    pub fn functions(&self) -> Vec<String> {
        vec![
            "ABS".to_string(),
            "ATN".to_string(),
            "COS".to_string(),
            "EXP".to_string(),
            "INT".to_string(),
            "LOG".to_string(),
            "RND".to_string(),
            "SIN".to_string(),
            "SQR".to_string(),
            "TAN".to_string(),
        ]
    }

    pub fn call(&self, name: &str, args: &[f64]) -> Option<f64> {
        match name.to_uppercase().as_str() {
            "ABS" => args.get(0).map(|x| x.abs()),
            "ATN" => args.get(0).map(|x| x.atan()),
            "COS" => args.get(0).map(|x| x.cos()),
            "EXP" => args.get(0).map(|x| x.exp()),
            "INT" => args.get(0).map(|x| x.floor()),
            "LOG" => args.get(0).map(|x| x.ln()),
            "RND" => {
                if args.is_empty() {
                    Some(rand::thread_rng().gen())
                } else {
                    args.get(0).map(|&x| {
                        if x < 0.0 {
                            // Seed the RNG
                            let mut rng = rand::rngs::StdRng::seed_from_u64(x.abs() as u64);
                            rng.gen()
                        } else if x == 0.0 {
                            // Return last number
                            0.0 // TODO: Store last number
                        } else {
                            // Return random number between 0 and x
                            rand::thread_rng().gen_range(0.0..x)
                        }
                    })
                }
            }
            "SIN" => args.get(0).map(|x| x.sin()),
            "SQR" => args.get(0).map(|x| x.sqrt()),
            "TAN" => args.get(0).map(|x| x.tan()),
            _ => None,
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
                assert_eq!(lambda(&vec!["65".to_string()]).unwrap(), "\"A\"");
                assert_eq!(lambda(&vec!["97".to_string()]).unwrap(), "\"a\"");
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
                    "\"He\""
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
                    "\"el\""
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
                    "\"lo\""
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

                // Test RND(-1) - returns -1
                assert_eq!(lambda(&vec!["-1".to_string()]).unwrap(), "-1");

                // Test RND(0) - returns random number between 0 and 1
                let result = lambda(&vec!["0".to_string()]).unwrap();
                let value = result.parse::<f64>().unwrap();
                assert!(value >= 0.0 && value < 1.0);
            }
            _ => panic!("Expected number function"),
        }
    }
}
