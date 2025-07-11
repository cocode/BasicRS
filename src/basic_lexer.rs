use crate::basic_types::{Token, BasicError, is_valid_identifier, IdentifierType};

pub struct Lexer {
    chars: Vec<char>,
    position: usize,
    file_line_number: usize,
    basic_line_number: Option<usize>,
    last_rem_comment: Option<String>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        Lexer {
            chars,
            position: 0,
            file_line_number: 1,
            basic_line_number: None,
            last_rem_comment: None,
        }
    }

    // Main tokenize function that processes the entire program line by line
    pub fn tokenize(&mut self) -> Result<Vec<Token>, BasicError> {
        let mut all_tokens = Vec::new();
        
        while self.position < self.chars.len() {
            // Skip leading whitespace
            if self.current_char() == ' ' || self.current_char() == '\t' {
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
        if self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c.is_ascii_digit() {
                let line_number = self.tokenize_line_number()?;
                line_tokens.push(line_number);
            }
        }
        
        // Tokenize the statements on this line
        let statement_tokens = self.tokenize_statements()?;
        line_tokens.extend(statement_tokens);
        
        // Add newline token at end of line
        if self.position < self.chars.len() {
            let c = self.chars[self.position];
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
        
        while self.position < self.chars.len() {
            let c = self.chars[self.position];
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
                Ok(Token::LineNumber(line_num))
            }
            Err(_) => {
                Err(BasicError::Syntax {
                    message: format!("Invalid line number: {}", number),
                    basic_line_number: self.basic_line_number,
                    file_line_number: Some(self.file_line_number),
                })
            }
        }
    }

    // Tokenize statements on a line (everything after line number until newline)
    pub fn tokenize_statements(&mut self) -> Result<Vec<Token>, BasicError> {
        let mut tokens = Vec::new();
        
        while self.position < self.chars.len() {
            let c = self.chars[self.position];
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
                    while self.position < self.chars.len() {
                        let c = self.chars[self.position];
                        if c.is_ascii_digit() || c == '.' {
                            number.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(number));
                }
                '.' => {
                    // This is a decimal number starting with a decimal point
                    let mut number = String::new();
                    number.push('.');
                    self.advance();
                    while self.position < self.chars.len() {
                        let c = self.chars[self.position];
                        if c.is_ascii_digit() {
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
                    while self.position < self.chars.len() {
                        let c = self.chars[self.position];
                        if c == '"' {
                            self.advance(); // Skip closing quote
                            found_closing_quote = true;
                            break;
                        }
                        if c == '\n' || c == '\r' {
                            return Err(BasicError::Syntax {
                                message: "Unterminated string literal".to_string(),
                                basic_line_number: self.basic_line_number,
                                file_line_number: Some(self.file_line_number),
                            });
                        }
                        string.push(c);
                        self.advance();
                    }
                    
                    if !found_closing_quote {
                        return Err(BasicError::Syntax {
                            message: "Unterminated string literal".to_string(),
                            basic_line_number: self.basic_line_number,
                            file_line_number: Some(self.file_line_number),
                        });
                    }
                    
                    tokens.push(Token::String(string));
                }
                'A'..='Z' | 'a'..='z' => {
                    // New lookahead-based identifier parsing for BASIC
                    let token = self.tokenize_identifier_lookahead()?;
                    tokens.push(token);
                    // Special handling for REM: if last_rem_comment is set, push it as a string token
                    if let Some(comment) = self.last_rem_comment.take() {
                        tokens.push(Token::String(comment));
                        // After REM, the rest of the line is a comment, so break
                        break;
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
                    if self.position < self.chars.len() {
                        match self.chars[self.position] {
                            '=' => {
                                tokens.push(Token::LessEqual);
                                self.advance();
                            }
                            '>' => {
                                tokens.push(Token::NotEqual);
                                self.advance();
                            }
                            _ => tokens.push(Token::Less),
                        }
                    } else {
                        tokens.push(Token::Less);
                    }
                }
                '>' => {
                    self.advance();
                    if self.position < self.chars.len() && self.chars[self.position] == '=' {
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
                        message: format!("Unexpected character: '{}' basic line {} file line {}", c,
                                         self.basic_line_number.unwrap_or(0).to_string(),
                                         self.file_line_number),
                        basic_line_number: self.basic_line_number,
                        file_line_number: Some(self.file_line_number),
                    });
                }
            }
        }
        
        Ok(tokens)
    }

    // Helper methods for character array approach
    fn current_char(&self) -> char {
        if self.position < self.chars.len() {
            self.chars[self.position]
        } else {
            '\0' // End of input
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }



    // New lookahead-based identifier parsing for BASIC
    fn tokenize_identifier_lookahead(&mut self) -> Result<Token, BasicError> {
        let start_pos = self.position;
        let mut chars = Vec::new();
        
        // Collect all characters that could be part of the identifier
        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                chars.push(c.to_ascii_uppercase());
                self.advance();
            } else {
                break;
            }
        }
        
        let input_str: String = chars.iter().collect();
        
        // Step 1: Scan for keywords, functions, or user-defined functions
        if let Some(token) = self.try_match_keyword_or_function(&input_str) {
            // Special handling for REM
            if let Some(keyword_len) = self.get_keyword_length(&input_str) {
                let keyword = &input_str[..keyword_len];
                if keyword == "REM" {
                    self.position = start_pos + keyword_len;
                    // Emit REM token
                    // Collect the rest of the line as a comment
                    let mut comment = String::new();
                    while self.position < self.chars.len() {
                        let c = self.chars[self.position];
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        comment.push(c);
                        self.advance();
                    }
                    // Trim leading whitespace from the comment
                    let trimmed_comment = comment.trim_start().to_string();
                    self.last_rem_comment = Some(trimmed_comment);
                    return Ok(Token::Rem);
                } else {
                    self.position = start_pos + keyword_len;
                }
            }
            return Ok(token);
        }
        
        // Step 2: Scan for identifiers in length order: A1$, A1, A$, A
        if let Some((identifier, consumed_len)) = self.try_match_identifier(&input_str) {
            // Reset position to where we started plus the consumed length
            self.position = start_pos + consumed_len;
            return Ok(Token::Identifier(identifier, IdentifierType::Variable));
        }
        
        // If we get here, we couldn't match anything
        Err(BasicError::Syntax {
            message: format!("Invalid identifier: {}", input_str),
            basic_line_number: self.basic_line_number,
            file_line_number: Some(self.file_line_number),
        })
    }

    // Try to match keywords or functions
    fn try_match_keyword_or_function(&mut self, input: &str) -> Option<Token> {
        // Keywords
        let keywords = vec![
            ("REM", Token::Rem),
            ("LET", Token::Let),
            ("PRINT", Token::Print),
            ("INPUT", Token::Input),
            ("IF", Token::If),
            ("THEN", Token::Then),
            ("ELSE", Token::Else),
            ("FOR", Token::For),
            ("TO", Token::To),
            ("STEP", Token::Step),
            ("NEXT", Token::Next),
            ("GOTO", Token::Goto),
            ("GOSUB", Token::Gosub),
            ("RETURN", Token::Return),
            ("END", Token::End),
            ("STOP", Token::Stop),
            ("DATA", Token::Data),
            ("READ", Token::Read),
            ("RESTORE", Token::Restore),
            ("DIM", Token::Dim),
            ("ON", Token::On),
            ("DEF", Token::Def),
            ("AND", Token::And),
            ("OR", Token::Or),
            ("NOT", Token::Not),
        ];
        
        // Built-in functions
        let functions = vec![
            "ABS", "ASC", "ATN", "COS", "EXP", "INT", "LOG", "RND", "SGN", "SIN", "SQR", "TAN",
            "CHR$", "LEFT$", "LEN", "MID$", "RIGHT$", "SPACE$", "STR$", "TAB"
        ];
        
        // Try to match the longest keyword/function first
        for len in (1..=input.len()).rev() {
            let candidate = &input[..len];
            let candidate_upper = candidate.to_ascii_uppercase();
            // Check keywords
            for (keyword, token) in &keywords {
                if candidate_upper == *keyword {
                    // Special handling for REM
                    if *keyword == "REM" {
                        // Consume the rest of the line for REM statements
                        let mut comment = String::new();
                        while self.position < self.chars.len() {
                            let c = self.chars[self.position];
                            if c == '\n' || c == '\r' {
                                break;
                            }
                            comment.push(c);
                            self.advance();
                        }
                        // Return the REM token, the comment will be handled separately
                        return Some(Token::Rem);
                    }
                    return Some(token.clone());
                }
            }
            // Check functions
            for function in &functions {
                if candidate_upper == *function {
                    return Some(Token::Identifier(candidate_upper.clone(), IdentifierType::BuiltInFunction));
                }
            }
            // Check user-defined function pattern: FNX
            if candidate_upper.len() == 3 && &candidate_upper[0..2] == "FN" && candidate_upper.chars().nth(2).unwrap().is_ascii_uppercase() {
                return Some(Token::Identifier(candidate_upper, IdentifierType::UserDefinedFunction));
            }
        }
        
        None
    }

    // Try to match identifiers in length order: A1$, A1, A$, A
        fn try_match_identifier(&self, input: &str) -> Option<(String, usize)> {
            // Try different identifier patterns in order of preference
            let patterns = vec![
                // A1$ - letter + digit + $
                (r"^[A-Z]\d\$", 3),
                // A1 - letter + digit
                (r"^[A-Z]\d", 2),
                // A$ - letter + $
                (r"^[A-Z]\$", 2),
                // A - single letter
                (r"^[A-Z]", 1),
            ];
            for (pattern, min_len) in patterns {
                // Try longest match first for each pattern
                for len in (min_len..=input.len()).rev() {
                    let candidate = &input[..len];
                    if self.matches_pattern(candidate, pattern) && is_valid_identifier(candidate) {
                        return Some((candidate.to_string(), len));
                    }
                }
            }
        
        None
    }

    // Simple pattern matching (we could use regex, but this is simpler for BASIC)
    fn matches_pattern(&self, input: &str, pattern: &str) -> bool {
        if input.is_empty() {
            return false;
        }
        
        let chars: Vec<char> = input.chars().collect();
        
        match pattern {
            r"^[A-Z]\d\$" => {
                chars.len() >= 3 && 
                chars[0].is_ascii_uppercase() && 
                chars[1].is_ascii_digit() && 
                chars[2] == '$'
            }
            r"^[A-Z]\d" => {
                chars.len() >= 2 && 
                chars[0].is_ascii_uppercase() && 
                chars[1].is_ascii_digit()
            }
            r"^[A-Z]\$" => {
                chars.len() >= 2 && 
                chars[0].is_ascii_uppercase() && 
                chars[1] == '$'
            }
            r"^[A-Z]" => {
                chars.len() >= 1 && 
                chars[0].is_ascii_uppercase()
            }
            _ => false
        }
    }

    // Get the length of the longest matching keyword
    fn get_keyword_length(&self, input: &str) -> Option<usize> {
        let keywords = vec![
            "REM", "LET", "PRINT", "INPUT", "IF", "THEN", "ELSE",
            "FOR", "TO", "STEP", "NEXT", "GOTO", "GOSUB", "RETURN",
            "END", "STOP", "DATA", "READ", "RESTORE", "DIM", "ON",
            "DEF", "AND", "OR", "NOT"
        ];
        
        let functions = vec![
            "ABS", "ASC", "ATN", "COS", "EXP", "INT", "LOG", "RND", "SGN", "SIN", "SQR", "TAN",
            "CHR$", "LEFT$", "LEN", "MID$", "RIGHT$", "SPACE$", "STR$"
        ];
        
        // Try to match the longest keyword/function first
        for len in (1..=input.len()).rev() {
            let candidate = &input[..len];
            
            // Check keywords
            for keyword in &keywords {
                if candidate == *keyword {
                    return Some(len);
                }
            }
            
            // Check functions
            for function in &functions {
                if candidate == *function {
                    return Some(len);
                }
            }
        }
        
        None
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
        assert_eq!(tokens[1], Token::Identifier("X".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::Number("123".to_string()));
    }

    #[test]
    fn test_tokenize_for() {
        let mut lexer = Lexer::new("FOR I = 1 TO 10");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0], Token::For);
        assert_eq!(tokens[1], Token::Identifier("I".to_string(), IdentifierType::Variable));
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
        assert_eq!(tokens[2], Token::Identifier("ABS".to_string(), IdentifierType::BuiltInFunction));
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
        
        if let Err(BasicError::Syntax { message, basic_line_number, file_line_number }) = result {
            assert!(message.contains("Unterminated string"));
            assert_eq!(basic_line_number, None); // No basic line number for this error
            assert_eq!(file_line_number, Some(1));
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

    #[test]
    fn test_basic_space_free_syntax() {
        // Test the case mentioned: 100FORI=ATOBSTEPC
        let mut lexer = Lexer::new("100FORI=ATOBSTEPC");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens for '100FORI=ATOBSTEPC':");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }
        
        // Should parse as: LineNumber(100), For, Identifier("I"), Equal, Identifier("A"), To, Identifier("B"), Step, Identifier("C")
        assert_eq!(tokens[0], Token::LineNumber(100));
        assert_eq!(tokens[1], Token::For);
        assert_eq!(tokens[2], Token::Identifier("I".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[3], Token::Equal);
        assert_eq!(tokens[4], Token::Identifier("A".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[5], Token::To);
        assert_eq!(tokens[6], Token::Identifier("B".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[7], Token::Step);
        assert_eq!(tokens[8], Token::Identifier("C".to_string(), IdentifierType::Variable));
    }

    #[test]
    fn test_decimal_number_in_comparison() {
        // Test the specific failing case: 850 IFR1>.98THENK3=3:K9=K9+3:GOTO980
        let mut lexer = Lexer::new("850 IFR1>.98THENK3=3:K9=K9+3:GOTO980");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens for '850 IFR1>.98THENK3=3:K9=K9+3:GOTO980':");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }
        
        // Should parse as: LineNumber(850), If, Identifier("R1"), Greater, Number(".98"), Then, Identifier("K3"), Equal, Number("3"), Colon, Identifier("K9"), Equal, Identifier("K9"), Plus, Number("3"), Colon, Goto, Number("980")
        assert_eq!(tokens[0], Token::LineNumber(850));
        assert_eq!(tokens[1], Token::If);
        assert_eq!(tokens[2], Token::Identifier("R1".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[3], Token::Greater);
        assert_eq!(tokens[4], Token::Number(".98".to_string()));
        assert_eq!(tokens[5], Token::Then);
        assert_eq!(tokens[6], Token::Identifier("K3".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[7], Token::Equal);
        assert_eq!(tokens[8], Token::Number("3".to_string()));
        assert_eq!(tokens[9], Token::Colon);
        assert_eq!(tokens[10], Token::Identifier("K9".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[11], Token::Equal);
        assert_eq!(tokens[12], Token::Identifier("K9".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[13], Token::Plus);
        assert_eq!(tokens[14], Token::Number("3".to_string()));
        assert_eq!(tokens[15], Token::Colon);
        assert_eq!(tokens[16], Token::Goto);
        assert_eq!(tokens[17], Token::Number("980".to_string()));
    }

    #[test]
    fn test_tab_function() {
        // Test TAB function recognition
        let mut lexer = Lexer::new("PRINT TAB(8)");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens for 'PRINT TAB(8)':");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }
        
        assert_eq!(tokens[0], Token::Print);
        assert_eq!(tokens[1], Token::Identifier("TAB".to_string(), IdentifierType::BuiltInFunction));
        assert_eq!(tokens[2], Token::LeftParen);
        assert_eq!(tokens[3], Token::Number("8".to_string()));
        assert_eq!(tokens[4], Token::RightParen);
    }

    #[test]
    fn test_complex_print_statement() {
        // Test the specific failing line: 2840 PRINTTAB(8);:R1=I:GOSUB8790:PRINTG2$;" REPAIR COMPLETED."
        let mut lexer = Lexer::new("2840 PRINTTAB(8);:R1=I:GOSUB8790:PRINTG2$;\" REPAIR COMPLETED.\"");
        let tokens = lexer.tokenize().unwrap();
        
        println!("Tokens for complex PRINT statement:");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }
        
        // Should parse as: LineNumber(2840), Print, Identifier("TAB"), LeftParen, Number("8"), RightParen, Semicolon, Colon, Identifier("R1"), Equal, Identifier("I"), Colon, Gosub, Number("8790"), Colon, Print, Identifier("G2$"), Semicolon, String(" REPAIR COMPLETED.")
        assert_eq!(tokens[0], Token::LineNumber(2840));
        assert_eq!(tokens[1], Token::Print);
        assert_eq!(tokens[2], Token::Identifier("TAB".to_string(), IdentifierType::BuiltInFunction));
        assert_eq!(tokens[3], Token::LeftParen);
        assert_eq!(tokens[4], Token::Number("8".to_string()));
        assert_eq!(tokens[5], Token::RightParen);
        assert_eq!(tokens[6], Token::Semicolon);
        assert_eq!(tokens[7], Token::Colon);
        assert_eq!(tokens[8], Token::Identifier("R1".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[9], Token::Equal);
        assert_eq!(tokens[10], Token::Identifier("I".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[11], Token::Colon);
        assert_eq!(tokens[12], Token::Gosub);
        assert_eq!(tokens[13], Token::Number("8790".to_string()));
        assert_eq!(tokens[14], Token::Colon);
        assert_eq!(tokens[15], Token::Print);
        assert_eq!(tokens[16], Token::Identifier("G2$".to_string(), IdentifierType::Variable));
        assert_eq!(tokens[17], Token::Semicolon);
        assert_eq!(tokens[18], Token::String(" REPAIR COMPLETED.".to_string()));
    }
} 