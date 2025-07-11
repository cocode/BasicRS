use crate::basic_types::{ArrayDecl, ExpressionType, IdentifierType, SymbolValue};

use crate::basic_types::{
    Token, BasicError, Statement, Expression,
    Program
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    current_basic_line: Option<usize>,  // If there is a syntax error, there may not be a line number
    current_file_line: usize,           // There should always be a 'line number the file' (or source string)
    _data_values: Vec<SymbolValue>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            current_basic_line: None,
            current_file_line: 1,
            _data_values: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Program, BasicError> {
        let mut program = Program::new();
        
        while !self.is_at_end() {
            let line_number = self.parse_line_number()?;
            self.current_basic_line = Some(line_number);
            // println!("line {}", line_number);
            let source = self.get_rebuilt_line_source();
            let statements = self.parse_statements()?;

            program.add_line(line_number, source, statements);
            self.current_file_line += 1;
            
            // Skip any extra newlines between statements
            while self.check(&Token::Newline) {
                self.advance();
            }
        }
        
        Ok(program)
    }

    fn parse_line_number(&mut self) -> Result<usize, BasicError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::LineNumber(n)) => {
                self.advance();
                Ok(n)
            }
            _ => {
                let current_token = self.peek().map(|t| format!("{:?}", t)).unwrap_or_else(|| "end of input".to_string());
                Err(BasicError::Syntax {
                    message: format!("Expected line number at start of line, got {}", current_token),
                    basic_line_number: self.current_basic_line,
                    file_line_number: Some(self.current_file_line),
                })
            }
        }
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, BasicError> {
        let mut statements = Vec::new();

        while !self.is_at_end() && !self.check(&Token::Newline) {
            let stmt = self.parse_statement()?;
            statements.push(stmt.clone());

            // After REM, consume rest of line TODO Not right, we already consume the line when parsing the REM
            if let Statement::Rem { .. } = stmt {
                while !self.is_at_end() && !self.check(&Token::Newline) {
                    self.advance();
                }
                break;
            }

            if self.check(&Token::Colon) {
                self.advance(); // Skip colon
            }
        }

        // Don't advance past newline here - let the main parse() function handle it

        Ok(statements)
    }

    fn parse_data_constant(&mut self) -> Result<SymbolValue, BasicError> {
        let token = self.peek().cloned();

        match token {
            Some(Token::Number(n)) => {
                self.advance();
                let value = n.parse::<f64>().map_err(|_| BasicError::Syntax {
                    message: format!("Invalid numeric constant in DATA: {}", n),
                    basic_line_number: self.current_basic_line,
                    file_line_number: None,
                })?;
                Ok(SymbolValue::Number(value))
            }
            Some(Token::Minus) => {
                // Handle negative numbers
                self.advance();
                match self.peek().cloned() {
                    Some(Token::Number(n)) => {
                        self.advance();
                        let value = n.parse::<f64>().map_err(|_| BasicError::Syntax {
                            message: format!("Invalid numeric constant in DATA: -{}", n),
                            basic_line_number: self.current_basic_line,
                            file_line_number: None,
                        })?;
                        Ok(SymbolValue::Number(-value))
                    }
                    _ => Err(BasicError::Syntax {
                        message: "Expected number after minus sign in DATA".to_string(),
                        basic_line_number: self.current_basic_line,
                        file_line_number: None,
                    })
                }
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(SymbolValue::String(s))
            }
            Some(other) => Err(BasicError::Syntax {
                message: format!("Invalid token in DATA statement: {}", other),
                basic_line_number: self.current_basic_line,
                file_line_number: None,
            }),
            None => Err(BasicError::Syntax {
                message: "Unexpected end of input in DATA statement".to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: None,
            }),
        }
    }

    fn parse_implicit_or_explicit_let(&mut self, skip_token: bool) -> Result<Statement, BasicError> {
        if skip_token {
            self.advance(); // skip `LET`
        }

        let var = self.parse_lvalue()?;
        self.consume(&Token::Equal, "Expected '=' after variable name")?;
        let value = self.parse_expression()?;

        Ok(Statement::Let { var, value })
    }

    fn parse_statement(&mut self) -> Result<Statement, BasicError> {
        match self.peek() {
            Some(Token::Let) => self.parse_implicit_or_explicit_let(true),
            Some(Token::Identifier(_, _)) => self.parse_implicit_or_explicit_let(false),
            Some(Token::Print) => {
                self.advance();
                let mut expressions = Vec::new();
                
                // Parse comma/semicolon-separated expressions
                if !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                    loop {
                        // Parse expression (or empty string if just spacing)
                        if self.check(&Token::Comma) || self.check(&Token::Semicolon) {
                            // Empty expression (just spacing)
                            expressions.push(Expression::new_string("".to_string()));
                            self.advance();
                        } else {
                            // Parse actual expression
                            expressions.push(self.parse_expression()?);
                            
                            if self.check(&Token::Comma) || self.check(&Token::Semicolon) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        
                        // If we're at the end or hit a colon/newline, stop
                        if self.is_at_end() || self.check(&Token::Colon) || self.check(&Token::Newline) {
                            break;
                        }
                    }
                }
                
                // Check if there are unexpected tokens after the PRINT statement
                if !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                    let current_token = self.peek().map(|t| format!("{:?}", t)).unwrap_or_else(|| "end of input".to_string());
                    return Err(BasicError::Syntax {
                        message: format!("Unexpected token after PRINT expression: {}", current_token),
                        basic_line_number: self.current_basic_line,
                        file_line_number: Some(self.current_file_line),
                    });
                }
                
                Ok(Statement::Print { expressions })
            }
            Some(Token::Input) => {
                self.advance();

                // Check if there's a prompt string
                let prompt = if let Some(Token::String(s)) = self.peek().cloned() {
                    self.advance();
                    Some(s)
                } else {
                    None
                };

                // If there was a prompt, expect and skip a semicolon or comma
                if prompt.is_some() {
                    if self.check(&Token::Semicolon) || self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        return Err(BasicError::Syntax {
                            message: "Expected ';' or ',' after INPUT prompt".to_string(),
                            basic_line_number: self.current_basic_line,
                            file_line_number: None,
                        });
                    }
                }

                // Parse multiple variables separated by commas
                let mut vars = Vec::new();
                loop {
                    let var = self.parse_identifier()?;
                    vars.push(var);
                    
                    if self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                // For now, we'll use the first variable as the main variable
                // TODO: Update Statement::Input to support multiple variables
                let var = vars[0].clone();

                Ok(Statement::Input { var, prompt })
            }
            Some(Token::If) => {
                self.advance();
                let condition = self.parse_expression()?;
                // self.consume(&Token::Then, "Expected THEN after condition")?;
                Ok(Statement::If { condition })
            }
            Some(Token::Then) => {
                self.advance();
                // Check if next token is a number, for IF x THEN 100
                if let Some(Token::Number(_)) = self.peek() {
                    // Insert GOTO token before the number
                    self.tokens.insert(self.current, Token::Goto);
                }
                Ok(Statement::Then)
            }
            Some(Token::Else) => {
                self.advance();
                // Check if next token is a number
                if let Some(Token::Number(_)) = self.peek() {
                    // Insert GOTO token before the number
                    self.tokens.insert(self.current, Token::Goto);
                }
                Ok(Statement::Else)
            }
            Some(Token::For) => {
                self.advance();
                let var = self.parse_identifier()?;
                self.consume(&Token::Equal, "Expected '=' after FOR variable")?;
                let start = self.parse_expression()?;
                self.consume(&Token::To, "Expected 'TO' after FOR start value")?;
                let stop = self.parse_expression()?;
                let step = if self.match_any(&[Token::Step]) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                Ok(Statement::For { var, start, stop, step })
            }
            Some(Token::Next) => {
                self.advance();
                let var = self.parse_identifier()?;
                Ok(Statement::Next { var })
            }
            Some(Token::Goto) => {
                self.advance();
                let line = self.parse_number()? as usize;
                Ok(Statement::Goto { line })
            }
            Some(Token::Gosub) => {
                self.advance();
                let line = self.parse_number()? as usize;
                Ok(Statement::Gosub { line })
            }
            Some(Token::Return) => {
                self.advance();
                Ok(Statement::Return)
            }
            Some(Token::End) => {
                self.advance();
                Ok(Statement::End)
            }
            Some(Token::Stop) => {
                self.advance();
                Ok(Statement::Stop)
            }
            Some(Token::Rem) => {
                self.advance();
                let comment = self.get_rest_of_line();
                Ok(Statement::Rem { comment })
            }
            Some(Token::Data) => {
                self.advance();
                let mut values = Vec::new();
                while !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                    let value = self.parse_data_constant()?;
                    values.push(value);

                    if self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(Statement::Data { values })
            }
            Some(Token::Read) => {
                self.advance();
                let mut vars = Vec::new();

                while !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                    let var = self.parse_primary()?;
                    vars.push(var);

                    if self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                Ok(Statement::Read { vars })
            }
            Some(Token::Restore) => {
                self.advance();
                let line = if !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                    Some(self.parse_number()? as usize)
                } else {
                    None
                };
                Ok(Statement::Restore { line })
            }
            Some(Token::Dim) => {
                self.advance();

                let mut arrays = Vec::new();

                loop {
                    let var = self.parse_identifier()?;
                    self.consume(&Token::LeftParen, "Expected '(' after array name")?;

                    let mut dimensions = Vec::new();
                    while !self.check(&Token::RightParen) {
                        let dim = self.parse_number()? as usize;
                        dimensions.push(dim);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    self.consume(&Token::RightParen, "Expected ')' after dimensions")?;

                    arrays.push(ArrayDecl { name: var, dimensions });

                    if self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }

                Ok(Statement::Dim { arrays })
            }
            Some(Token::On) => {
                self.advance();
                let expr = self.parse_expression()?;
                
                if self.check(&Token::Goto) {
                    self.advance();
                    let mut line_numbers = Vec::new();
                    while !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                        let line = self.parse_number()? as usize;
                        line_numbers.push(line);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    Ok(Statement::OnGoto { expr, line_numbers })
                } else if self.check(&Token::Gosub) {
                    self.advance();
                    let mut line_numbers = Vec::new();
                    while !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                        let line = self.parse_number()? as usize;
                        line_numbers.push(line);

                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    Ok(Statement::OnGosub { expr, line_numbers })
                } else {
                    Err(BasicError::Syntax {
                        message: "Expected GOTO or GOSUB after ON expression".to_string(),
                        basic_line_number: self.current_basic_line,
                        file_line_number: None,
                    })
                }
            }
            Some(Token::Def) => {
                self.advance();
                let name = self.parse_identifier()?;
                self.consume(&Token::LeftParen, "Expected '(' after function name")?;
                
                let mut params = Vec::new();
                while !self.check(&Token::RightParen) {
                    let param = self.parse_identifier()?;
                    params.push(param);
                    
                    if self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                
                self.consume(&Token::RightParen, "Expected ')' after parameters")?;
                self.consume(&Token::Equal, "Expected '=' after parameters")?;
                
                let expr = self.parse_expression()?;
                
                Ok(Statement::Def { name, params, expr })
            }
            Some(token) => Err(BasicError::Syntax {
                message: format!("Unexpected token: {:?}", token),
                basic_line_number: self.current_basic_line,
                file_line_number: Some(self.current_file_line),
            }),
            None => Err(BasicError::Syntax {
                message: "Unexpected end of input".to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: Some(self.current_file_line),
            }),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, BasicError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, BasicError> {
        let mut expr = self.parse_and()?;
        
        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            expr = Expression::new_binary_op("OR".to_string(), expr, right);
        }
        
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expression, BasicError> {
        let mut expr = self.parse_equality()?;
        
        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expression::new_binary_op("AND".to_string(), expr, right);
        }
        
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, BasicError> {
        let mut expr = self.parse_comparison()?;
        
        while self.match_any(&[Token::Equal, Token::NotEqual]) {
            let op = match self.previous() {
                Token::Equal => "=",
                Token::NotEqual => "<>",
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?; // TODO why start with comparison? Not or?
            expr = Expression::new_binary_op(op.to_string(), expr, right);
        }
        
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, BasicError> {
        let mut expr = self.parse_term()?;
        
        while self.match_any(&[
            Token::Less, Token::LessEqual,
            Token::Greater, Token::GreaterEqual,
        ]) {
            let op = match self.previous() {
                Token::Less => "<",
                Token::LessEqual => "<=",
                Token::Greater => ">",
                Token::GreaterEqual => ">=",
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expression::new_binary_op(op.to_string(), expr, right);
        }
        
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, BasicError> {
        let mut expr = self.parse_factor()?;
        
        while self.match_any(&[Token::Plus, Token::Minus]) {
            let op = match self.previous() {
                Token::Plus => "+",
                Token::Minus => "-",
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::new_binary_op(op.to_string(), expr, right);
        }
        
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expression, BasicError> {
        let mut expr = self.parse_unary()?;
        
        while self.match_any(&[Token::Star, Token::Slash, Token::Power]) {
            let op = match self.previous() {
                Token::Star => "*",
                Token::Slash => "/",
                Token::Power => "^",
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::new_binary_op(op.to_string(), expr, right);
        }
        
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, BasicError> {
        if self.match_any(&[Token::Minus, Token::Not]) {
            let op = match self.previous() {
                Token::Minus => "-",
                Token::Not => "NOT",
                _ => unreachable!(),
            };
            let expr = self.parse_unary()?;
            Ok(Expression::new_unary_op(op.to_string(), expr))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, BasicError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expression::new_number(n.parse().unwrap()))
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expression::new_string(s.clone()))
            }
            Some(Token::Identifier(name, id_type)) => {
                self.advance();
                let mut array_ref = false;
                if self.check(&Token::LeftParen) {
                    array_ref = true;
                    // Function call or array access
                    self.advance();
                    let mut args = Vec::new();

                    if !self.check(&Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.check(&Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }

                    self.consume(&Token::RightParen, "Expected ')' after arguments")?;

                    match id_type {
                        IdentifierType::UserDefinedFunction | IdentifierType::BuiltInFunction => {
                            Ok(Expression::new_function_call(name.clone(), args))
                        }
                        IdentifierType::Array => Ok(Expression::new_array(name.clone(), args)),
                        IdentifierType::Variable => {
                            // Not yet consistent on array refs. Should the token be array or variable?
                            if array_ref {
                                Ok(Expression::new_array(name.clone(), args))
                            } else {
                                Ok(Expression::new_variable(name.clone()))
                            }
                        },
                        other => Err(BasicError::Syntax {
                            message: format!(
                                "Unexpected identifier type '{:?}' in function/array expression",
                                other
                            ),
                            basic_line_number: None,  // TODO catch unlikely error
                            file_line_number: None,
                        }),
                    }
                } else {
                    Ok(Expression::new_variable(name.clone()))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(BasicError::Syntax {
                message: "Expected expression".to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: None,
            }),
        }
    }
    // Helper methods
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token: &Token) -> bool {
        self.peek().map_or(false, |t| t == token)
    }

    fn match_any(&mut self, tokens: &[Token]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token: &Token, message: &str) -> Result<&Token, BasicError> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(BasicError::Syntax {
                message: message.to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: None,
            })
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn parse_identifier(&mut self) -> Result<String, BasicError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::Identifier(id, id_type)) => {
                self.advance();
                Ok(id.clone())
            }
            _ => Err(BasicError::Syntax {
                message: "Expected identifier".to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: None,
            }),
        }
    }

    fn parse_number(&mut self) -> Result<f64, BasicError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::Number(n)) => {
                self.advance();
                n.parse().map_err(|_| BasicError::Syntax {
                    message: format!("Invalid number: {}", n),
                    basic_line_number: self.current_basic_line,
                    file_line_number: None,
                })
            }
            _ => Err(BasicError::Syntax {
                message: "Expected number".to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: None,
            }),
        }
    }
    /// This method reconstitutes the source line from the tokens in the statement.
    /// It is NOT the actual incoming source line. That is lost in the lexer.
    fn get_rebuilt_line_source(&self) -> String {
        let mut source = String::new();
        let mut i = self.current;
        while i < self.tokens.len() {
            match &self.tokens[i] {
                Token::Newline => break,
                token => {
                    if !source.is_empty() {
                        source.push(' ');
                    }
                    source.push_str(&token.to_string());
                }
            }
            i += 1;
        }
        source
    }

    fn get_rest_of_line(&mut self) -> String {
        let mut comment = String::new();
        while !self.is_at_end() && !self.check(&Token::Newline) {
            let token = self.advance();
            match token {
                Token::String(s) => comment.push_str(&s), // no extra quotes
                other => comment.push_str(&other.to_string()),
            }
            if !self.is_at_end() && !self.check(&Token::Newline) {
                comment.push(' ');
            }
        }
        comment
    }

    // fn parse_statement_block(&mut self) -> Result<Vec<Statement>, BasicError> {
    //     let mut statements = Vec::new();
    //
    //     while !self.is_at_end() && !self.check(&Token::Else) && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
    //         statements.push(self.parse_statement()?);
    //
    //         if self.check(&Token::Colon) {
    //             self.advance(); // Skip colon separator
    //         } else {
    //             break;
    //         }
    //     }
    //
    //     Ok(statements)
    // }

    /// Parse an expression that is the left hand side of a LET statement.
    /// LET D(5) = 99
    fn parse_lvalue(&mut self) -> Result<Expression, BasicError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expression::new_number(n.parse().unwrap()))
            }
            Some(Token::String(s)) => {
                self.advance();
                Ok(Expression::new_string(s.clone()))
            }
            Some(Token::Identifier(name, id_type)) => {
                self.advance();
                if self.check(&Token::LeftParen) {
                    self.advance();
                    let mut args = Vec::new();

                    if !self.check(&Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.check(&Token::Comma) {
                                break;
                            }
                            self.advance();
                        }
                    }

                    self.consume(&Token::RightParen, "Expected ')' after arguments")?;
                    Ok(Expression::new_array(name.clone(), args))
                } else {
                    Ok(Expression::new_variable(name.clone()))
                }
            }
            Some(Token::LeftParen) => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(&Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            _ => Err(BasicError::Syntax {
                message: "Expected expression".to_string(),
                basic_line_number: self.current_basic_line,
                file_line_number: Some(self.current_file_line),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::basic_types::{ExpressionType, IdentifierType, Token, Statement, Expression};
    use super::*;

    #[test]
    fn test_parse_line_number() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Let,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Number("123".to_string()),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 10);
        assert_eq!(program.lines[0].statements.len(), 1);

        let stmt = &program.lines[0].statements[0];

        if let Statement::Let { var, .. } = stmt {
            if let Expression { expr_type: ExpressionType::Variable(name), .. } = var {
                assert_eq!(name, "X");
            } else {
                panic!("Expected variable expression");
            }
        } else {
            panic!("Expected LET statement");
        }
    }
    #[test]
    fn test_parse_let_statement_with_identifier() {
        let tokens = vec![
            Token::LineNumber(20),
            Token::Let,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Colon,
            Token::Print,
            Token::Identifier("Y".to_string(), IdentifierType::Variable),
            Token::Newline,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 20);
        assert_eq!(program.lines[0].statements.len(), 2);

        // Check LET statement
        if let Statement::Let { var, value: _ } = &program.lines[0].statements[0] {
            if let Expression { expr_type: ExpressionType::Variable(name), .. } = var {
                assert_eq!(name, "X");
            } else {
                panic!("Expected variable expression");
            }
        } else {
            panic!("Expected LET statement");
        }

        // Check PRINT statement
        if let Statement::Print { expressions } = &program.lines[0].statements[1] {
            assert_eq!(expressions.len(), 1);
            if let Expression { expr_type: ExpressionType::Variable(name), .. } = &expressions[0] {
                assert_eq!(name, "Y");
            } else {
                panic!("Expected variable expression");
            }
        } else {
            panic!("Expected PRINT statement");
        }
    }

    #[test]
    fn test_parse_multiple_lines() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Let,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Newline,
            Token::LineNumber(20),
            Token::Print,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.lines.len(), 2);
        assert_eq!(program.lines[0].line_number, 10);
        assert_eq!(program.lines[1].line_number, 20);
    }

    #[test]
    fn test_parse_error_missing_line_number() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        assert!(result.is_err());
        if let Err(BasicError::Syntax { message, basic_line_number, file_line_number }) = result {
            assert!(message.contains("line number"));
            assert_eq!(basic_line_number, None);
            assert_eq!(file_line_number, Some(1));
        } else {
            panic!("Expected syntax error");
        }
    }

    #[test]
    fn test_parse_error_invalid_statement() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Equal, // Invalid start of statement
            Token::Number("1".to_string()),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        assert!(result.is_err());
        if let Err(BasicError::Syntax { message, basic_line_number, file_line_number }) = result {
            assert!(message.contains("Unexpected token"));
            assert_eq!(basic_line_number, Some(10));
            assert_eq!(file_line_number, Some(1));
        } else {
            panic!("Expected syntax error");
        }
    }

    #[test]
    fn test_parse_input_with_prompt() {
        let tokens = vec![
            Token::LineNumber(2060),
            Token::Input,
            Token::String("COMMAND".to_string()),
            Token::Semicolon,
            Token::Identifier("A$".to_string(), IdentifierType::Variable),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 2060);
        assert_eq!(program.lines[0].statements.len(), 1);
        
        if let Statement::Input { var, prompt } = &program.lines[0].statements[0] {
            assert_eq!(var, "A$");
            assert_eq!(prompt, &Some("COMMAND".to_string()));
        } else {
            panic!("Expected INPUT statement");
        }
    }

    #[test]
    fn test_parse_complex_print_with_tab() {
        // Test parsing the complex PRINT statement with TAB function
        let tokens = vec![
            Token::LineNumber(2840),
            Token::Print,
            Token::Identifier("TAB".to_string(), IdentifierType::BuiltInFunction),
            Token::LeftParen,
            Token::Number("8".to_string()),
            Token::RightParen,
            Token::Semicolon,
            Token::Colon,
            Token::Identifier("R1".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Identifier("I".to_string(), IdentifierType::Variable),
            Token::Colon,
            Token::Gosub,
            Token::Number("8790".to_string()),
            Token::Colon,
            Token::Print,
            Token::Identifier("G2$".to_string(), IdentifierType::Variable),
            Token::Semicolon,
            Token::String(" REPAIR COMPLETED.".to_string()),
            Token::Newline,
        ];
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        println!("Parsed program:");
        for line in &program.lines {
            println!("  Line {}: {:?}", line.line_number, line.statements);
        }
        
        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 2840);
        assert_eq!(program.lines[0].statements.len(), 4); // PRINT, LET, GOSUB, PRINT
    }

    #[test]
    fn test_parse_complex_expression() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Let,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Plus,
            Token::Number("2".to_string()),
            Token::Star,
            Token::Number("3".to_string()),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 10);
        assert_eq!(program.lines[0].statements.len(), 1);

        let stmt = &program.lines[0].statements[0];
        if let Statement::Let { var, value } = stmt {
            if let Expression { expr_type: ExpressionType::Variable(name), .. } = var {
                assert_eq!(name, "X");
            } else {
                panic!("Expected variable expression");
            }
            
            // The expression should be parsed as 1 + (2 * 3) due to operator precedence
            if let Expression { expr_type: ExpressionType::BinaryOp { op, left, right }, .. } = value {
                assert_eq!(op, "+");
                // Left side should be 1
                if let Expression { expr_type: ExpressionType::Number(n), .. } = &**left {
                    assert_eq!(*n, 1.0);
                } else {
                    panic!("Expected number 1");
                }
                // Right side should be 2 * 3
                if let Expression { expr_type: ExpressionType::BinaryOp { op, left, right }, .. } = &**right {
                    assert_eq!(op, "*");
                    if let Expression { expr_type: ExpressionType::Number(n), .. } = &**left {
                        assert_eq!(*n, 2.0);
                    } else {
                        panic!("Expected number 2");
                    }
                    if let Expression { expr_type: ExpressionType::Number(n), .. } = &**right {
                        assert_eq!(*n, 3.0);
                    } else {
                        panic!("Expected number 3");
                    }
                } else {
                    panic!("Expected multiplication expression");
                }
            } else {
                panic!("Expected binary operation");
            }
        } else {
            panic!("Expected LET statement");
        }
    }

    #[test]
    fn test_parse_function_call() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Let,
            Token::Identifier("X".to_string(), IdentifierType::Variable),
            Token::Equal,
            Token::Identifier("ABS".to_string(), IdentifierType::BuiltInFunction),
            Token::LeftParen,
            Token::Number("5".to_string()),
            Token::RightParen,
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.lines.len(), 1);
        let stmt = &program.lines[0].statements[0];
        if let Statement::Let { value, .. } = stmt {
            if let Expression { expr_type: ExpressionType::FunctionCall { name, args }, .. } = value {
                assert_eq!(name, "ABS");
                assert_eq!(args.len(), 1);
                if let Expression { expr_type: ExpressionType::Number(n), .. } = &args[0] {
                    assert_eq!(*n, 5.0);
                } else {
                    panic!("Expected number argument");
                }
            } else {
                panic!("Expected function call");
            }
        } else {
            panic!("Expected LET statement");
        }
    }
}

#[test]
fn test_parse_let_statement_with_array() {
    let tokens = vec![
        Token::Identifier("D".to_string(), IdentifierType::Variable),
        Token::LeftParen,
        Token::Number("5".to_string()),
        Token::RightParen,
    ];


    let mut parser = Parser::new(tokens);
    let expression = parser.parse_expression().unwrap();

    println!("Parsed expression: {}", expression);
 }
#[test]
fn test_parse_let_statement_with_array_plus_one() {
    let tokens = vec![
        // Token::Identifier("D".to_string(), IdentifierType::Variable),
        // Token::LeftParen,
        // Token::Number("5".to_string()),
        // Token::RightParen,
        // Token::Equal,
        Token::Identifier("D".to_string(), IdentifierType::Variable),
        Token::LeftParen,
        Token::Number("5".to_string()),
        Token::RightParen,
        Token::Plus,
        Token::Number("1".to_string()),
    ];


    let mut parser = Parser::new(tokens);
    let expression = parser.parse_expression().unwrap();

    println!("Parsed expression: {}", expression);
}