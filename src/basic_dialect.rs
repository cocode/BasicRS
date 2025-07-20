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


/// Controls whether user input is automatically converted to uppercase
/// true = Convert input to uppercase (traditional BASIC behavior)
/// false = Preserve original case of user input
pub const UPPERCASE_INPUT: bool = true;


// =============================================================================
// Not yet implemented features:
// =============================================================================

/// Maximum line number
pub const MAX_LINE_NUMBER: usize = 99999;

/// Maximum string length
pub const MAX_STRING_LENGTH: usize = 255;

// Maximum array dimensions
pub const MAX_ARRAY_DIMS: usize = 2;

// Maximum number of variables
pub const MAX_VARIABLES: usize = 1000;

// Maximum recursion depth
pub const MAX_RECURSION_DEPTH: usize = 100;

// Maximum number of nested FOR loops
pub const MAX_FOR_DEPTH: usize = 200;

// Maximum number of nested GOSUB calls
pub const MAX_GOSUB_DEPTH: usize = 200;


 