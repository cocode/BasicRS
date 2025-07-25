use std::fmt;
use crate::basic_function_registry::FUNCTION_REGISTRY;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Let,
    Print,
    Input,
    If,
    Then,
    Else,
    For,
    To,
    Step,
    Next,
    Goto,
    Gosub,
    Return,
    End,
    Stop,
    Rem,
    Data,
    Read,
    Restore,
    Dim,
    On,
    Def,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Power,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Not,
    
    // Punctuation
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Colon,
    
    // Values
    Number(String),
    String(String),
    Identifier(String, IdentifierType),
    LineNumber(usize),
    
    // Special
    Newline,
}

impl fmt::Display for IdentifierType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentifierType::Variable => write!(f, "Variable"),
            IdentifierType::Array => write!(f, "Array"),
            IdentifierType::Keyword => write!(f, "Keyword"),
            IdentifierType::BuiltInFunction => write!(f, "BuiltInFunction"),
            IdentifierType::UserDefinedFunction => write!(f, "UserDefinedFunction"),
            }
        }
    }

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Let => write!(f, "LET"),
            Token::Print => write!(f, "PRINT"),
            Token::Input => write!(f, "INPUT"),
            Token::If => write!(f, "IF"),
            Token::Then => write!(f, "THEN"),
            Token::Else => write!(f, "ELSE"),
            Token::For => write!(f, "FOR"),
            Token::To => write!(f, "TO"),
            Token::Step => write!(f, "STEP"),
            Token::Next => write!(f, "NEXT"),
            Token::Goto => write!(f, "GOTO"),
            Token::Gosub => write!(f, "GOSUB"),
            Token::Return => write!(f, "RETURN"),
            Token::End => write!(f, "END"),
            Token::Stop => write!(f, "STOP"),
            Token::Rem => write!(f, "REM"),
            Token::Data => write!(f, "DATA"),
            Token::Read => write!(f, "READ"),
            Token::Restore => write!(f, "RESTORE"),
            Token::Dim => write!(f, "DIM"),
            Token::On => write!(f, "ON"),
            Token::Def => write!(f, "DEF"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Power => write!(f, "^"),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Not => write!(f, "NOT"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Identifier(i, j ) => write!(f, "{} {}", i, j),
            Token::LineNumber(l) => write!(f, "{}", l),
            Token::Newline => write!(f, "\n"),
        }
    }
}

impl Token {
    pub fn new_number(n: &str) -> Self {
        Token::Number(n.to_string())
    }

    pub fn new_string(s: &str) -> Self {
        Token::String(s.to_string())
    }

    pub fn new_identifier(id: &str, id_type: IdentifierType) -> Self {
        Token::Identifier(id.to_string(), id_type)
    }

    pub fn new_equal() -> Self {
        Token::Equal
    }

    pub fn new_greater() -> Self {
        Token::Greater
    }

    pub fn token(&self) -> Result<&str, BasicError> {
        match self {
            Token::Number(n) => Ok(n),
            Token::String(s) => Ok(s),
            Token::Identifier(id, id_type) => match id_type {
                IdentifierType::BuiltInFunction => Err(BasicError::Runtime {
                    message: format!("Cannot pass function '{}' as an argument to a function", id),
                    basic_line_number: None,
                    file_line_number: None,
                }),
                _ => Ok(id),
            },
            _ => Err(BasicError::Runtime {
                message: format!("Unexpected token type: {:?}", self),
                basic_line_number: None,
                file_line_number: None,
            }),
        }
    }
}

#[derive(Debug)]
pub enum BasicError {
    Syntax {
        message: String,
        basic_line_number: Option<usize>,
        file_line_number: Option<usize>,
    },
    Runtime {
        message: String,
        basic_line_number: Option<usize>,
        file_line_number: Option<usize>,
    },
    Internal {
        message: String,
        basic_line_number: Option<usize>,
        file_line_number: Option<usize>,
    },
    Type {
        message: String,
        basic_line_number: Option<usize>,
        file_line_number: Option<usize>,
    },
}

impl fmt::Display for BasicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BasicError::Syntax { message, basic_line_number, file_line_number } => {
                match (basic_line_number, file_line_number) {
                    (Some(basic), _) => write!(f, "Syntax error at BASIC line {}: {}", basic, message),
                    (None, Some(file)) => write!(f, "Syntax error at file line {}: {}", file, message),
                    (None, None) => write!(f, "Syntax error: {}", message),
                }
            }
            BasicError::Runtime { message, basic_line_number, file_line_number } => {
                match (basic_line_number, file_line_number) {
                    (Some(basic), _) => write!(f, "Runtime error at BASIC line {}: {}", basic, message),
                    (None, Some(file)) => write!(f, "Runtime error at file line {}: {}", file, message),
                    (None, None) => write!(f, "Runtime error: {}", message),
                }
            }
            BasicError::Internal { message, basic_line_number, file_line_number } => {
                        match (basic_line_number, file_line_number) {
                            (Some(basic), _) => write!(f, "Internal error at BASIC line {}: {}", basic, message),
                            (None, Some(file)) => write!(f, "Internal error at file line {}: {}", file, message),
                            (None, None) => write!(f, "Internal error: {}", message),
                        }
                    }
            BasicError::Type { message, basic_line_number, file_line_number } => {
                match (basic_line_number, file_line_number) {
                    (Some(basic), _) => write!(f, "Type error at BASIC line {}: {}", basic, message),
                    (None, Some(file)) => write!(f, "Type error at file line {}: {}", file, message),
                    (None, None) => write!(f, "Type error: {}", message),
                }
            }
        }
    }
}

impl std::error::Error for BasicError {}

impl From<std::io::Error> for BasicError {
    fn from(error: std::io::Error) -> Self {
        BasicError::Internal {
            message: format!("I/O error: {}", error),
            basic_line_number: None,
            file_line_number: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunStatus {
    // TODO there should be a 'have not run yet' status, but we start with run.
    Run,
    EndNormal,  // is EndNormal a duplicate ofEndProgram
    EndErrorSyntax,
    EndErrorRuntime,
    EndErrorInternal,
    EndErrorType,
    EndOfProgram,
    EndStop,
    BreakCode,
    BreakData,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Variable,
    Function,
    Array,
}

// Constants for BASIC syntax
pub const NUMBERS: &str = "0123456789";
pub const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayDecl {
    pub name: String,
    pub dimensions: Vec<usize>,
}

impl fmt::Display for ArrayDecl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}(", self.name)?;
        for (i, dim) in self.dimensions.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", dim)?;
        }
        write!(f, ")")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrintItem {
    Expression(Expression),
    Tab(usize),
    Comma,      // Tab to next column
    Semicolon,  // No spacing
}

// Statement types
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let { var: Expression, value: Expression },
    Print { items: Vec<PrintItem> },
    Input { vars: Vec<String>, prompt: Option<String> },
    If { condition: Expression },
    Then,
    Else,
    For { var: String, start: Expression, stop: Expression, step: Option<Expression> },
    Next { var: String },
    Goto { line: usize },
    Gosub { line: usize },
    Return,
    End,
    Stop,
    Rem { comment: String },
    Data { values: Vec<SymbolValue> },
    Read { vars: Vec<Expression>},
    Restore { line: Option<usize> },
    Dim {
        arrays: Vec<ArrayDecl>,
    },
    OnGoto { expr: Expression, line_numbers: Vec<usize> },
    OnGosub { expr: Expression, line_numbers: Vec<usize> },
    Def { name: String, params: Vec<String>, expr: Expression },
}

impl Statement {

    pub fn new_let(var: Expression, value: Expression) -> Self {
        Statement::Let { var, value }
    }

    pub fn new_print(expressions: Vec<Expression>) -> Self {
        Statement::Print { items: expressions.into_iter().map(PrintItem::Expression).collect() }
    }
    pub fn new_input(vars: Vec<String>) -> Self {
        Statement::Input { vars, prompt: None }
    }

    pub fn new_if(condition: Expression) -> Self {
        Statement::If { condition }
    }

    pub fn new_then() -> Self {
        Statement::Then
    }

    pub fn new_else() -> Self {
        Statement::Else
    }

    pub fn new_for(var: String, start: Expression, stop: Expression, step: Option<Expression>) -> Self {
        Statement::For { var, start, stop, step }
    }

    pub fn new_next(var: String) -> Self {
        Statement::Next { var }
    }

    pub fn new_goto(line: usize) -> Self {
        Statement::Goto { line }
    }

    pub fn new_gosub(line: usize) -> Self {
        Statement::Gosub { line }
    }

    pub fn new_return() -> Self {
        Statement::Return
    }

    pub fn new_end() -> Self {
        Statement::End
    }

    pub fn new_stop() -> Self {
        Statement::Stop
    }

    pub fn new_rem(comment: String) -> Self {
        Statement::Rem { comment }
    }

    pub fn new_data(values: Vec<SymbolValue>) -> Self {
        Statement::Data { values }
    }

    pub fn new_read(vars: Vec<Expression>) -> Self {
        Statement::Read { vars }
    }

    pub fn new_restore(line: Option<usize>) -> Self {
        Statement::Restore { line }
    }

    
    pub fn new_dim(arrays: Vec<ArrayDecl>) -> Self {
        Statement::Dim { arrays }
    }

    pub fn new_on_goto(expr: Expression, line_numbers: Vec<usize>) -> Self {
        Statement::OnGoto { expr, line_numbers }
    }

    pub fn new_on_gosub(expr: Expression, line_numbers: Vec<usize>) -> Self {
        Statement::OnGosub { expr, line_numbers }
    }

    pub fn new_def(name: String, params: Vec<String>, expr: Expression) -> Self {
        Statement::Def { name, params, expr }
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Statement::*;

        match self {
            Let { var, value } => write!(f, "LET {} = {}", var, value),
            Print { items } => {
                write!(f, "PRINT")?;
                for item in items{
                    match item {
                        PrintItem::Expression(expr) => write!(f, " {}", expr)?,
                        PrintItem::Tab(n) => write!(f, "\t{}", " ".repeat(*n))?,
                        PrintItem::Comma => write!(f, ", ")?,
                        PrintItem::Semicolon => write!(f, "; ")?,
                    }
                }
                Ok(())
            }
            Input { vars, prompt } => {
                write!(f, "INPUT")?;
                if let Some(p) = prompt {
                    write!(f, " \"{}\"", p)?;
                }
                write!(f, " {}", vars.join(", "))
            },
            If { condition } => {
                write!(f, "IF {}", condition)
            }
            Then => write!(f, "THEN"),
            Else => write!(f, "ELSE"),
            For { var, start, stop, step } => {
                write!(f, "FOR {} = {} TO {}", var, start, stop)?;
                if let Some(step) = step {
                    write!(f, " STEP {}", step)?;
                }
                Ok(())
            }
            Next { var } => write!(f, "NEXT {}", var),
            Goto { line } => write!(f, "GOTO {}", line),
            Gosub { line } => write!(f, "GOSUB {}", line),
            Return => write!(f, "RETURN"),
            End => write!(f, "END"),
            Stop => write!(f, "STOP"),
            Rem { comment } => write!(f, "REM {}", comment),
            Data { values } => {
                write!(f, "DATA")?;
                for (i, v) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                Ok(())
            }
            Read { vars } => {
                write!(f, "READ")?;
                for (i, v) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                Ok(())
            }
            Restore { line } => {
                write!(f, "RESTORE")?;
                if let Some(n) = line {
                    write!(f, " {}", n)?;
                }
                Ok(())
            }
            Dim { arrays } => {
                write!(f, "DIM")?;
                for (i, array) in arrays.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", array)?;
                }
                Ok(())
            }
            OnGoto { expr, line_numbers } => {
                write!(f, "ON {} GOTO ", expr)?;
                for (i, n) in line_numbers.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", n)?;
                }
                Ok(())
            }
            OnGosub { expr, line_numbers } => {
                write!(f, "ON {} GOSUB ", expr)?;
                for (i, n) in line_numbers.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", n)?;
                }
                Ok(())
            }
            Def { name, params, expr } => {
                write!(f, "DEF {}(", name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") = {}", expr)
            }
        }
    }
}
// Expression types
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionType {
    Number(f64),
    String(String),
    Variable(String),

    Array {
        name: String,
        indices: Vec<Expression>,
    },

    BinaryOp {
        op: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    UnaryOp {
        op: String,
        expr: Box<Expression>,
    },

    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdentifierType {
    Variable,
    Array,
    Keyword,
    BuiltInFunction,
    UserDefinedFunction,
}

impl fmt::Display for ExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionType::Number(n) => write!(f, "{}", n),
            ExpressionType::String(s) => write!(f, "\"{}\"", s),
            ExpressionType::Variable(name) => write!(f, "{}", name),

            ExpressionType::Array { name, indices } => {
                write!(f, "{}(", name)?;
                for (i, index) in indices.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", index)?;
                }
                write!(f, ")")
            }

            ExpressionType::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", left, op, right)
            }

            ExpressionType::UnaryOp { op, expr } => {
                write!(f, "{}{}", op, expr)
            }

            ExpressionType::FunctionCall { name, args } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}
// Expression struct
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub expr_type: ExpressionType,
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.expr_type)
    }
}

impl Expression {
    pub fn new_number(n: f64) -> Self {
        Expression {
            expr_type: ExpressionType::Number(n),
        }
    }

    pub fn new_string(s: String) -> Self {
        Expression {
            expr_type: ExpressionType::String(s),
        }
    }

    pub fn new_variable(name: String) -> Self {
        Expression {
            expr_type: ExpressionType::Variable(name),
        }
    }

    pub fn new_array(name: String, indices: Vec<Expression>) -> Self {
        Expression {
            expr_type: ExpressionType::Array { name, indices },
        }
    }

    pub fn new_binary_op(op: String, left: Expression, right: Expression) -> Self {
        Expression {
            expr_type: ExpressionType::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
        }
    }
    pub fn new_unary_op(op: String, expr: Expression) -> Self {
        Expression {
            expr_type: ExpressionType::UnaryOp {
                op,
                expr: Box::new(expr),
            },
        }
    }

    pub fn new_function_call(name: String, args: Vec<Expression>) -> Self {
        Expression {
            expr_type: ExpressionType::FunctionCall { name, args },
        }
    }
}

// Program line structure
#[derive(Debug, Clone, PartialEq)]
pub struct ProgramLine {
    pub line_number: usize,
    pub source: String,
    pub statements: Vec<Statement>,
}
impl fmt::Display for ProgramLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.line_number)?;
        for (i, stmt) in self.statements.iter().enumerate() {
            write!(f, "{}", stmt)?;
            if i < self.statements.len() - 1 {
                write!(f, " : ")?; // BASIC separates multiple statements with colons
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Program {
    pub lines: Vec<ProgramLine>,
}

impl Program {
    pub fn new() -> Self {
        Program { lines: Vec::new() }
    }

    pub fn add_line(&mut self, line_number: usize, source: String, statements: Vec<Statement>) {
        match self.lines.binary_search_by_key(&line_number, |l| l.line_number) {
            Ok(pos) => self.lines[pos] = ProgramLine { line_number, source, statements },
            Err(pos) => self.lines.insert(pos, ProgramLine { line_number, source, statements }),
        }
    }

    pub fn get_line(&self, line_number: usize) -> Option<&ProgramLine> {
        self.lines.binary_search_by_key(&line_number, |l| l.line_number)
            .ok()
            .map(|i| &self.lines[i])
    }

    pub fn remove_line(&mut self, line_number: usize) {
        if let Ok(pos) = self.lines.binary_search_by_key(&line_number, |l| l.line_number) {
            self.lines.remove(pos);
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in &self.lines {
            writeln!(f, "{}", line)?;
        }
        Ok(())
    }
}

// Helper functions
pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let chars: Vec<char> = name.chars().collect();
    
    // First character must be A-Z
    if !chars[0].is_ascii_uppercase() {
        return false;
    }
    
    // Check if it's a known function name (3+ characters, all uppercase)
    if chars.len() >= 3 && chars.iter().all(|c| c.is_ascii_uppercase()) {
        // Allow known built-in functions
        if FUNCTION_REGISTRY.is_function(name) {
            return true;
        }
        
        // Allow BASIC function names (FN + letter pattern like FNA, FNB, FNC)
        if chars.len() == 3 && chars[0] == 'F' && chars[1] == 'N' && chars[2].is_ascii_uppercase() {
            return true;
        }
    }
    
    // Check if it ends with $ (string function)
    if chars.len() > 1 && chars[chars.len() - 1] == '$' && 
       chars[..chars.len()-1].iter().all(|c| c.is_ascii_uppercase()) {
        // Only allow known string functions
        let known_string_functions = ["CHR$", "LEFT$", "MID$", "RIGHT$", "SPACE$", "STR$"];
        if known_string_functions.contains(&name) {
            return true;
        }
    }
    
    // Standard BASIC variable rules: letter + optional digit + optional $
    if chars.len() > 1 {
        let last_char = chars[chars.len() - 1];
        if !last_char.is_ascii_digit() && last_char != '$' {
            return false;
        }
        
        // Allow up to 3 characters: letter + digit + $
        if chars.len() > 3 {
            return false;
        }
        
        // Check for valid patterns: A, A1, A$, A1$
        if chars.len() == 2 {
            // Two characters: must be A1 or A$
            if !chars[1].is_ascii_digit() && chars[1] != '$' {
                return false;
            }
        } else if chars.len() == 3 {
            // Three characters: must be A1$ (letter + digit + $)
            if !chars[1].is_ascii_digit() || chars[2] != '$' {
                return false;
            }
        }
    }
    
    true
}

// Symbol table entry types
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub value: String,
    pub symbol_type: SymbolType,
    pub arg: Option<String>,
}


// Operation token for expression evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub token: String,
    pub op_type: String,
    pub arg: Option<String>,
    pub value: Option<String>,
    pub symbols: Option<Vec<Symbol>>,
}

// Helper functions
pub fn assert_syntax(value: bool, message: &str) -> Result<(), BasicError> {
    if !value {
        Err(BasicError::Syntax {
            message: message.to_string(),
            basic_line_number: None,
            file_line_number: None,
        })
    } else {
        Ok(())
    }
}

pub fn assert_internal(value: bool, message: &str) -> Result<(), BasicError> {
    if !value {
        Err(BasicError::Internal {
            message: message.to_string(),
            basic_line_number: None,
            file_line_number: None,
        })
    } else {
        Ok(())
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_valid_identifiers() {
        assert!(is_valid_identifier("A"));
        assert!(is_valid_identifier("A$"));
        assert!(is_valid_identifier("A1"));
        assert!(is_valid_identifier("B2"));
        assert!(is_valid_identifier("Z9"));
        assert!(is_valid_identifier("X$"));
    }

    #[test]
    fn test_invalid_identifiers() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("1A"));
        assert!(!is_valid_identifier("A1B"));
        assert!(!is_valid_identifier("AB"));
        assert!(!is_valid_identifier("A$B"));
        assert!(!is_valid_identifier("A12"));
        assert!(!is_valid_identifier("a"));
        assert!(!is_valid_identifier("a$"));
    }

    #[test]
    fn test_program_basic_operations() {
        let line1 = ProgramLine {
            line_number: 10,
            statements: vec![Statement::new_print(vec![
                Expression::new_string("Hello".to_string())
            ])],
            source: "10 PRINT \"Hello\"".to_string(),
        };

        let line2 = ProgramLine {
            line_number: 20,
            statements: vec![Statement::new_end()],
            source: "20 END".to_string(),
        };

        let mut program = Program::new();
        program.lines.push(line1.clone());
        program.lines.push(line2.clone());

        assert_eq!(program.get_line(10).unwrap().line_number, 10);
        assert_eq!(program.get_line(20).unwrap().line_number, 20);
        assert!(program.get_line(30).is_none());
    }

    #[test]
    fn test_program_line_ordering() {
        let mut program = Program::new();
        
        program.add_line(20, "20 PRINT \"Second\"".to_string(), vec![
            Statement::new_print(vec![Expression::new_string("Second".to_string())])
        ]);
        
        program.add_line(10, "10 PRINT \"First\"".to_string(), vec![
            Statement::new_print(vec![Expression::new_string("First".to_string())])
        ]);
        
        program.add_line(30, "30 END".to_string(), vec![
            Statement::new_end()
        ]);
        
        assert_eq!(program.lines[0].line_number, 10);
        assert_eq!(program.lines[1].line_number, 20);
        assert_eq!(program.lines[2].line_number, 30);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayElementType {
    Number,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArrayData {
    Numbers(Vec<f64>),    // Flattened storage for N-dimensional numeric arrays
    Strings(Vec<String>), // Flattened storage for N-dimensional string arrays
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolValue {
    Number(f64),
    String(String),
    Array {
        element_type: ArrayElementType,
        dimensions: Vec<usize>,
        data: ArrayData,
    },
    // Legacy support - these will be removed after refactoring is complete
    Array1DNumber(Vec<f64>),
    Array2DNumber(Vec<Vec<f64>>),
    Array1DString(Vec<String>),
    Array2DString(Vec<Vec<String>>),
    FunctionDef {
        param: Vec<String>,
        expr: Expression,
    },
}
impl PartialOrd for SymbolValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SymbolValue::Number(a), SymbolValue::Number(b)) => a.partial_cmp(b),
            (SymbolValue::String(a), SymbolValue::String(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl SymbolValue {
    pub fn len(&self) -> usize {
        match self {
            SymbolValue::Array { dimensions, .. } => dimensions.iter().product(),
            SymbolValue::Array1DNumber(arr) => arr.len(),
            SymbolValue::Array2DNumber(arr) => arr.len(),
            SymbolValue::Array1DString(arr) => arr.len(),
            SymbolValue::Array2DString(arr) => arr.len(),
            SymbolValue::String(s) => s.len(),
            _ => 0,
        }
    }
}

impl fmt::Display for SymbolValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SymbolValue::Number(n) => {
                // In classic BASIC, positive numbers get both leading and trailing spaces
                // Negative numbers get only trailing space (minus sign serves as leading char)
                if *n >= 0.0 {
                    write!(f, " {} ", n)
                } else {
                    write!(f, "{} ", n)
                }
            },
            SymbolValue::String(s) => write!(f, "{}", s),

            SymbolValue::Array { element_type, dimensions, data } => {
                match (element_type, data) {
                    (ArrayElementType::Number, ArrayData::Numbers(vec)) => {
                        if dimensions.len() == 1 {
                            write!(f, "{:?}", vec)
                        } else {
                            write!(f, "Array{:?}", dimensions)
                        }
                    }
                    (ArrayElementType::String, ArrayData::Strings(vec)) => {
                        if dimensions.len() == 1 {
                            write!(f, "{:?}", vec)
                        } else {
                            write!(f, "Array{:?}", dimensions)
                        }
                    }
                    _ => write!(f, "InvalidArray")
                }
            }

            SymbolValue::Array1DNumber(a) => write!(f, "{:?}", a),
            SymbolValue::Array2DNumber(a) => {
                write!(f, "[")?;
                for (i, row) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", row)?;
                }
                write!(f, "]")
            }

            SymbolValue::Array1DString(a) => write!(f, "{:?}", a),
            SymbolValue::Array2DString(a) => {
                write!(f, "[")?;
                for (i, row) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", row)?;
                }
                write!(f, "]")
            }
            SymbolValue::FunctionDef { param, expr } => {
                write!(f, "FN({}) = {}", param.join(", "), expr)
            }
        }
    }
}