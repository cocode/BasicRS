use std::env;
use std::fs;
use std::process;
use basic_rs::basic_parser::Parser;
use basic_rs::basic_lexer::Lexer;

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
            for token in &tokens {
                println!("T: {}", token);
            }
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    // TODO: Run the program
                    println!("Program parsed successfully!");
                    println!("Program has {} lines.", program.lines.len());
                    println!("{}", program);
                    use basic_rs::basic_interpreter::Interpreter;
                    let mut interpreter = Interpreter::new(program);
                    println!("Intepreter created");
                    interpreter.run();
                    println!("Program done");
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("Parse error: {:?}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", program_path, e);
            process::exit(1);
        }
    }
    println!("All done.")
}