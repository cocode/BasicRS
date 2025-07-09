use std::env;
use std::fs;
use std::process;
use basic_rs::basic_parser::Parser;
use basic_rs::basic_lexer::Lexer;
use basic_rs::basic_types::RunStatus;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <program.bas>", args[0]);
        process::exit(1);
    }

    let program_path = &args[1];
    match fs::read_to_string(program_path) {
        Ok(source) => {
            let mut lexer = Lexer::new(&source);

            let tokens = lexer.tokenize().expect("Lexing failed");
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    println!("Program parsed successfully!");
                    println!("Program has {} lines.", program.lines.len());
                    use basic_rs::basic_interpreter::Interpreter;
                    let mut interpreter = Interpreter::new(program);
                    match interpreter.run() {
                        Ok(()) => {
                            println!("Program completed successfully");
                            process::exit(0);
                        }
                        Err(e) => {
                            use basic_rs::basic_types::BasicError;
                            match &e {
                                BasicError::Syntax { message, .. } => {
                                    eprintln!("Syntax error: {}", message);
                                    process::exit(2);
                                }
                                BasicError::Runtime { message, .. } => {
                                    eprintln!("Runtime error: {}", message);
                                    process::exit(3);
                                }
                                BasicError::Internal { message, basic_line_number, file_line_number } => {
                                    eprintln!("Internal error: {}", message);
                                    process::exit(4);
                                }
                                BasicError::Type { message, .. } => {
                                    eprintln!("Type error: {}", message);
                                    process::exit(5);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Parse error: {:?}", e);
                    process::exit(2);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", program_path, e);
            process::exit(1);
        }
    }
}