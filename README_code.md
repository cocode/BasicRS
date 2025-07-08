#  <img src="images/logo2.png" alt="Logo" width="50" height="25"> BasicRS

BasicRS is a Rust implementation of a BASIC Interpreter 

## To Run Unit Tests

cargo test --lib

## Basic Test Suite
The BASIC test quite is a set of BASIC programs that test the implementation of BASIC. 

### To Run Basic Test Suite
cargo test --test run_tests

### To Run One Basic Test Suite

cargo run -- test_suite/hello.bas

## ./run_all_tests.sh
A shell script which runs tests and summarizes results. Does the same as cargo test, but more readable.

## Terminology
A LINE is made up of multiple STATEMENTS, each one beginning with a KEYWORD.

### LINE
    100 PRINT X:GOTO 200
### STATEMENTS
    "PRINT X" and "GOTO 100"
### KEYWORDS
    "PRINT", and "GOTO"

