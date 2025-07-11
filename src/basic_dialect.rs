/// Configuration constants for controlling BASIC dialect features.
///
/// This module defines switches that control various language features that vary
/// between different dialects of BASIC. These settings allow the interpreter to
/// emulate different BASIC variants by changing behavior for arrays, operators,
/// input handling, etc. We expect to add more, as we support more programs.

// =============================================================================
// OPERATOR CONFIGURATION
// =============================================================================

/// Operator used for exponentiation in mathematical expressions
/// Standard options: '^' (most common) or '**' (requires lexer changes)
pub const EXPONENTIATION_OPERATOR: &str = "^";

// =============================================================================
// ARRAY CONFIGURATION
// =============================================================================

/// Base index for array subscripts
/// 0 = Zero-based arrays (like C, Rust): A(0) is first element
/// 1 = One-based arrays (traditional BASIC): A(1) is first element
pub const ARRAY_OFFSET: usize = 1;

// =============================================================================
// INPUT/OUTPUT CONFIGURATION
// =============================================================================

/// Controls whether user input is automatically converted to uppercase
/// true = Convert input to uppercase (traditional BASIC behavior)
/// false = Preserve original case of user input
pub const UPPERCASE_INPUT: bool = true;

// Maximum line number
pub const MAX_LINE_NUMBER: usize = 99999;

// Maximum string length
pub const MAX_STRING_LENGTH: usize = 255;

// Maximum array dimensions
pub const MAX_ARRAY_DIMS: usize = 3;

// Maximum number of variables
pub const MAX_VARIABLES: usize = 1000;

// Maximum recursion depth
pub const MAX_RECURSION_DEPTH: usize = 100;

// Maximum number of nested FOR loops
pub const MAX_FOR_DEPTH: usize = 20;

// Maximum number of nested GOSUB calls
pub const MAX_GOSUB_DEPTH: usize = 20;

// Keywords
pub const KEYWORDS: &[&str] = &[
    "LET",
    "PRINT",
    "INPUT",
    "IF",
    "THEN",
    "ELSE",
    "FOR",
    "TO",
    "STEP",
    "NEXT",
    "GOTO",
    "GOSUB",
    "RETURN",
    "REM",
    "END",
    "STOP",
    "DATA",
    "READ",
    "RESTORE",
    "DIM",
    "AND",
    "OR",
    "NOT",
];

// Operators in order of precedence (highest to lowest)
pub const OPERATORS: &[&str] = &[
    "^",            // Exponentiation
    "*", "/",       // Multiplication and division
    "+", "-",       // Addition and subtraction
    "=", "<>",      // Equality and inequality
    "<", ">",       // Less than and greater than
    "<=", ">=",     // Less than or equal and greater than or equal
    "AND",          // Logical AND
    "OR",           // Logical OR
    "NOT",          // Logical NOT
];

// Built-in functions
pub const FUNCTIONS: &[&str] = &[
    "ABS",
    "ATN",
    "COS",
    "EXP",
    "INT",
    "LOG",
    "RND",
    "SGN",
    "SIN",
    "SQR",
    "TAN",
    "CHR$",
    "LEFT$",
    "LEN",
    "MID$",
    "RIGHT$",
];

// Define string functions (those that return strings)
pub const STRING_FUNCTIONS: &[&str] = &[
    "CHR$",
    "LEFT$",
    "MID$",
    "RIGHT$",
];

// Define numeric functions (those that return numbers)
pub const NUMERIC_FUNCTIONS: &[&str] = &[
    "ABS",
    "ATN",
    "COS",
    "EXP",
    "INT",
    "LOG",
    "RND",
    "SGN",
    "SIN",
    "SQR",
    "TAN",
    "LEN",
];

// Helper function to check if a function returns a string
pub fn is_string_function(name: &str) -> bool {
    STRING_FUNCTIONS.contains(&name)
}

// Helper function to check if a function returns a number
pub fn is_numeric_function(name: &str) -> bool {
    NUMERIC_FUNCTIONS.contains(&name)
}

// Helper function to get operator precedence
pub fn get_operator_precedence(op: &str) -> i32 {
    OPERATORS.iter()
        .position(|&o| o == op)
        .map(|i| 8 - i as i32)
        .unwrap_or(0)
} 