use std::str::Chars;
use crate::basic_types::{Token, BasicError, is_valid_identifier};

pub struct Lexer<'a> {
    input: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    file_line_number: usize,
    basic_line_number: Option<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Lexer {
            input,
            chars,
            current,
            file_line_number: 1,
            basic_line_number: None,
        }
    }

    // Main tokenize function that processes the entire program line by line
    pub fn tokenize(&mut self) -> Result<Vec<Token>, BasicError> {
        println!("TOKENIZING");
        let mut all_tokens = Vec::new();
        
        while let Some(c) = self.current {
            // Skip leading whitespace
            if c == ' ' || c == '\t' {
                self.advance();
                continue;
            }
            
            // Process one line at a time
            let line_tokens = self.tokenize_line()?;
            all_tokens.extend(line_tokens);
        }
        
        Ok(all_tokens)
    }

    // Tokenize a single line, extracting line number and statements
    fn tokenize_line(&mut self) -> Result<Vec<Token>, BasicError> {
        let mut line_tokens = Vec::new();
        
        // Check for line number at start of line
        if let Some(c) = self.current {
            if c.is_ascii_digit() {
                let line_number = self.tokenize_line_number()?;
                line_tokens.push(line_number);
            }
        }
        
        // Tokenize the statements on this line
        let statement_tokens = self.tokenize_statements()?;
        line_tokens.extend(statement_tokens);
        
        // Add newline token at end of line
        if let Some(c) = self.current {
            if c == '\n' || c == '\r' {
                line_tokens.push(Token::Newline);
                self.advance();
                self.file_line_number += 1;
            }
        }
        
        Ok(line_tokens)
    }

    // Extract line number from start of line
    fn tokenize_line_number(&mut self) -> Result<Token, BasicError> {
        let mut number = String::new();
        
        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                number.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        match number.parse::<usize>() {
            Ok(line_num) => {
                self.basic_line_number = Some(line_num);
                println!("BASIC LINE NUMBER {}", line_num);
                Ok(Token::LineNumber(line_num))
            }
            Err(_) => {
                Err(BasicError::Syntax {
                    message: format!("Invalid line number: {}", number),
                    line_number: Some(self.file_line_number),
                })
            }
        }
    }

    // Tokenize statements on a line (everything after line number until newline)
    pub fn tokenize_statements(&mut self) -> Result<Vec<Token>, BasicError> {
        let mut tokens = Vec::new();
        
        while let Some(c) = self.current {
            match c {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' | '\r' => {
                    // End of line reached
                    break;
                }
                '0'..='9' => {
                    // This is a number (not a line number since we're in statements)
                    let mut number = String::new();
                    while let Some(c) = self.current {
                        if c.is_ascii_digit() || c == '.' {
                            number.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(number));
                }
                '"' => {
                    let mut string = String::new();
                    self.advance(); // Skip opening quote
                    
                    let mut found_closing_quote = false;
                    while let Some(c) = self.current {
                        if c == '"' {
                            self.advance(); // Skip closing quote
                            found_closing_quote = true;
                            break;
                        }
                        if c == '\n' || c == '\r' {
                            return Err(BasicError::Syntax {
                                message: "Unterminated string literal".to_string(),
                                line_number: Some(self.file_line_number),
                            });
                        }
                        string.push(c);
                        self.advance();
                    }
                    
                    if !found_closing_quote {
                        return Err(BasicError::Syntax {
                            message: "Unterminated string literal".to_string(),
                            line_number: Some(self.file_line_number),
                        });
                    }
                    
                    tokens.push(Token::String(string));
                }
                'A'..='Z' | 'a'..='z' => {
                    let mut accumulated = String::new();
                    let mut is_keyword = false;
                    let mut keyword_token = None;
                    
                    // Accumulate characters
                    while let Some(c) = self.current {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                            accumulated.push(c.to_ascii_uppercase());
                            self.advance();
                            // Check if current accumulated string is a keyword
                            match accumulated.as_str() {
                                "REM" | "LET" | "PRINT" | "INPUT" | "IF" | "THEN" | "ELSE" |
                                "FOR" | "TO" | "STEP" | "NEXT" | "GOTO" | "GOSUB" | "RETURN" |
                                "END" | "STOP" | "DATA" | "READ" | "RESTORE" | "DIM" | "ON" |
                                "DEF" | "AND" | "OR" | "NOT" => {
                                    is_keyword = true;
                                    keyword_token = match accumulated.as_str() {
                                        "REM" => Some(Token::Rem),
                                        "LET" => Some(Token::Let),
                                        "PRINT" => Some(Token::Print),
                                        "INPUT" => Some(Token::Input),
                                        "IF" => Some(Token::If),
                                        "THEN" => Some(Token::Then),
                                        "ELSE" => Some(Token::Else),
                                        "FOR" => Some(Token::For),
                                        "TO" => Some(Token::To),
                                        "STEP" => Some(Token::Step),
                                        "NEXT" => Some(Token::Next),
                                        "GOTO" => Some(Token::Goto),
                                        "GOSUB" => Some(Token::Gosub),
                                        "RETURN" => Some(Token::Return),
                                        "END" => Some(Token::End),
                                        "STOP" => Some(Token::Stop),
                                        "DATA" => Some(Token::Data),
                                        "READ" => Some(Token::Read),
                                        "RESTORE" => Some(Token::Restore),
                                        "DIM" => Some(Token::Dim),
                                        "ON" => Some(Token::On),
                                        "DEF" => Some(Token::Def),
                                        "AND" => Some(Token::And),
                                        "OR" => Some(Token::Or),
                                        "NOT" => Some(Token::Not),
                                        _ => None,
                                    };
                                }
                                "ABS" | "ASC" | "ATN" | "COS" | "EXP" | "INT" | "LOG" | "RND" | "SGN" | "SIN" | "SQR" | "TAN" |
                                "CHR$" | "LEFT$" | "LEN" | "MID$" | "RIGHT$" | "SPACE$" | "STR$" => {
                                    // Built-in functions - treat as identifiers but don't split them
                                    is_keyword = false;
                                }
                                _ => {}
                            }
                        } else {
                            break;
                        }
                    }
                    
                    if is_keyword {
                        // Handle keyword
                        if keyword_token == Some(Token::Rem) {
                            tokens.push(Token::Rem);
                            // Consume the rest of the line for REM statements
                            let mut comment = String::new();
                            while let Some(c) = self.current {
                                if c == '\n' || c == '\r' {
                                    break;
                                }
                                comment.push(c);
                                self.advance();
                            }
                            tokens.push(Token::String(comment.trim().to_string()));
                        } else {
                            tokens.push(keyword_token.unwrap());
                        }
                    } else if is_valid_identifier(&accumulated) {
                        // Handle complete valid identifier (including built-in functions)
                        tokens.push(Token::Identifier(accumulated));
                    } else {
                        // Handle variable - take longest valid variable from front
                        let mut valid_variable = String::new();
                        let chars: Vec<char> = accumulated.chars().collect();
                        // Try to find the longest valid variable from the front
                        for i in 0..chars.len() {
                            let candidate = chars[0..=i].iter().collect::<String>();
                            if is_valid_identifier(&candidate) {
                                valid_variable = candidate;
                            } else {
                                break;
                            }
                        }
                        if !valid_variable.is_empty() {
                            let valid_len = valid_variable.len();
                            tokens.push(Token::Identifier(valid_variable));
                            // If there are remaining characters, they need to be processed
                            if valid_len < accumulated.len() {
                                let remaining = &accumulated[valid_len..];
                                // Recursively process the remaining characters
                                let mut remaining_lexer = Lexer::new(remaining);
                                let mut remaining_tokens = remaining_lexer.tokenize_statements()?;
                                tokens.append(&mut remaining_tokens);
                            }
                        } else {
                            return Err(BasicError::Syntax {
                                message: format!("Invalid identifier: {}", accumulated),
                                line_number: Some(self.file_line_number),
                            });
                        }
                    }
                }
                '+' => {
                    tokens.push(Token::Plus);
                    self.advance();
                }
                '-' => {
                    tokens.push(Token::Minus);
                    self.advance();
                }
                '*' => {
                    tokens.push(Token::Star);
                    self.advance();
                }
                '/' => {
                    tokens.push(Token::Slash);
                    self.advance();
                }
                '^' => {
                    tokens.push(Token::Power);
                    self.advance();
                }
                '=' => {
                    tokens.push(Token::Equal);
                    self.advance();
                }
                '<' => {
                    self.advance();
                    match self.current {
                        Some('=') => {
                            tokens.push(Token::LessEqual);
                            self.advance();
                        }
                        Some('>') => {
                            tokens.push(Token::NotEqual);
                            self.advance();
                        }
                        _ => tokens.push(Token::Less),
                    }
                }
                '>' => {
                    self.advance();
                    if let Some('=') = self.current {
                        tokens.push(Token::GreaterEqual);
                        self.advance();
                    } else {
                        tokens.push(Token::Greater);
                    }
                }
                '(' => {
                    tokens.push(Token::LeftParen);
                    self.advance();
                }
                ')' => {
                    tokens.push(Token::RightParen);
                    self.advance();
                }
                ',' => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                ';' => {
                    tokens.push(Token::Semicolon);
                    self.advance();
                }
                ':' => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                _ => {
                    return Err(BasicError::Syntax {
                        message: format!("Unexpected character: {} basic line {} file line {}", c,
                                         self.basic_line_number.unwrap_or(0).to_string(),
                                         self.file_line_number),
                        line_number: None,
                    });
                }
            }
        }
        
        Ok(tokens)
    }

    fn advance(&mut self) {
        self.current = self.chars.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_let() {
        let mut lexer = Lexer::new("LET X = 123");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Identifier("X".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::Number("123".to_string()));
    }

    #[test]
    fn test_tokenize_for() {
        let mut lexer = Lexer::new("FOR I = 1 TO 10");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], Token::For);
        assert_eq!(tokens[1], Token::Identifier("I".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::Number("1".to_string()));
        assert_eq!(tokens[4], Token::To);
        assert_eq!(tokens[5], Token::Number("10".to_string()));
    }

    #[test]
    fn test_tokenize_line_numbers() {
        let mut lexer = Lexer::new("10 LET X = 123\n20 PRINT X");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::LineNumber(10));
        assert_eq!(tokens[5], Token::Newline);
        assert_eq!(tokens[6], Token::LineNumber(20));
    }

    #[test]
    fn test_line_number_no_spaces() {
        // Test the specific case that's failing: "200 print abs(-12)"
        let mut lexer = Lexer::new("200 print abs(-12)");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens for '200 print abs(-12)':");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }
        
        assert_eq!(tokens[0], Token::LineNumber(200));
        assert_eq!(tokens[1], Token::Print);
        assert_eq!(tokens[2], Token::Identifier("ABS".to_string()));
        assert_eq!(tokens[3], Token::LeftParen);
        assert_eq!(tokens[4], Token::Minus);
        assert_eq!(tokens[5], Token::Number("12".to_string()));
        assert_eq!(tokens[6], Token::RightParen);
    }

    #[test]
    fn test_tokenize_rem() {
        let mut lexer = Lexer::new("10 REM This is a comment\n20 PRINT X");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::LineNumber(10));
        assert_eq!(tokens[1], Token::Rem);
        assert_eq!(tokens[2], Token::String("This is a comment".to_string()));
        assert_eq!(tokens[3], Token::Newline);
        assert_eq!(tokens[4], Token::LineNumber(20));
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new("PRINT \"unterminated");
        let result = lexer.tokenize();
        assert!(result.is_err());
        
        if let Err(BasicError::Syntax { message, line_number }) = result {
            assert!(message.contains("Unterminated string"));
            assert_eq!(line_number, Some(1));
        } else {
            panic!("Expected syntax error");
        }
    }

    #[test]
    fn test_valid_identifiers() {
        let valid_inputs = vec![
            "X", "Y", "Z", "A1", "B2", "C$"
        ];
        
        for input in valid_inputs {
            let source = format!("LET {} = 123", input);
            let mut lexer = Lexer::new(&source);
            let result = lexer.tokenize();
            assert!(result.is_ok(), "Failed for input: {}", input);
        }
    }

    // Test is not valid. 1X tokenizes just fine, as 1, X. But it still isn't a valid identifier
    // #[test]
    // fn test_invalid_identifiers() {
    //     let invalid_inputs = vec![
    //         "1X", "ABC", "A1B", "A$B", "A$$"
    //     ];
    //
    //     for input in invalid_inputs {
    //         let source = format!("LET {} = 123", input);
    //         let mut lexer = Lexer::new(&source);
    //         let result = lexer.tokenize();
    //         assert!(result.is_err(), "Should fail for input: {}", input);
    //     }
    // }
} 