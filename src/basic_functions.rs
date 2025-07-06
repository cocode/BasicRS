use std::collections::HashMap;
use lazy_static::lazy_static;
use rand::Rng;
use crate::basic_types::{Token, BasicError};
use rand::prelude::*;

#[derive(Clone)]
pub enum BasicFunction {
    Number {
        name: String,
        lambda: fn(&[String]) -> Result<String, BasicError>,
        arg_count: usize,
    },
    String {
        name: String,
        lambda: fn(&[String]) -> Result<String, BasicError>,
        arg_count: usize,
    },
}

impl BasicFunction {
    pub fn call(&self, args: Vec<Token>) -> Result<Token, BasicError> {
        match self {
            BasicFunction::Number { lambda, arg_count, name } => {
                if args.len() != *arg_count {
                    return Err(BasicError::Syntax {
                        message: format!("Wrong number of arguments for {}", name),
                        line_number: None,
                    });
                }

                let arg_strings: Vec<String> = args
                    .into_iter()
                    .map(|t| t.token().unwrap_or("").to_string())
                    .collect();

                let result = lambda(&arg_strings)?;  // Propagate error from lambda
                Ok(Token::new_number(&result))
            }

            BasicFunction::String { lambda, arg_count, name } => {
                if args.len() != *arg_count {
                    return Err(BasicError::Syntax {
                        message: format!("Wrong number of arguments for {}", name),
                        line_number: None,
                    });
                }

                let arg_strings: Vec<String> = args
                    .into_iter()
                    .map(|t| t.token().unwrap_or("").to_string())
                    .collect();
                let result = lambda(&arg_strings)?;  // Propagate error from lambda
                Ok(Token::new_string(&result))
            }
        }
    }
    pub fn arg_count(&self) -> usize {
        match self {
            BasicFunction::Number { arg_count, .. } => *arg_count,
            BasicFunction::String { arg_count, .. } => *arg_count,
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
                if args.len() != 1 {
                    return Err(BasicError::Syntax {
                        message: "ABS() takes exactly 1 argument".to_string(),
                        line_number: None,
                    });
                }

                let value: f64 = args[0].parse().map_err(|_| BasicError::Syntax {
                    message: format!("Invalid numeric argument for ABS(): '{}'", args[0]),
                    line_number: None,
                })?;
                Ok(value.abs().to_string())
            },
            arg_count: 1,
        });

        m.insert("SQR".to_string(), BasicFunction::Number {
            name: "SQR".to_string(),
            lambda: |args| {
                let value = args[0].parse::<f64>().unwrap_or(0.0);
                Ok((value * value).to_string())
            },
            arg_count: 1,
        });

        m.insert("RND".to_string(), BasicFunction::Number {
            name: "RND".to_string(),
            lambda: |args| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let value = args[0].parse::<f64>().unwrap_or(0.0);
                if value > 0.0 {
                    rng.gen_range(0.0..1.0).to_string()
                } else if value < 0.0 {
                    value.to_string()
                } else {
                    rng.gen_range(0.0..1.0).to_string()
                }
            },
            arg_count: 1,
        });

        // String functions
        m.insert("LEN".to_string(), BasicFunction::Number {
            name: "LEN".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                s.chars().count().to_string()
            },
            arg_count: 1,
        });

        m.insert("LEFT$".to_string(), BasicFunction::String {
            name: "LEFT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n = args[1].parse::<usize>().unwrap_or(0);
                format!("\"{}\"", s.chars().take(n).collect::<String>())
            },
            arg_count: 2,
        });

        m.insert("RIGHT$".to_string(), BasicFunction::String {
            name: "RIGHT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n = args[1].parse::<usize>().unwrap_or(0);
                format!("\"{}\"", s.chars().rev().take(n).collect::<String>().chars().rev().collect::<String>())
            },
            arg_count: 2,
        });

        m.insert("MID$".to_string(), BasicFunction::String {
            name: "MID$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let start = args[1].parse::<usize>().unwrap_or(1).saturating_sub(1);
                let len = args[2].parse::<usize>().unwrap_or(s.len());
                format!("\"{}\"", s.chars().skip(start).take(len).collect::<String>())
            },
            arg_count: 3,
        });

        m.insert("CHR$".to_string(), BasicFunction::String {
            name: "CHR$".to_string(),
            lambda: |args| {
                let value = args[0].parse::<u8>().unwrap_or(0);
                format!("\"{}\"", char::from(value))
            },
            arg_count: 1,
        });

        m.insert("SGN".to_string(), BasicFunction::Number {
            name: "SGN".to_string(),
            lambda: |args| {
                let value = args[0].parse::<f64>().unwrap_or(0.0);
                if value > 0.0 { "1" }
                else if value < 0.0 { "-1" }
                else { "0" }.to_string()
            },
            arg_count: 1,
        });

        m
    };
}

pub fn get_function(name: &str) -> Option<BasicFunction> {
    match name {
        "ABS" => Some(BasicFunction::Number {
            name: "ABS".to_string(),
            lambda: |args| {
                let value = args[0].parse::<f64>().unwrap_or(0.0);
                Ok(value.abs().to_string())
            },
            arg_count: 1,
        }),
        "CHR$" => Some(BasicFunction::String {
            name: "CHR$".to_string(),
            lambda: |args| {
                let value = args[0].parse::<u8>().unwrap_or(0);
                format!("\"{}\"", char::from(value))
            },
            arg_count: 1,
        }),
        "LEFT$" => Some(BasicFunction::String {
            name: "LEFT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n = args[1].parse::<usize>().unwrap_or(0);
                format!("\"{}\"", s.chars().take(n).collect::<String>())
            },
            arg_count: 2,
        }),
        "LEN" => Some(BasicFunction::Number {
            name: "LEN".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                s.chars().count().to_string()
            },
            arg_count: 1,
        }),
        "MID$" => Some(BasicFunction::String {
            name: "MID$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let start = args[1].parse::<usize>().unwrap_or(1).saturating_sub(1);
                let len = args[2].parse::<usize>().unwrap_or(s.len());
                format!("\"{}\"", s.chars().skip(start).take(len).collect::<String>())
            },
            arg_count: 3,
        }),
        "RIGHT$" => Some(BasicFunction::String {
            name: "RIGHT$".to_string(),
            lambda: |args| {
                let s = args[0].trim_matches('"');
                let n = args[1].parse::<usize>().unwrap_or(0);
                format!("\"{}\"", s.chars().rev().take(n).collect::<String>().chars().rev().collect::<String>())
            },
            arg_count: 2,
        }),
        "RND" => Some(BasicFunction::Number {
            name: "RND".to_string(),
            lambda: |args| {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let value = args[0].parse::<f64>().unwrap_or(0.0);
                if value > 0.0 {
                    rng.gen_range(0.0..1.0).to_string()
                } else if value < 0.0 {
                    value.to_string()
                } else {
                    rng.gen_range(0.0..1.0).to_string()
                }
            },
            arg_count: 1,
        }),
        "SGN" => Some(BasicFunction::Number {
            name: "SGN".to_string(),
            lambda: |args| {
                let value = args[0].parse::<f64>().unwrap_or(0.0);
                if value > 0.0 { "1" }
                else if value < 0.0 { "-1" }
                else { "0" }.to_string()
            },
            arg_count: 1,
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
                assert_eq!(lambda(&vec!["-1".to_string()]), "1");
                assert_eq!(lambda(&vec!["1".to_string()]), "1");
            }
            _ => panic!("Expected number function"),
        }
    }

    #[test]
    fn test_chr() {
        let chr_fn = get_function("CHR$").unwrap();
        match chr_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(lambda(&vec!["65".to_string()]), "\"A\"");
                assert_eq!(lambda(&vec!["97".to_string()]), "\"a\"");
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_left() {
        let left_fn = get_function("LEFT$").unwrap();
        match left_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(lambda(&vec!["\"Hello\"".to_string(), "2".to_string()]), "\"He\"");
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_len() {
        let len_fn = get_function("LEN").unwrap();
        match len_fn {
            BasicFunction::Number { lambda, .. } => {
                assert_eq!(lambda(&vec!["\"Hello\"".to_string()]), "5");
            }
            _ => panic!("Expected number function"),
        }
    }

    #[test]
    fn test_mid() {
        let mid_fn = get_function("MID$").unwrap();
        match mid_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(lambda(&vec!["\"Hello\"".to_string(), "2".to_string(), "2".to_string()]), "\"el\"");
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_right() {
        let right_fn = get_function("RIGHT$").unwrap();
        match right_fn {
            BasicFunction::String { lambda, .. } => {
                assert_eq!(lambda(&vec!["\"Hello\"".to_string(), "2".to_string()]), "\"lo\"");
            }
            _ => panic!("Expected string function"),
        }
    }

    #[test]
    fn test_sgn() {
        let sgn_fn = get_function("SGN").unwrap();
        match sgn_fn {
            BasicFunction::Number { lambda, .. } => {
                assert_eq!(lambda(&vec!["-1".to_string()]), "-1");
                assert_eq!(lambda(&vec!["0".to_string()]), "0");
                assert_eq!(lambda(&vec!["1".to_string()]), "1");
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
                let result = lambda(&vec!["1".to_string()]);
                let value = result.parse::<f64>().unwrap();
                assert!(value >= 0.0 && value < 1.0);

                // Test RND(-1) - returns -1
                assert_eq!(lambda(&vec!["-1".to_string()]), "-1");

                // Test RND(0) - returns random number between 0 and 1
                let result = lambda(&vec!["0".to_string()]);
                let value = result.parse::<f64>().unwrap();
                assert!(value >= 0.0 && value < 1.0);
            }
            _ => panic!("Expected number function"),
        }
    }
}