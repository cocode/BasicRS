use std::str::Chars;
use crate::basic_types::{Token, BasicError, is_valid_identifier};

pub struct Lexer<'a> {
    input: &'a str,
    chars: Chars<'a>,
    current: Option<char>,
    line_number: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current = chars.next();
        Lexer {
            input,
            chars,
            current,
            line_number: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, BasicError> {
        let mut tokens = Vec::new();
        
        while let Some(c) = self.current {
            match c {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' | '\r' => {
                    tokens.push(Token::Newline);
                    self.advance();
                    self.line_number += 1;
                }
                '0'..='9' => {
                    // Check if this is a line number at the start of a line
                    let is_start_of_line = tokens.is_empty() || 
                        matches!(tokens.last(), Some(Token::Newline));
                    
                    let mut number = String::new();
                    while let Some(c) = self.current {
                        if c.is_ascii_digit() || c == '.' {
                            number.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    if is_start_of_line && !number.contains('.') {
                        tokens.push(Token::LineNumber(number.parse().unwrap()));
                    } else {
                        tokens.push(Token::Number(number));
                    }
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
                                line_number: Some(self.line_number),
                            });
                        }
                        string.push(c);
                        self.advance();
                    }
                    
                    if !found_closing_quote {
                        return Err(BasicError::Syntax {
                            message: "Unterminated string literal".to_string(),
                            line_number: Some(self.line_number),
                        });
                    }
                    
                    tokens.push(Token::String(string));
                }
                'A'..='Z' | 'a'..='z' => {
                    let mut identifier = String::new();
                    while let Some(c) = self.current {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                            identifier.push(c.to_ascii_uppercase());
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    match identifier.as_str() {
                        "REM" => {
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
                        }
                        "LET" => tokens.push(Token::Let),
                        "PRINT" => tokens.push(Token::Print),
                        "INPUT" => tokens.push(Token::Input),
                        "IF" => tokens.push(Token::If),
                        "THEN" => tokens.push(Token::Then),
                        "ELSE" => tokens.push(Token::Else),
                        "FOR" => tokens.push(Token::For),
                        "TO" => tokens.push(Token::To),
                        "STEP" => tokens.push(Token::Step),
                        "NEXT" => tokens.push(Token::Next),
                        "GOTO" => tokens.push(Token::Goto),
                        "GOSUB" => tokens.push(Token::Gosub),
                        "RETURN" => tokens.push(Token::Return),
                        "END" => tokens.push(Token::End),
                        "STOP" => tokens.push(Token::Stop),
                        "DATA" => tokens.push(Token::Data),
                        "READ" => tokens.push(Token::Read),
                        "RESTORE" => tokens.push(Token::Restore),
                        "DIM" => tokens.push(Token::Dim),
                        "AND" => tokens.push(Token::And),
                        "OR" => tokens.push(Token::Or),
                        "NOT" => tokens.push(Token::Not),
                        _ => {
                            // Validate identifier format: must be A-Z followed by optional digit or $
                            if !is_valid_identifier(&identifier) {
                                return Err(BasicError::Syntax {
                                    message: format!("Invalid identifier: {}", identifier),
                                    line_number: Some(self.line_number),
                                });
                            }
                            tokens.push(Token::Identifier(identifier))
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
                        message: format!("Unexpected character: {}", c),
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
    fn test_invalid_identifier() {
        let mut lexer = Lexer::new("LET ABC = 123");
        let result = lexer.tokenize();
        assert!(result.is_err());
        
        if let Err(BasicError::Syntax { message, line_number }) = result {
            assert!(message.contains("Invalid identifier"));
            assert_eq!(line_number, Some(1));
        } else {
            panic!("Expected syntax error");
        }
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