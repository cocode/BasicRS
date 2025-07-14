use std::collections::HashMap;
use crate::basic_types::BasicError;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

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

#[derive(Debug, Clone)]
pub enum FunctionType {
    Number,
    String,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: &'static str,
    pub function_type: FunctionType,
    pub arg_types: Vec<ArgType>,
    pub implementation: fn(&[String]) -> Result<String, BasicError>,
}

pub struct FunctionRegistry {
    functions: HashMap<&'static str, FunctionDef>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let mut registry = FunctionRegistry {
            functions: HashMap::new(),
        };
        
        // Register all built-in functions
        registry.register_math_functions();
        registry.register_string_functions();
        
        registry
    }
    
    fn register_math_functions(&mut self) {
        // ABS function
        self.functions.insert("ABS", FunctionDef {
            name: "ABS",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.abs().to_string())
            },
        });
        
        // ATN function
        self.functions.insert("ATN", FunctionDef {
            name: "ATN",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.atan().to_string())
            },
        });
        
        // COS function
        self.functions.insert("COS", FunctionDef {
            name: "COS",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.cos().to_string())
            },
        });
        
        // EXP function
        self.functions.insert("EXP", FunctionDef {
            name: "EXP",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.exp().to_string())
            },
        });
        
        // INT function
        self.functions.insert("INT", FunctionDef {
            name: "INT",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.floor().to_string())
            },
        });
        
        // LOG function
        self.functions.insert("LOG", FunctionDef {
            name: "LOG",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.ln().to_string())
            },
        });
        
        // RND function
        self.functions.insert("RND", FunctionDef {
            name: "RND",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                if value < 0.0 {
                    // Negative values seed the generator and return a random number
                    let seed = (value.abs() * 1000000.0) as u64;
                    let mut rng = StdRng::seed_from_u64(seed);
                    let result: f64 = rng.gen();
                    Ok(result.to_string())
                } else if value == 0.0 {
                    // Return random number between 0 and 1
                    let result: f64 = rand::thread_rng().gen();
                    Ok(result.to_string())
                } else {
                    // Return random number between 0 and 1
                    let result: f64 = rand::thread_rng().gen();
                    Ok(result.to_string())
                }
            },
        });
        
        // SGN function
        self.functions.insert("SGN", FunctionDef {
            name: "SGN",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                if value > 0.0 {
                    Ok("1".to_string())
                } else if value < 0.0 {
                    Ok("-1".to_string())
                } else {
                    Ok("0".to_string())
                }
            },
        });
        
        // SIN function
        self.functions.insert("SIN", FunctionDef {
            name: "SIN",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.sin().to_string())
            },
        });
        
        // SQR function
        self.functions.insert("SQR", FunctionDef {
            name: "SQR",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.sqrt().to_string())
            },
        });
        
        // TAN function
        self.functions.insert("TAN", FunctionDef {
            name: "TAN",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.tan().to_string())
            },
        });
    }
    
    fn register_string_functions(&mut self) {
        // ASC function
        self.functions.insert("ASC", FunctionDef {
            name: "ASC",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::String],
            implementation: |args| {
                let s = args[0].trim_matches('"');
                if s.is_empty() {
                    return Err(BasicError::Syntax {
                        message: "ASC requires a non-empty string".to_string(),
                        basic_line_number: None,
                        file_line_number: None,
                    });
                }
                let ascii_value = s.chars().next().unwrap() as u8;
                Ok(ascii_value.to_string())
            },
        });
        
        // CHR$ function
        self.functions.insert("CHR$", FunctionDef {
            name: "CHR$",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let ascii_value: f64 = args[0].parse().unwrap();
                let ascii_value = ascii_value as u8;
                Ok((ascii_value as char).to_string())
            },
        });
        
        // LEFT$ function
        self.functions.insert("LEFT$", FunctionDef {
            name: "LEFT$",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::String, ArgType::Number],
            implementation: |args| {
                let s = args[0].trim_matches('"');
                let len: f64 = args[1].parse().unwrap();
                let len = len as usize;
                let result = s.chars().take(len).collect::<String>();
                Ok(result)
            },
        });
        
        // LEN function
        self.functions.insert("LEN", FunctionDef {
            name: "LEN",
            function_type: FunctionType::Number,
            arg_types: vec![ArgType::String],
            implementation: |args| {
                let s = args[0].trim_matches('"');
                Ok(s.len().to_string())
            },
        });
        
        // MID$ function
        self.functions.insert("MID$", FunctionDef {
            name: "MID$",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::String, ArgType::Number, ArgType::Number],
            implementation: |args| {
                let s = args[0].trim_matches('"');
                let start: f64 = args[1].parse().unwrap();
                let len: f64 = args[2].parse().unwrap();
                let start = (start as usize).saturating_sub(1);
                let len = len as usize;
                let result = s.chars().skip(start).take(len).collect::<String>();
                Ok(result)
            },
        });
        
        // RIGHT$ function
        self.functions.insert("RIGHT$", FunctionDef {
            name: "RIGHT$",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::String, ArgType::Number],
            implementation: |args| {
                let s = args[0].trim_matches('"');
                let len: f64 = args[1].parse().unwrap();
                let len = len as usize;
                let start = s.len().saturating_sub(len);
                let result = s.chars().skip(start).collect::<String>();
                Ok(result)
            },
        });
        
        // SPACE$ function
        self.functions.insert("SPACE$", FunctionDef {
            name: "SPACE$",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let count: f64 = args[0].parse().unwrap();
                let count = count as usize;
                let result = " ".repeat(count);
                Ok(result)
            },
        });
        
        // STR$ function
        self.functions.insert("STR$", FunctionDef {
            name: "STR$",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let value: f64 = args[0].parse().unwrap();
                Ok(value.to_string())
            },
        });
        
        // TAB function - special case
        self.functions.insert("TAB", FunctionDef {
            name: "TAB",
            function_type: FunctionType::String,
            arg_types: vec![ArgType::Number],
            implementation: |args| {
                let column: f64 = args[0].parse().unwrap();
                let column = column as usize;
                // TAB is handled specially in the interpreter
                Ok(format!("TAB({})", column))
            },
        });
    }
    
    // Public API methods
    pub fn get_function(&self, name: &str) -> Option<&FunctionDef> {
        self.functions.get(name)
    }
    
    pub fn get_function_names(&self) -> Vec<&'static str> {
        self.functions.keys().cloned().collect()
    }
    
    pub fn get_numeric_function_names(&self) -> Vec<&'static str> {
        self.functions.iter()
            .filter(|(_, def)| matches!(def.function_type, FunctionType::Number))
            .map(|(name, _)| *name)
            .collect()
    }
    
    pub fn get_string_function_names(&self) -> Vec<&'static str> {
        self.functions.iter()
            .filter(|(_, def)| matches!(def.function_type, FunctionType::String))
            .map(|(name, _)| *name)
            .collect()
    }
    
    pub fn is_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    pub fn is_string_function(&self, name: &str) -> bool {
        self.functions.get(name)
            .map(|def| matches!(def.function_type, FunctionType::String))
            .unwrap_or(false)
    }
    
    pub fn is_numeric_function(&self, name: &str) -> bool {
        self.functions.get(name)
            .map(|def| matches!(def.function_type, FunctionType::Number))
            .unwrap_or(false)
    }
    
    pub fn call_function(&self, name: &str, args: &[String]) -> Result<String, BasicError> {
        if let Some(func_def) = self.functions.get(name) {
            (func_def.implementation)(args)
        } else {
            Err(BasicError::Runtime {
                message: format!("Unknown function: {}", name),
                basic_line_number: None,
                file_line_number: None,
            })
        }
    }
    
    pub fn get_arg_types(&self, name: &str) -> Option<&[ArgType]> {
        self.functions.get(name).map(|def| def.arg_types.as_slice())
    }
    
    pub fn get_arg_count(&self, name: &str) -> Option<usize> {
        self.functions.get(name).map(|def| def.arg_types.len())
    }

    /// Call a numeric function with f64 arguments (for interpreter use)
    pub fn call_numeric_function(&self, name: &str, args: &[f64]) -> Option<f64> {
        if self.is_numeric_function(name) {
            let string_args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
            if let Ok(result) = self.call_function(name, &string_args) {
                result.parse::<f64>().ok()
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Call a function with Token arguments and return a Token result
    pub fn call_function_with_tokens(&self, name: &str, args: Vec<crate::basic_types::Token>) -> Result<crate::basic_types::Token, BasicError> {
        use crate::basic_types::{Token, IdentifierType};
        
        if let Some(func_def) = self.get_function(name) {
            // Convert tokens to strings
            let arg_strings: Vec<String> = args
                .into_iter()
                .map(|t| match t {
                    Token::Number(n) => Ok(n),
                    Token::String(s) => Ok(s),
                    Token::Identifier(name, IdentifierType::Variable) => Ok(name),
                    _ => Err(BasicError::Runtime {
                        message: format!("Invalid token: {:?}", t),
                        basic_line_number: None,
                        file_line_number: None,
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            
            // Call the function
            let result = self.call_function(name, &arg_strings)?;
            
            // Return appropriate token type
            match func_def.function_type {
                FunctionType::Number => Ok(Token::new_number(&result)),
                FunctionType::String => Ok(Token::new_string(&result)),
            }
        } else {
            Err(BasicError::Runtime {
                message: format!("Unknown function '{}'", name),
                basic_line_number: None,
                file_line_number: None,
            })
        }
    }
}

// Global singleton instance
lazy_static::lazy_static! {
    pub static ref FUNCTION_REGISTRY: FunctionRegistry = FunctionRegistry::new();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_registry_has_all_functions() {
        let registry = FunctionRegistry::new();
        
        // Test math functions
        assert!(registry.is_function("ABS"));
        assert!(registry.is_function("ATN"));
        assert!(registry.is_function("COS"));
        assert!(registry.is_function("EXP"));
        assert!(registry.is_function("INT"));
        assert!(registry.is_function("LOG"));
        assert!(registry.is_function("RND"));
        assert!(registry.is_function("SGN"));
        assert!(registry.is_function("SIN"));
        assert!(registry.is_function("SQR"));
        assert!(registry.is_function("TAN"));
        
        // Test string functions
        assert!(registry.is_function("ASC"));
        assert!(registry.is_function("CHR$"));
        assert!(registry.is_function("LEFT$"));
        assert!(registry.is_function("LEN"));
        assert!(registry.is_function("MID$"));
        assert!(registry.is_function("RIGHT$"));
        assert!(registry.is_function("SPACE$"));
        assert!(registry.is_function("STR$"));
        assert!(registry.is_function("TAB"));
    }
    
    #[test]
    fn test_function_type_classification() {
        let registry = FunctionRegistry::new();
        
        // Test numeric functions
        assert!(registry.is_numeric_function("ABS"));
        assert!(registry.is_numeric_function("SIN"));
        assert!(registry.is_numeric_function("LEN"));
        assert!(registry.is_numeric_function("ASC"));
        
        // Test string functions
        assert!(registry.is_string_function("CHR$"));
        assert!(registry.is_string_function("LEFT$"));
        assert!(registry.is_string_function("MID$"));
        assert!(registry.is_string_function("RIGHT$"));
        assert!(registry.is_string_function("SPACE$"));
        assert!(registry.is_string_function("STR$"));
        assert!(registry.is_string_function("TAB"));
    }
    
    #[test]
    fn test_abs_function() {
        let registry = FunctionRegistry::new();
        let result = registry.call_function("ABS", &["-5".to_string()]).unwrap();
        assert_eq!(result, "5");
    }
    
    #[test]
    fn test_chr_function() {
        let registry = FunctionRegistry::new();
        let result = registry.call_function("CHR$", &["65".to_string()]).unwrap();
        assert_eq!(result, "A");
    }
    
    #[test]
    fn test_len_function() {
        let registry = FunctionRegistry::new();
        let result = registry.call_function("LEN", &["\"Hello\"".to_string()]).unwrap();
        assert_eq!(result, "5");
    }
    
    #[test]
    fn test_get_function_names() {
        let registry = FunctionRegistry::new();
        let names = registry.get_function_names();
        assert!(names.contains(&"ABS"));
        assert!(names.contains(&"CHR$"));
        assert!(names.len() > 10);
    }
    
    #[test]
    fn test_get_numeric_function_names() {
        let registry = FunctionRegistry::new();
        let names = registry.get_numeric_function_names();
        assert!(names.contains(&"ABS"));
        assert!(names.contains(&"SIN"));
        assert!(names.contains(&"LEN"));
        assert!(!names.contains(&"CHR$"));
    }
    
    #[test]
    fn test_get_string_function_names() {
        let registry = FunctionRegistry::new();
        let names = registry.get_string_function_names();
        assert!(names.contains(&"CHR$"));
        assert!(names.contains(&"LEFT$"));
        assert!(!names.contains(&"ABS"));
    }
    
    #[test]
    fn test_get_arg_count() {
        let registry = FunctionRegistry::new();
        assert_eq!(registry.get_arg_count("ABS"), Some(1));
        assert_eq!(registry.get_arg_count("LEFT$"), Some(2));
        assert_eq!(registry.get_arg_count("MID$"), Some(3));
        assert_eq!(registry.get_arg_count("NONEXISTENT"), None);
    }
} 