use std::fs;
use std::process;
use clap::Parser;
use basic_rs::basic_parser::Parser as BasicParser;
use basic_rs::basic_lexer::Lexer;
use basic_rs::basic_reports::{load_coverage_from_file, generate_html_coverage_report, print_coverage_report};

#[derive(Parser)]
#[command(author, version, about = "Generate coverage reports from BasicRS coverage data")]
struct Args {
    /// Coverage data file (JSON)
    coverage_file: String,
    
    /// BASIC program file
    program_file: String,
    
    /// Output HTML file (optional, defaults to text output)
    #[arg(short = 'o', long = "html")]
    html: Option<String>,
    
    /// Show detailed line-by-line coverage in text mode
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    // Load coverage data
    let coverage = match load_coverage_from_file(&args.coverage_file) {
        Ok(coverage) => coverage,
        Err(e) => {
            eprintln!("Error loading coverage file {}: {}", args.coverage_file, e);
            process::exit(1);
        }
    };

    // Load and parse the BASIC program
    let source = match fs::read_to_string(&args.program_file) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Error reading program file {}: {}", args.program_file, e);
            process::exit(1);
        }
    };

    let mut lexer = Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Error lexing program: {:?}", e);
            process::exit(1);
        }
    };

    let mut parser = BasicParser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error parsing program: {:?}", e);
            process::exit(1);
        }
    };

    // Generate report
    match args.html {
        Some(html_file) => {
            // Generate HTML report
            if let Err(e) = generate_html_coverage_report(&coverage, &program, &html_file) {
                eprintln!("Error generating HTML report: {}", e);
                process::exit(1);
            }
        }
        None => {
            // Generate text report
            print_coverage_report(&coverage, &program, args.verbose);
        }
    }
} 