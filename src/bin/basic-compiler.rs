use std::fs;
use std::path::Path;
use std::process;
use basic_rs::basic_parser::Parser;
use basic_rs::basic_lexer::Lexer;
use basic_rs::llvm_codegen::LLVMCodeGenerator;
use basic_rs::basic_types::BasicError;
use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(author, version, about = "BasicRS LLVM-IR Code Generator - Converts BASIC programs to LLVM-IR")]
struct Args {
    /// BASIC program file to compile
    input: String,
    
    /// Output LLVM-IR file (defaults to input with .ll extension)
    #[arg(short, long)]
    output: Option<String>,
    
    /// Enable debug output during code generation
    #[arg(long)]
    debug: bool,
    
    /// Enable trace statements in generated code
    #[arg(long)]
    trace: bool,
}

fn print_basic_error(kind: &str, message: &str, basic_line_number: &Option<usize>, file_line_number: &Option<usize>) {
    let mut parts = vec![format!("{} error:", kind)];
    if let Some(basic_line) = basic_line_number {
        parts.push(format!("BASIC line {}", basic_line));
    }
    if let Some(file_line) = file_line_number {
        parts.push(format!("file line {}", file_line));
    }
    let label = parts.join(", ");
    eprintln!("{} {}", label, message);
}

fn main() {
    let args = Args::parse();

    // Determine output file name
    let output_path = match args.output {
        Some(path) => path,
        None => {
            let input_path = Path::new(&args.input);
            let stem = input_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("output");
            format!("{}.ll", stem)
        }
    };

    // Read and parse the BASIC program
    let program = match fs::read_to_string(&args.input) {
        Ok(source) => {
            let mut lexer = Lexer::new(&source);
            
            let tokens = match lexer.tokenize() {
                Ok(tokens) => tokens,
                Err(e) => {
                    eprintln!("Lexing failed: {:?}", e);
                    process::exit(10);
                }
            };
            
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    if args.debug {
                        println!("Program parsed successfully!");
                        println!("Program has {} lines.", program.lines.len());
                    }
                    program
                }
                Err(e) => {
                    match &e {
                        BasicError::Syntax { message, basic_line_number, file_line_number } => {
                            print_basic_error("Parse", message, basic_line_number, file_line_number);
                            process::exit(11);
                        }
                        BasicError::Runtime { message, basic_line_number, file_line_number } => {
                            print_basic_error("Parse", message, basic_line_number, file_line_number);
                            process::exit(12);
                        }
                        BasicError::Internal { message, basic_line_number, file_line_number } => {
                            print_basic_error("Internal Parse", message, basic_line_number, file_line_number);
                            process::exit(13);
                        }
                        BasicError::Type { message, basic_line_number, file_line_number } => {
                            print_basic_error("Type Parse", message, basic_line_number, file_line_number);
                            process::exit(14);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", args.input, e);
            process::exit(15);
        }
    };

    // Generate LLVM-IR
    let mut codegen = LLVMCodeGenerator::new(program, args.debug, args.trace);
    
    let llvm_ir = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        codegen.generate_ir()
    })) {
        Ok(ir) => ir,
        Err(_) => {
            eprintln!("LLVM-IR generation failed with internal error");
            process::exit(16);
        }
    };

    // Write LLVM-IR to output file
    match fs::write(&output_path, llvm_ir) {
        Ok(_) => {
            if args.debug {
                println!("Successfully generated LLVM-IR: {}", output_path);
            }
            process::exit(0);
        }
        Err(e) => {
            eprintln!("Error writing LLVM-IR file {}: {}", output_path, e);
            process::exit(17);
        }
    }
} 