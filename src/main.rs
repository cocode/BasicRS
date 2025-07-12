use std::fs;
use std::process;
use basic_rs::basic_parser::Parser;
use basic_rs::basic_lexer::Lexer;
use basic_rs::basic_types::RunStatus;
use basic_rs::basic_reports::{CoverageData, save_coverage_to_file, load_coverage_from_file, merge_coverage};
use clap::Parser as ClapParser;

#[derive(ClapParser)]
#[command(author, version, about = "BasicRS - A BASIC interpreter written in Rust")]
struct Args {
    /// BASIC program file to execute
    program: String,
    
    /// Enable coverage tracking and save to file
    #[arg(long)]
    coverage_file: Option<String>,
    
    /// Reset coverage data (delete existing file before starting)
    #[arg(long)]
    reset_coverage: bool,
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

    // Handle reset coverage flag
    if args.reset_coverage {
        if let Some(ref coverage_file) = args.coverage_file {
            if fs::metadata(coverage_file).is_ok() {
                if let Err(e) = fs::remove_file(coverage_file) {
                    eprintln!("Warning: Failed to remove existing coverage file {}: {}", coverage_file, e);
                }
            }
        } else {
            eprintln!("Error: --reset-coverage requires --coverage-file");
            process::exit(1);
        }
    }

    let program_path = &args.program;
    match fs::read_to_string(program_path) {
        Ok(source) => {
            let mut lexer = Lexer::new(&source);

            let tokens = lexer.tokenize().expect("Lexing failed");
            let mut parser = Parser::new(tokens);
            match parser.parse() {
                Ok(program) => {
                    // println!("Program parsed successfully!");
                    // println!("Program has {} lines.", program.lines.len());
                    use basic_rs::basic_interpreter::Interpreter;
                    let mut interpreter = Interpreter::new(program);
                    if let Err(e) = interpreter.enable_trace() {
                        eprintln!("Failed to enable trace: {}", e);
                        process::exit(97);
                    }
                    
                    // Enable coverage if requested
                    if args.coverage_file.is_some() {
                        interpreter.enable_coverage();
                    }
                    
                    match interpreter.run() {
                        Ok(()) => {
                            // Save coverage data if requested
                            if let Some(ref coverage_file) = args.coverage_file {
                                if let Some(coverage) = interpreter.get_coverage() {
                                    if let Err(e) = save_coverage_data(coverage, coverage_file) {
                                        eprintln!("Warning: Failed to save coverage data: {}", e);
                                    }
                                }
                            }
                            
                            let status = interpreter.get_run_status();
                            match status {
                                RunStatus::EndNormal => {
                                    println!("Program completed successfully");
                                    process::exit(0);
                                }
                                RunStatus::EndStop => {
                                    println!("Program stopped");
                                    process::exit(1);
                                }
                                RunStatus::EndOfProgram => {
                                    println!("Program reached end");
                                    process::exit(0);
                                }
                                RunStatus::BreakCode => {
                                    println!("Breakpoint hit");
                                    process::exit(2);
                                }
                                RunStatus::BreakData => {
                                    println!("Data breakpoint hit");
                                    process::exit(3);
                                }
                                _ => {
                                    println!("Program ended with status: {:?}", status);
                                    process::exit(4);
                                }
                            }
                        }
                        Err(e) => {
                            use basic_rs::basic_types::BasicError;
                            match &e {
                                BasicError::Syntax { message, basic_line_number, file_line_number } => {
                                    print_basic_error("Syntax", message, basic_line_number, file_line_number);
                                    process::exit(5);
                                }
                                BasicError::Runtime { message, basic_line_number, file_line_number } => {
                                    print_basic_error("Runtime", message, basic_line_number, file_line_number);
                                    process::exit(6);
                                }
                                BasicError::Internal { message, basic_line_number, file_line_number } => {
                                    print_basic_error("Internal", message, basic_line_number, file_line_number);
                                    process::exit(7);
                                }
                                BasicError::Type { message, basic_line_number, file_line_number } => {
                                    print_basic_error("Type", message, basic_line_number, file_line_number);
                                    process::exit(8);
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

fn save_coverage_data(new_coverage: &CoverageData, coverage_file: &str) -> std::io::Result<()> {
    // Load existing coverage if file exists
    let merged_coverage = if fs::metadata(coverage_file).is_ok() {
        match load_coverage_from_file(coverage_file) {
            Ok(existing_coverage) => merge_coverage(existing_coverage, new_coverage.clone()),
            Err(e) => {
                eprintln!("Warning: Failed to load existing coverage file, creating new one: {}", e);
                new_coverage.clone()
            }
        }
    } else {
        new_coverage.clone()
    };
    
    // Save merged coverage
    save_coverage_to_file(&merged_coverage, coverage_file)
}