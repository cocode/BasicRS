use crate::basic_types::{ArrayDecl};

use crate::basic_types::{
    Token, BasicError, Statement, Expression,
    Program
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    current_line: usize,
    _data_values: ()
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            current_line: 1,
            _data_values: ()
        }
    }

    pub fn parse(&mut self) -> Result<Program, BasicError> {
        let mut program = Program::new();
        
        while !self.is_at_end() {
            let line_number = self.parse_line_number()?;
            self.current_line = line_number;
            let source = self.get_line_source();
            let statements = self.parse_statements()?;
            for stmt in &statements {
                if let Statement::Data { values } = stmt {
                    self._data_values.extend(values.iter().cloned());
                }
            }
            program.add_line(line_number, source, statements);
            
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
            _ => Err(BasicError::Syntax {
                message: "Expected line number at start of line".to_string(),
                line_number: Some(self.current_line),
            }),
        }
    }

    fn parse_statements(&mut self) -> Result<Vec<Statement>, BasicError> {
        let mut statements = Vec::new();

        while !self.is_at_end() && !self.check(&Token::Newline) {
            let stmt = self.parse_statement()?;
            statements.push(stmt.clone());

            // After REM, consume rest of line
            if let Statement::Rem { .. } = stmt {
                while !self.is_at_end() && !self.check(&Token::Newline) {
                    self.advance();
                }
                break;
            }

            if self.check(&Token::Colon) {
                self.advance(); // Skip colon
            } else {
                break;
            }
        }

        if self.check(&Token::Newline) {
            self.advance(); // Skip newline
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, BasicError> {
        match self.peek() {
            Some(Token::Let) => {
                self.advance();
                let var = self.parse_identifier()?;
                self.consume(&Token::Equal, "Expected '=' after variable name")?;
                let value = self.parse_expression()?;
                Ok(Statement::Let { var, value })
            }
            Some(Token::Print) => {
                self.advance();
                let mut expressions = Vec::new();
                let mut current_expression = String::new();
                let mut in_string = false;
                
                while !self.is_at_end() && !self.check(&Token::Colon) && !self.check(&Token::Newline) {
                    if let Some(token) = self.peek() {
                        match token {
                            Token::String(s) => {
                                if !in_string {
                                    current_expression.push_str(s);
                                    in_string = true;
                                }
                                self.advance();
                            }
                            Token::Identifier(s) => {
                                if in_string {
                                    current_expression.push(' ');
                                }
                                current_expression.push_str(s);
                                in_string = false;
                                self.advance();
                            }
                            Token::Number(n) => {
                                if in_string {
                                    current_expression.push(' ');
                                }
                                current_expression.push_str(&n.to_string());
                                in_string = false;
                                self.advance();
                            }
                            Token::Plus => {
                                self.advance();
                            }
                            Token::Comma => {
                                if !current_expression.is_empty() {
                                    expressions.push(Expression::new_string(current_expression.clone()));
                                    current_expression.clear();
                                }
                                in_string = false;
                                self.advance();
                            }
                            _ => break,
                        }
                    }
                }
                if !current_expression.is_empty() {
                    expressions.push(Expression::new_string(current_expression));
                }
                Ok(Statement::Print { expressions })
            }
            Some(Token::Input) => {
                self.advance();
                let var = self.parse_identifier()?;
                Ok(Statement::Input { var })
            }
            Some(Token::If) => {
                self.advance();
                let condition = self.parse_expression()?;
                self.consume(&Token::Then, "Expected THEN after condition")?;
                let then_stmt = Box::new(self.parse_statement()?);
                let else_stmt = if self.match_any(&[Token::Else]) {
                    Some(Box::new(self.parse_statement()?))
                } else {
                    None
                };
                Ok(Statement::If { condition, then_stmt, else_stmt })
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
                    let var = self.parse_identifier()?;
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
                    Some(self.parse_line_number()?)
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
            Some(token) => Err(BasicError::Syntax {
                message: format!("Unexpected token: {}", token),
                line_number: Some(self.current_line),
            }),
            None => Err(BasicError::Syntax {
                message: "Unexpected end of input".to_string(),
                line_number: Some(self.current_line),
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
            let right = self.parse_comparison()?;
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
            Some(Token::Identifier(name)) => {
                self.advance();
                if self.check(&Token::LeftParen) {
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

                    // Determine if this is a function call or array access
                    if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        Ok(Expression::new_function_call(name.clone(), args))
                    } else {
                        Ok(Expression::new_array(name.clone(), args))
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
                line_number: None,
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
                line_number: None,
            })
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn parse_identifier(&mut self) -> Result<String, BasicError> {
        let token = self.peek().cloned();
        match token {
            Some(Token::Identifier(id)) => {
                self.advance();
                Ok(id.clone())
            }
            _ => Err(BasicError::Syntax {
                message: "Expected identifier".to_string(),
                line_number: None,
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
                    line_number: Some(self.current_line),
                })
            }
            _ => Err(BasicError::Syntax {
                message: "Expected number".to_string(),
                line_number: None,
            }),
        }
    }

    fn get_line_source(&self) -> String {
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
            comment.push_str(&self.advance().to_string());
            if !self.is_at_end() && !self.check(&Token::Newline) {
                comment.push(' ');
            }
        }
        comment
    }
}

#[cfg(test)]
mod tests {
    use crate::basic_types::ExpressionType;
    use super::*;
    use crate::basic_types::{Token, Statement, Expression};

    #[test]
    fn test_parse_line_number() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Let,
            Token::Identifier("X".to_string()),
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
            assert_eq!(var, "X");
        } else {
            panic!("Expected LET statement");
        }
    }
    #[test]
    fn test_parse_let_statement_with_identifier() {
        let tokens = vec![
            Token::LineNumber(20),
            Token::Let,
            Token::Identifier("X".to_string()),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Colon,
            Token::Print,
            Token::Identifier("Y".to_string()),
            Token::Newline,
        ];

        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 20);
        assert_eq!(program.lines[0].statements.len(), 2);

        // Check LET statement
        if let Statement::Let { var, value } = &program.lines[0].statements[0] {
            assert_eq!("X", var);
            // assert_eq!("1", self.evaluate_expression(value), "1");
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
    fn test_parse_rem() {
        let tokens = vec![
            Token::LineNumber(30),
            Token::Rem,
            Token::String("This is a comment".to_string()),
            Token::Colon,
            Token::Print, // This should be ignored after REM
            Token::Identifier("X".to_string()),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.lines.len(), 1);
        assert_eq!(program.lines[0].line_number, 30);
        assert_eq!(program.lines[0].statements.len(), 1);
        
        if let Statement::Rem { comment } = &program.lines[0].statements[0] {
            assert_eq!(comment, "This is a comment");
        } else {
            panic!("Expected REM statement");
        }
    }

    #[test]
    fn test_parse_multiple_lines() {
        let tokens = vec![
            Token::LineNumber(10),
            Token::Let,
            Token::Identifier("X".to_string()),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Newline,
            Token::LineNumber(20),
            Token::Print,
            Token::Identifier("X".to_string()),
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
            Token::Identifier("X".to_string()),
            Token::Equal,
            Token::Number("1".to_string()),
            Token::Newline,
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        assert!(result.is_err());
        if let Err(BasicError::Syntax { message, line_number }) = result {
            assert!(message.contains("line number"));
            assert_eq!(line_number, Some(1));
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
        if let Err(BasicError::Syntax { message, line_number }) = result {
            assert!(message.contains("Unexpected token"));
            assert_eq!(line_number, Some(10));
        } else {
            panic!("Expected syntax error");
        }
    }
}
