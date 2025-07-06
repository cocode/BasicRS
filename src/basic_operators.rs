use std::collections::HashMap;
use crate::basic_types::{Token, BasicError, SymbolType};

#[derive(Debug, Clone)]
pub enum Op {
    Mono(MonoOp),
    StrMono(StrMonoOp),
    StrDollar(StrDollarMonoOp),
    Str(StrOp),
}

impl Op {
    pub fn eval(&self, stack: &mut Vec<Token>, _op: Option<&OpOperation>) -> Result<Token, BasicError> {
        match self {
            Op::Mono(op) => op.eval(stack, None),
            Op::StrMono(op) => op.eval(stack, None),
            Op::StrDollar(op) => op.eval(stack, None),
            Op::Str(op) => op.eval(stack, None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonoOp {
    lambda: fn(f64) -> f64,
    return_type: Option<String>,
}

impl MonoOp {
    pub fn new(lambda: fn(f64) -> f64) -> Self {
        MonoOp {
            lambda,
            return_type: None,
        }
    }

    fn check_args(&self, stack: &[Token]) -> Result<(), BasicError> {
        if stack.len() < 1 {
            return Err(BasicError::Syntax {
                message: "Not enough operands for unary operator".to_string(),
                line_number: None,
            });
        }
        Ok(())
    }

    fn eval(&self, stack: &mut Vec<Token>, _op: Option<&OpOperation>) -> Result<Token, BasicError> {
        self.check_args(stack)?;
        let first = stack.pop().unwrap();
        
        // Extract numeric value from token
        let value = match &first {
            Token::Number(n) => n.parse::<f64>().map_err(|_| BasicError::Type {
                message: "Invalid number format".to_string(),
                line_number: None,
            })?,
            _ => return Err(BasicError::Type {
                message: "Expected number for unary operation".to_string(),
                line_number: None,
            }),
        };
        
        let answer = (self.lambda)(value);
        Ok(Token::Number(answer.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct StrMonoOp {
    lambda: fn(String) -> String,
    return_type: String,
}

impl StrMonoOp {
    pub fn new(lambda: fn(String) -> String, return_type: &str) -> Self {
        StrMonoOp {
            lambda,
            return_type: return_type.to_string(),
        }
    }

    fn eval(&self, stack: &mut Vec<Token>, _op: Option<&OpOperation>) -> Result<Token, BasicError> {
        let first = stack.pop().unwrap();
        
        // Extract string value from token
        let value = match &first {
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            Token::Number(n) => n.clone(),
            _ => return Err(BasicError::Type {
                message: "Cannot convert token to string".to_string(),
                line_number: None,
            }),
        };
        
        let answer = (self.lambda)(value);
        if self.return_type == "string" {
            Ok(Token::String(answer))
        } else {
            Ok(Token::Number(answer))
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrDollarMonoOp {
    lambda: fn(String) -> String,
    return_type: String,
}

impl StrDollarMonoOp {
    pub fn new(lambda: fn(String) -> String, return_type: &str) -> Self {
        StrDollarMonoOp {
            lambda,
            return_type: return_type.to_string(),
        }
    }

    fn eval(&self, stack: &mut Vec<Token>, _op: Option<&OpOperation>) -> Result<Token, BasicError> {
        let first = stack.pop().unwrap();
        
        // Extract and format value from token
        let value = match &first {
            Token::Number(n) => {
                if let Ok(num) = n.parse::<f64>() {
                    if num.fract() == 0.0 {
                        num.trunc().to_string()
                    } else {
                        num.to_string()
                    }
                } else {
                    n.clone()
                }
            },
            Token::String(s) => s.clone(),
            Token::Identifier(s) => s.clone(),
            _ => return Err(BasicError::Type {
                message: "Cannot convert token to string".to_string(),
                line_number: None,
            }),
        };
        
        let answer = (self.lambda)(value);
        if self.return_type == "string" {
            Ok(Token::String(answer))
        } else {
            Ok(Token::Number(answer))
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrOp {
    lambda: fn(Vec<String>) -> String,
    name: String,
    arg_count: usize,
    return_type: Option<String>,
}

impl StrOp {
    pub fn new(lambda: fn(Vec<String>) -> String, name: &str, arg_count: usize, return_type: Option<&str>) -> Self {
        StrOp {
            lambda,
            name: name.to_string(),
            arg_count,
            return_type: return_type.map(|s| s.to_string()),
        }
    }

    fn check_args(&self, stack: &[Token]) -> Result<(), BasicError> {
        if stack.len() < self.arg_count {
            return Err(BasicError::Syntax {
                message: format!("Not enough operands for {}", self.name),
                line_number: None,
            });
        }
        Ok(())
    }

    fn eval(&self, stack: &mut Vec<Token>, _op: Option<&OpOperation>) -> Result<Token, BasicError> {
        self.check_args(stack)?;
        let mut args = Vec::new();
        for _ in 0..self.arg_count {
            if let Some(token) = stack.pop() {
                // Extract string value from token
                let value = match &token {
                    Token::String(s) => s.clone(),
                    Token::Number(n) => n.clone(),
                    Token::Identifier(s) => s.clone(),
                    _ => return Err(BasicError::Type {
                        message: "Cannot convert token to string".to_string(),
                        line_number: None,
                    }),
                };
                args.push(value);
            }
        }
        args.reverse(); // Reverse to maintain correct order
        let answer = (self.lambda)(args);
        
        // Check for special prefixes that indicate dynamic return types
        if answer.starts_with("NUMBER:") {
            let number_part = &answer[7..]; // Remove "NUMBER:" prefix
            Ok(Token::Number(number_part.to_string()))
        } else if answer.starts_with("STRING:") {
            let string_part = &answer[7..]; // Remove "STRING:" prefix
            Ok(Token::String(string_part.to_string()))
        } else if let Some(return_type) = &self.return_type {
            if return_type == "string" {
                Ok(Token::String(answer))
            } else {
                Ok(Token::Number(answer))
            }
        } else {
            Ok(Token::String(answer))
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpOperation {
    pub token: String,
    pub op_type: String,
    pub arg: Option<String>,
    pub value: Option<String>,
    pub symbols: Option<HashMap<String, SymbolType>>,
}

#[derive(Debug, Clone)]
pub struct OpDef {
    pub text: String,
    pub precedence: i32,
    pub op: Op,
}

lazy_static::lazy_static! {
    static ref OPERATORS: HashMap<String, OpDef> = {
        let mut m = HashMap::new();
        
        // Exponentiation (highest precedence)
        m.insert("^".to_string(), OpDef {
            text: "^".to_string(),
            precedence: 7,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                a.powf(b).to_string()
            }, "^", 2, Some("number"))),
        });

        // Multiplication and division
        m.insert("*".to_string(), OpDef {
            text: "*".to_string(),
            precedence: 6,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                (a * b).to_string()
            }, "*", 2, Some("number"))),
        });

        m.insert("/".to_string(), OpDef {
            text: "/".to_string(),
            precedence: 6,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if b == 0.0 {
                    "Division by zero".to_string()
                } else {
                    (a / b).to_string()
                }
            }, "/", 2, Some("number"))),
        });

        // Addition and subtraction
        m.insert("+".to_string(), OpDef {
            text: "+".to_string(),
            precedence: 5,
            op: Op::Str(StrOp::new(|args| {
                // Check if both arguments are numeric
                if let (Ok(a), Ok(b)) = (args[0].parse::<f64>(), args[1].parse::<f64>()) {
                    // Return numeric result - but we need to signal this is a number
                    format!("NUMBER:{}", (a + b).to_string())
                } else {
                    // String concatenation
                    let mut result = args[0].clone();
                    if result.starts_with('"') && result.ends_with('"') {
                        result = result[1..result.len()-1].to_string();
                    }
                    let mut second = args[1].clone();
                    if second.starts_with('"') && second.ends_with('"') {
                        second = second[1..second.len()-1].to_string();
                    }
                    format!("STRING:\"{}{}\"", result, second)
                }
            }, "+", 2, None)),
        });

        m.insert("-".to_string(), OpDef {
            text: "-".to_string(),
            precedence: 5,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                (a - b).to_string()
            }, "-", 2, Some("number"))),
        });

        // Comparison operators
        m.insert("=".to_string(), OpDef {
            text: "=".to_string(),
            precedence: 4,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if a == b { "-1" } else { "0" }.to_string()
            }, "=", 2, Some("number"))),
        });

        m.insert("<>".to_string(), OpDef {
            text: "<>".to_string(),
            precedence: 4,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if a != b { "-1" } else { "0" }.to_string()
            }, "<>", 2, Some("number"))),
        });

        m.insert("<".to_string(), OpDef {
            text: "<".to_string(),
            precedence: 4,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if a < b { "-1" } else { "0" }.to_string()
            }, "<", 2, Some("number"))),
        });

        m.insert(">".to_string(), OpDef {
            text: ">".to_string(),
            precedence: 4,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if a > b { "-1" } else { "0" }.to_string()
            }, ">", 2, Some("number"))),
        });

        m.insert("<=".to_string(), OpDef {
            text: "<=".to_string(),
            precedence: 4,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if a <= b { "-1" } else { "0" }.to_string()
            }, "<=", 2, Some("number"))),
        });

        m.insert(">=".to_string(), OpDef {
            text: ">=".to_string(),
            precedence: 4,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0);
                let b = args[1].parse::<f64>().unwrap_or(0.0);
                if a >= b { "-1" } else { "0" }.to_string()
            }, ">=", 2, Some("number"))),
        });

        // Logical operators
        m.insert("AND".to_string(), OpDef {
            text: "AND".to_string(),
            precedence: 3,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0) != 0.0;
                let b = args[1].parse::<f64>().unwrap_or(0.0) != 0.0;
                if a && b { "-1" } else { "0" }.to_string()
            }, "AND", 2, Some("number"))),
        });

        m.insert("OR".to_string(), OpDef {
            text: "OR".to_string(),
            precedence: 2,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0) != 0.0;
                let b = args[1].parse::<f64>().unwrap_or(0.0) != 0.0;
                if a || b { "-1" } else { "0" }.to_string()
            }, "OR", 2, Some("number"))),
        });

        m.insert("NOT".to_string(), OpDef {
            text: "NOT".to_string(),
            precedence: 1,
            op: Op::Str(StrOp::new(|args| {
                let a = args[0].parse::<f64>().unwrap_or(0.0) != 0.0;
                if !a { "-1" } else { "0" }.to_string()
            }, "NOT", 1, Some("number"))),
        });

        m
    };
}


pub fn get_op_def(operator: &str) -> Option<&'static OpDef> {
    (*OPERATORS).get(operator)
}
pub fn get_precedence(token: &Token) -> i32 {
    // Extract operator string from token
    let op_str = match token {
        Token::Plus => "+",
        Token::Minus => "-",
        Token::Star => "*",
        Token::Slash => "/",
        Token::Power => "^",
        Token::Equal => "=",
        Token::NotEqual => "<>",
        Token::Less => "<",
        Token::LessEqual => "<=",
        Token::Greater => ">",
        Token::GreaterEqual => ">=",
        Token::And => "AND",
        Token::Or => "OR",
        Token::Not => "NOT",
        _ => return 0,
    };
    
    if let Some(op_def) = get_op_def(op_str) {
        op_def.precedence
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::Token;

    fn create_number_token(value: &str) -> Token {
        Token::Number(value.to_string())
    }

    fn create_string_token(value: &str) -> Token {
        Token::String(value.to_string())
    }

    #[test]
    fn test_arithmetic_operators() {
        // Test exponentiation
        let op = get_op_def("^").unwrap();
        let mut stack = vec![
            create_number_token("2"),
            create_number_token("3"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "8");
        } else {
            panic!("Expected number token");
        }

        // Test multiplication
        let op = get_op_def("*").unwrap();
        let mut stack = vec![
            create_number_token("4"),
            create_number_token("5"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "20");
        } else {
            panic!("Expected number token");
        }

        // Test division
        let op = get_op_def("/").unwrap();
        let mut stack = vec![
            create_number_token("10"),
            create_number_token("2"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "5");
        } else {
            panic!("Expected number token");
        }

        // Test division by zero
        let mut stack = vec![
            create_number_token("10"),
            create_number_token("0"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "Division by zero");
        } else {
            panic!("Expected number token");
        }

        // Test addition
        let op = get_op_def("+").unwrap();
        let mut stack = vec![
            create_number_token("6"),
            create_number_token("7"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "13");
        } else {
            panic!("Expected number token");
        }

        // Test string concatenation
        let mut stack = vec![
            create_string_token("Hello "),
            create_string_token("World"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::String(s) = result {
            assert_eq!(s, "\"Hello World\"");
        } else {
            panic!("Expected string token");
        }

        // Test subtraction
        let op = get_op_def("-").unwrap();
        let mut stack = vec![
            create_number_token("10"),
            create_number_token("3"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "7");
        } else {
            panic!("Expected number token");
        }
    }

    #[test]
    fn test_comparison_operators() {
        // Test equals
        let op = get_op_def("=").unwrap();
        let mut stack = vec![
            create_number_token("5"),
            create_number_token("5"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test not equals
        let op = get_op_def("<>").unwrap();
        let mut stack = vec![
            create_number_token("5"),
            create_number_token("6"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test less than
        let op = get_op_def("<").unwrap();
        let mut stack = vec![
            create_number_token("5"),
            create_number_token("6"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test greater than
        let op = get_op_def(">").unwrap();
        let mut stack = vec![
            create_number_token("7"),
            create_number_token("6"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test less than or equal
        let op = get_op_def("<=").unwrap();
        let mut stack = vec![
            create_number_token("5"),
            create_number_token("5"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test greater than or equal
        let op = get_op_def(">=").unwrap();
        let mut stack = vec![
            create_number_token("6"),
            create_number_token("5"),
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }
    }

    #[test]
    fn test_logical_operators() {
        // Test AND
        let op = get_op_def("AND").unwrap();
        let mut stack = vec![
            create_number_token("-1"), // True in BASIC
            create_number_token("-1"), // True in BASIC
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test OR
        let op = get_op_def("OR").unwrap();
        let mut stack = vec![
            create_number_token("-1"), // True in BASIC
            create_number_token("0"),  // False in BASIC
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }

        // Test NOT
        let op = get_op_def("NOT").unwrap();
        let mut stack = vec![
            create_number_token("0"),  // False in BASIC
        ];
        let result = op.op.eval(&mut stack, None).unwrap();
        if let Token::Number(n) = result {
            assert_eq!(n, "-1"); // True in BASIC
        } else {
            panic!("Expected number token");
        }
    }

    #[test]
    fn test_operator_precedence() {
        assert!(get_op_def("^").unwrap().precedence > get_op_def("*").unwrap().precedence);
        assert!(get_op_def("*").unwrap().precedence > get_op_def("+").unwrap().precedence);
        assert!(get_op_def("+").unwrap().precedence > get_op_def("=").unwrap().precedence);
        assert!(get_op_def("=").unwrap().precedence > get_op_def("AND").unwrap().precedence);
        assert!(get_op_def("AND").unwrap().precedence > get_op_def("OR").unwrap().precedence);
        assert!(get_op_def("OR").unwrap().precedence > get_op_def("NOT").unwrap().precedence);
    }
} 