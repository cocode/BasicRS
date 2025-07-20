# Structure of this project.

## Overview

BasicRS is a BASIC interpreter written in Rust that supports a subset of the BASIC programming language. The project includes both an interpreter and a compiler that generates LLVM IR code.

## Project Architecture

### Core Components

The project is organized into several key modules:

#### 1. **basic_types.rs** - Core Data Structures
Defines fundamental types used throughout the interpreter
- `Token` enum: Represents lexical tokens (keywords, operators, literals, identifiers)
- `Statement` enum: Represents parsed BASIC statements (LET, PRINT, IF, etc.)
- `Expression` struct: Represents expressions with various types (numbers, strings, variables, arrays, operations)
- `Program` struct: Contains the parsed program with line numbers and statements
- `SymbolValue` enum: Represents runtime values (numbers, strings, arrays)
- `BasicError` enum: Error types (Syntax, Runtime, Internal, Type)
- `RunStatus` enum: Program execution status

#### 2. **basic_lexer.rs** - Lexical Analysis
`Lexer` struct: Tokenizes BASIC source code
- Processes line numbers, keywords, operators, identifiers, and literals
- Handles BASIC's space-free syntax (e.g., `LETX=5`)
- Supports both explicit and implicit LET statements

#### 3. **basic_parser.rs** - Syntax Analysis
`Parser` struct: Converts tokens into an Abstract Syntax Tree (AST)
- Implements recursive descent parsing
- Handles operator precedence and associativity
- Parses all BASIC statements and expressions

#### 4. **basic_interpreter.rs** - Execution Engine
`Interpreter` struct: Executes parsed BASIC programs
- Implements control flow (GOTO, GOSUB, FOR/NEXT, IF/THEN)
- Manages symbol tables and variable scope
- Handles arrays, functions, and built-in functions
- Supports debugging features (breakpoints, tracing, coverage)

#### 5. **basic_symbols.rs** - Symbol Management
`SymbolTable` struct: Manages variables, functions, and arrays
- Handles variable scoping and lifetime
- Supports both numeric and string variables

#### 6. **basic_operators.rs** - Operator Implementation
- Implements arithmetic, comparison, and logical operators
- Handles type coercion and error checking

#### 7. **basic_function_registry.rs** - Built-in Functions
Registry of built-in functions (SIN, COS, RND, etc.)
- Function registration and lookup system

#### 8. **basic_keyword_registry.rs** - Keyword Management
Registry of BASIC keywords
- Keyword recognition and classification

#### 9. **basic_dialect.rs** - Language Dialect
Defines BASIC dialect-specific features
- Handles case sensitivity and input formatting

#### 10. **basic_reports.rs** - Reporting and Coverage
- Code coverage tracking and reporting
- HTML coverage report generation
- Coverage data serialization

### Compiler Components

#### 11. **llvm_codegen.rs** - LLVM Code Generation
- `LLVMCodeGenerator` struct: Generates LLVM IR from BASIC programs
- Converts BASIC statements to LLVM instructions
- Handles variable allocation and memory management
- Supports debugging and tracing in generated code

#### 12. **llvm_ir_builder.rs** - LLVM IR Construction
- `LLVMIRBuilder` struct: Low-level LLVM IR generation
- Manages LLVM module, function, and basic block creation
- Handles LLVM instruction generation

### Executables

The project provides several executables:

#### 1. **basic_rs** (main.rs)
- Main BASIC interpreter executable
- Command-line interface for running BASIC programs
- Supports coverage tracking with `--coverage-file` and `--reset-coverage` options
- Returns appropriate exit codes based on program completion status

#### 2. **basic_shell** (src/bin/basic_shell.rs)
- Interactive BASIC development environment
- Commands: load, run, step, continue, break, symbols, coverage
- Debugging features: breakpoints, variable inspection, stack inspection
- Program formatting and line renumbering

#### 3. **basic_compiler** (src/bin/basic-compiler.rs)
- Compiles BASIC programs to LLVM IR
- Generates optimized native code
- Supports debug and trace modes

#### 4. **basic_coverage** (src/bin/basic_coverage.rs)
- Coverage analysis and reporting tool
- Generates HTML coverage reports
- Analyzes coverage data from multiple runs

## Testing Framework

### Test Organization

#### 1. **Unit Tests**
- Located in each module with `#[cfg(test)]` sections
- Test individual components (lexer, parser, interpreter)
- Use `cargo test --lib` to run

#### 2. **BASIC Test Suite**
- Located in `test_suite/` directory
- Contains `.bas` files with BASIC programs
- Automatically generated test functions via `build.rs`
- Use `cargo test --test run_tests` to run

#### 3. **Integration Tests**
- Located in `tests/run_tests.rs`
- Tests complete program execution
- Validates exit codes and program behavior

### Running Tests

#### All Tests
```bash
./run_all_tests.sh
```

#### Unit Tests Only
```bash
cargo test --lib
```

#### BASIC Test Suite Only
```bash
cargo test --test run_tests
```

#### Specific Test
```bash
cargo test test_name
```

### Test File Format

BASIC test files can include expected exit codes:
```basic
10 @EXPECT_EXIT_CODE=0
20 PRINT "Hello, World!"
30 END
```

## Build System

### Dependencies
- **anyhow, thiserror**: Error handling
- **regex, lazy_static**: Text processing
- **clap**: Command-line interface
- **pretty_assertions**: Enhanced test assertions
- **tracing**: Debugging and logging
- **rand**: Random number generation
- **serde**: Serialization
- **chrono**: Date/time handling

### Build Process
1. `build.rs` automatically discovers `.bas` files in `test_suite/`
2. Generates test functions in `target/out/generated_tests.rs`
3. Compiles with LLVM support for code generation

## Project Structure

```
BasicRS/
├── src/
│   ├── lib.rs                 # Library entry point
│   ├── main.rs                # Main executable
│   ├── basic_types.rs         # Core data structures
│   ├── basic_lexer.rs         # Lexical analysis
│   ├── basic_parser.rs        # Syntax analysis
│   ├── basic_interpreter.rs   # Execution engine
│   ├── basic_symbols.rs       # Symbol management
│   ├── basic_operators.rs     # Operator implementation
│   ├── basic_function_registry.rs  # Built-in functions
│   ├── basic_keyword_registry.rs   # Keyword management
│   ├── basic_dialect.rs       # Language dialect
│   ├── basic_reports.rs       # Coverage and reporting
│   ├── llvm_codegen.rs        # LLVM code generation
│   ├── llvm_ir_builder.rs     # LLVM IR construction
│   └── bin/                   # Executables
│       ├── basic_shell.rs     # Interactive shell
│       ├── basic-compiler.rs  # LLVM compiler
│       └── basic_coverage.rs  # Coverage tool
├── test_suite/                # BASIC test programs
├── tests/                     # Integration tests
├── build.rs                   # Build script
├── Cargo.toml                 # Project configuration
└── run_all_tests.sh          # Test runner script
```

## Usage Examples

### Running a BASIC Program
```bash
cargo run -- program.bas
```

### Running with Coverage
```bash
cargo run -- program.bas --coverage-file coverage.json
```

### Interactive Shell
```bash
cargo run --bin basic_shell -- program.bas
```

### Compiling to LLVM IR
```bash
cargo run --bin basic-compiler -- program.bas
```

### Coverage Analysis
```bash
cargo run --bin basic_coverage -- coverage.json
```

## Error Handling

The project uses a comprehensive error handling system:
- **Syntax errors**: Invalid program structure
- **Runtime errors**: Execution-time problems
- **Type errors**: Type mismatches
- **Internal errors**: Interpreter implementation issues

All errors include line number information for debugging.

## Future Development

The project is actively developed with several planned improvements:
- Enhanced LLVM optimization
- Additional BASIC dialect support
- Improved debugging tools
- Performance optimizations
- Extended test coverage