use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;
use std::path::Path;
use std::time::Instant;

use basic_rs::basic_lexer::Lexer;
use basic_rs::basic_parser::Parser;
use basic_rs::basic_interpreter::Interpreter;
use basic_rs::basic_types::{BasicError, RunStatus, SymbolType, Program};
use basic_rs::basic_reports::{print_coverage_report, generate_html_coverage_report};

/// Basic shell for interactive BASIC program development and debugging
pub struct BasicShell {
    program_file: Option<String>,
    interpreter: Option<Interpreter>,
    load_status: bool,
    breakpoints: Vec<(usize, usize)>, // (line_number, offset)
    data_breakpoints: Vec<String>,
    coverage_enabled: bool,
}

impl BasicShell {
    pub fn new(program_file: Option<String>) -> Self {
        let mut shell = BasicShell {
            program_file: program_file.clone(),
            interpreter: None,
            load_status: false,
            breakpoints: Vec::new(),
            data_breakpoints: Vec::new(),
            coverage_enabled: false,
        };
        
        if let Some(ref file) = program_file {
            shell.load_from_file(file, false);
        }
        
        shell
    }
    
    /// Transfer breakpoints from shell to interpreter
    fn transfer_breakpoints_to_interpreter(&self, interpreter: &mut Interpreter) {
        // Transfer breakpoints to the interpreter
        for (line, offset) in &self.breakpoints {
            interpreter.add_breakpoint(*line, *offset);
        }
        
        // Transfer data breakpoints to the interpreter
        for var in &self.data_breakpoints {
            interpreter.add_data_breakpoint(var.clone());
        }
    }
    
    /// Load a program from a string (used by tests)
    pub fn load_from_string(&mut self, source: &str) -> Result<(), BasicError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|e| BasicError::Syntax {
            message: e.to_string(),
            basic_line_number: None,
            file_line_number: None,
        })?;
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| BasicError::Syntax {
            message: format!("{:?}", e),
            basic_line_number: None,
            file_line_number: None,
        })?;
        
        let mut interpreter = Interpreter::new(program);
        self.transfer_breakpoints_to_interpreter(&mut interpreter);
        self.interpreter = Some(interpreter);
        self.load_status = true;
        Ok(())
    }
    
    /// Load a program from file
    pub fn load_from_file(&mut self, filename: &str, _coverage: bool) -> bool {
        self.load_status = false;
        
        let mut file_path = filename.to_string();
        if !Path::new(&file_path).exists() {
            let with_bas = format!("{}.bas", filename);
            if Path::new(&with_bas).exists() {
                file_path = with_bas;
            } else {
                println!("File not found: {}", filename);
                return false;
            }
        }
        
        println!("Loading {}", file_path);
        
        match fs::read_to_string(&file_path) {
            Ok(source) => {
                match self.load_from_string(&source) {
                    Ok(()) => {
                        self.program_file = Some(file_path);
                        self.load_status = true;
                        true
                    }
                    Err(e) => {
                        match e {
                            BasicError::Syntax { message, basic_line_number, .. } => {
                                if let Some(line_num) = basic_line_number {
                                    println!("{} in line {}", message, line_num);
                                } else {
                                    println!("{}", message);
                                }
                            }
                            _ => println!("Error loading program: {:?}", e),
                        }
                        false
                    }
                }
            }
            Err(e) => {
                println!("Error reading file {}: {}", file_path, e);
                false
            }
        }
    }
    
    /// Print usage for a command
    fn usage(&self, cmd: &str) {
        if let Some(help_text) = self.get_help_text(cmd) {
            println!("{}", help_text);
        } else {
            println!("Unknown command: {}", cmd);
        }
    }
    
    /// Get help text for a command
    fn get_help_text(&self, cmd: &str) -> Option<&'static str> {
        match cmd {
            "?" => Some("Usage: ? expression\nEvaluates and prints an expression.\nNote: You can't print single array variables. Use 'sym'\nYou may have wanted the 'help' command."),
            "benchmark" => Some("Usage: benchmark\nRuns the program from the beginning, and shows timing."),
            "break" => Some("Usage: break LINE or break SYMBOL or break list break clear\nSets a breakpoint on a line, or on writes to a variable\nNote that if you have an array and a symbol with the same name, it will break on writes to either one."),
            "clear" => Some("Usage: clear\nClears the current program and all state (breakpoints, watchpoints, coverage, etc.)\nSee also STOP command."),
            "continue" => Some("Usage: continue\nContinues, after a breakpoint."),
            "coverage" => Some("Usage: coverage [lines|html]\nPrint code coverage report.\ncoverage lines - Show uncovered lines details\ncoverage html  - Generate beautiful HTML report\nNote: Coverage must be enabled with 'run coverage' first"),
            "quit" | "exit" => Some("Usage: quit. Synonym for 'exit'"),
            "format" => Some("Usage: format\nFormats the program. Does not save it."),
            "forstack" => Some("Usage: fors\nPrints the FOR stack."),
            "gosubs" => Some("Usage: gosubs\nPrints the GOSUB stack."),
            "help" => Some("Usage: help <command>"),
            "list" => Some("Usage: list <start line number> <count>"),
            "load" => Some("Usage: load <program>\nRunning load clears coverage data."),
            "next" => Some("Usage: next.\nExecutes the next line of the program."),
            "renumber" => Some("Usage: renum <start <increment>>\nRenumbers the program."),
            "run" => Some("Usage: run <coverage>\nRuns the program from the beginning.\nAdding the string 'coverage' will cause code coverage data to be recorded from this run"),
            "save" => Some("Usage: save FILE\nSaves the current program to a new file."),
            "statements" => Some("Usage: stmt <line>\nPrints the tokenized version of the program.\nThis is used for debugging TrekBasic."),
            "stop" => Some("Usage: stop.\nIf you are running a program, this sets you back to the start.\nUnlike clear, which clears the program, breakpoints, etc. This only resets execution."),
            "symbols" => Some("Usage: sym <symbol> <type>\nPrints the symbol table, or one entry.\nType is 'variable', 'array' or 'function'. Defaults to 'variable'.\nThis is used for debugging TrekBasic."),
            _ => None,
        }
    }
    
    /// Load command
    fn cmd_load(&mut self, args: Option<&str>) {
        if let Some(filename) = args {
            let filename = filename.trim();
            let filename = if filename.starts_with('"') && filename.ends_with('"') {
                &filename[1..filename.len()-1]
            } else {
                filename
            };
            
            if !self.load_from_file(filename, false) {
                self.program_file = None;
            }
        } else {
            println!("Load requires a filename");
        }
    }
    
    /// Coverage command
    fn cmd_coverage(&self, args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            if let Some(coverage) = interpreter.get_coverage() {
                let program = interpreter.get_program();
                
                match args {
                    Some("html") => {
                        if let Err(e) = generate_html_coverage_report(coverage, program, "coverage_report.html") {
                            println!("Error generating HTML report: {}", e);
                        }
                    }
                    Some("lines") => {
                        print_coverage_report(coverage, program, true);
                    }
                    None => {
                        print_coverage_report(coverage, program, false);
                    }
                    _ => {
                        self.usage("coverage");
                    }
                }
            } else {
                println!("Coverage was not enabled for the last/current run.");
                println!("Use 'run coverage' to enable coverage tracking.");
            }
        } else {
            println!("No program loaded.");
        }
    }
    
    /// Print current line
    fn print_current(&self) {
        if let Some(ref interpreter) = self.interpreter {
            let current_line = interpreter.get_current_line();
            let current_location = interpreter.get_current_location();
            println!("{}: {}", current_line.line_number, current_line.source);
            if current_location.offset > 0 {
                println!("  (Statement {} of {})", current_location.offset + 1, current_line.statements.len());
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// List command
    fn cmd_list(&self, args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            let program = interpreter.get_program();
            let current_location = interpreter.get_current_location();
            
            let mut count = 10;
            let mut start_index = current_location.index;
            
            // Parse arguments: list [start_line] [count]
            if let Some(args) = args {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if !parts.is_empty() {
                    if let Ok(line_num) = parts[0].parse::<usize>() {
                        // Find the index for this line number
                        if let Some(found_index) = program.lines.iter().position(|line| line.line_number == line_num) {
                            start_index = found_index;
                        } else {
                            println!("Invalid line number {}", line_num);
                            self.usage("list");
                            return;
                        }
                    } else {
                        println!("Invalid line number {}", parts[0]);
                        self.usage("list");
                        return;
                    }
                }
                
                if parts.len() > 1 {
                    if let Ok(c) = parts[1].parse::<usize>() {
                        count = c;
                    } else {
                        println!("Invalid count {}", parts[1]);
                        self.usage("list");
                        return;
                    }
                }
            }
            
            // List the lines
            let end_index = std::cmp::min(start_index + count, program.lines.len());
            for i in start_index..end_index {
                let line = &program.lines[i];
                let marker = if i == current_location.index { "*" } else { " " };
                println!("{}{:5} {}", marker, line.line_number, line.source);
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// For stack command
    fn cmd_for_stack(&self, _args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            let for_stack = interpreter.get_for_stack();
            println!("For/next stack:");
            if for_stack.is_empty() {
                println!("\t<empty>");
            } else {
                for for_record in for_stack {
                    println!("\tFOR {} = <start> TO {} STEP {}", 
                             for_record.var, 
                             for_record.stop, 
                             for_record.step);
                }
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Gosub stack command
    fn cmd_gosub_stack(&self, _args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            let gosub_stack = interpreter.get_gosub_stack();
            println!("GOSUB stack:");
            if gosub_stack.is_empty() {
                println!("\t<empty>");
            } else {
                let program = interpreter.get_program();
                for location in gosub_stack {
                    if location.index < program.lines.len() {
                        let line = &program.lines[location.index];
                        println!("\tLine: {}: Clause: {}", line.line_number, location.offset);
                    }
                }
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Quit command
    fn cmd_quit(&self, _args: Option<&str>) {
        process::exit(0);
    }
    
    /// Clear command
    fn cmd_clear(&mut self, _args: Option<&str>) {
        self.interpreter = None;
        self.breakpoints.clear();
        self.data_breakpoints.clear();
        self.coverage_enabled = false;
        self.load_status = false;
        self.program_file = None;
        println!("Program and all state cleared");
    }
    
    /// Save command
    fn cmd_save(&self, args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            if let Some(filename) = args {
                let filename = filename.trim();
                let filename = if !filename.ends_with(".bas") {
                    format!("{}.bas", filename)
                } else {
                    filename.to_string()
                };
                
                if Path::new(&filename).exists() {
                    println!("No overwriting of files supported now. Still debugging. Save it to new name.");
                    return;
                }
                
                // Save the program
                let program = interpreter.get_program();
                match fs::write(&filename, program.to_string()) {
                    Ok(()) => println!("Program saved as {}", filename),
                    Err(e) => println!("Error saving file {}: {}", filename, e),
                }
            } else {
                println!("Save needs a file name.");
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Symbols command
    fn cmd_symbols(&self, args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            if let Some(args) = args {
                // Display specific symbol
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.is_empty() {
                    self.usage("symbols");
                    return;
                }
                
                let symbol_name = parts[0];
                let symbol_type = if parts.len() > 1 {
                    match parts[1].to_lowercase().as_str() {
                        "array" => SymbolType::Array,
                        "function" => SymbolType::Function,
                        "variable" => SymbolType::Variable,
                        _ => {
                            println!("Invalid symbol type '{}'. Use 'variable', 'array', or 'function'.", parts[1]);
                            return;
                        }
                    }
                } else {
                    SymbolType::Variable
                };
                
                // Try to get the symbol value
                match symbol_type {
                    SymbolType::Variable => {
                        if let Some(value) = interpreter.get_symbol_value(symbol_name) {
                            println!("{}: {:?}", symbol_name, value);
                        } else {
                            println!("The symbol '{}' is not defined as a variable.", symbol_name);
                        }
                    }
                    SymbolType::Array => {
                        let array_key = format!("{}[]", symbol_name);
                        if let Some(value) = interpreter.get_symbol_value(&array_key) {
                            println!("{}: {:?}", symbol_name, value);
                        } else {
                            println!("The symbol '{}' is not defined as an array.", symbol_name);
                        }
                    }
                    SymbolType::Function => {
                        // Functions might be stored in internal symbols
                        if let Some(value) = interpreter.get_symbol_value(symbol_name) {
                            println!("{}: {:?}", symbol_name, value);
                        } else {
                            println!("The symbol '{}' is not defined as a function.", symbol_name);
                        }
                    }
                }
                
                println!("Types are 'variable', 'array' and 'function'. Default is 'variable'");
            } else {
                // Display all symbols
                let symbols = interpreter.get_all_symbols();
                
                if symbols.is_empty() {
                    println!("No symbols defined.");
                } else {
                    println!("Symbol table:");
                    for (name, value) in symbols {
                        println!("  '{}': {:?}", name, value);
                    }
                }
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Print command (? command)
    fn cmd_print(&self, args: Option<&str>) {
        if let Some(expr_str) = args {
            if let Some(ref _interpreter) = self.interpreter {
                // For now, just attempt to evaluate simple numeric expressions
                // This is a simplified implementation - a full implementation would
                // need to parse and evaluate BASIC expressions properly
                println!("Expression evaluation: {} (not fully implemented)", expr_str);
                println!("Use the 'sym' command to inspect variables instead.");
            } else {
                // Try to evaluate simple constants even without a program
                if let Ok(value) = expr_str.trim().parse::<f64>() {
                    println!(" {} ", value);
                } else if expr_str.trim().starts_with('"') && expr_str.trim().ends_with('"') {
                    let s = &expr_str.trim()[1..expr_str.trim().len()-1];
                    println!("{}", s);
                } else {
                    println!("No program loaded. Can only evaluate simple constants.");
                }
            }
        } else {
            self.usage("?");
        }
    }
    
    /// Next command
    fn cmd_next(&mut self, _args: Option<&str>) {
        if let Some(ref mut interpreter) = self.interpreter {
            // Store the current location before stepping
            let before_location = *interpreter.get_current_location();
            let program = interpreter.get_program().clone();
            
            match interpreter.step() {
                Ok(()) => {
                    let status = interpreter.get_run_status();
                    match status {
                        RunStatus::Run => {
                            // Show what we just executed
                            if before_location.index < program.lines.len() {
                                let executed_line = &program.lines[before_location.index];
                                println!("Executed: {}: {}", executed_line.line_number, executed_line.source);
                                if before_location.offset > 0 {
                                    println!("  (Statement {} of {})", before_location.offset + 1, executed_line.statements.len());
                                }
                            }
                            
                            // Show where we are now
                            println!("Next: ");
                            self.print_current();
                        }
                        RunStatus::EndNormal => println!("Program completed successfully"),
                        RunStatus::EndStop => println!("Program stopped"),
                        RunStatus::EndOfProgram => println!("Program reached end"),
                        _ => println!("Program completed with status: {:?}", status),
                    }
                }
                Err(e) => {
                    match e {
                        BasicError::Runtime { message, basic_line_number, .. } => {
                            if let Some(line_num) = basic_line_number {
                                println!("Runtime Error in line {}: {}", line_num, message);
                            } else {
                                println!("Runtime Error: {}", message);
                            }
                        }
                        BasicError::Syntax { message, basic_line_number, .. } => {
                            if let Some(line_num) = basic_line_number {
                                println!("Syntax Error in line {}: {}", line_num, message);
                            } else {
                                println!("Syntax Error: {}", message);
                            }
                        }
                        _ => println!("Error: {:?}", e),
                    }
                }
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Continue command
    fn cmd_continue(&mut self, args: Option<&str>) {
        if let Some(ref mut interpreter) = self.interpreter {
            let _single_step = args == Some("step");
            
            // TODO: Implement program execution with breakpoints
            match interpreter.run() {
                Ok(()) => {
                    let status = interpreter.get_run_status();
                    match status {
                        RunStatus::EndNormal => println!("Program completed successfully"),
                        RunStatus::EndStop => println!("Program stopped"),
                        RunStatus::EndOfProgram => println!("Program reached end"),
                        RunStatus::BreakCode => {
                            println!("Breakpoint!");
                            self.print_current();
                        }
                        RunStatus::BreakData => {
                            println!("Data Breakpoint!");
                            self.print_current();
                        }
                        _ => println!("Program completed with status: {:?}", status),
                    }
                }
                Err(e) => {
                    match e {
                        BasicError::Runtime { message, basic_line_number, .. } => {
                            if let Some(line_num) = basic_line_number {
                                println!("Runtime Error in line {}: {}", line_num, message);
                            } else {
                                println!("Runtime Error: {}", message);
                            }
                        }
                        BasicError::Syntax { message, basic_line_number, .. } => {
                            if let Some(line_num) = basic_line_number {
                                println!("Syntax Error in line {}: {}", line_num, message);
                            } else {
                                println!("Syntax Error: {}", message);
                            }
                        }
                        _ => println!("Error: {:?}", e),
                    }
                }
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Run command
    fn cmd_run(&mut self, args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            let enable_coverage = args == Some("coverage");
            
            // Create fresh interpreter with same program
            let program = interpreter.get_program().clone();
            let mut new_interpreter = Interpreter::new(program);
            
            if enable_coverage {
                new_interpreter.enable_coverage();
                self.coverage_enabled = true;
            } else {
                self.coverage_enabled = false;
            }
            
            // Transfer breakpoints to the new interpreter
            self.transfer_breakpoints_to_interpreter(&mut new_interpreter);
            
            self.interpreter = Some(new_interpreter);
            self.cmd_continue(None);
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Benchmark command
    fn cmd_benchmark(&mut self, _args: Option<&str>) {
        if let Some(ref file) = self.program_file.clone() {
            let load_start = Instant::now();
            self.load_from_file(&file, false);
            let load_time = load_start.elapsed();
            
            let run_start = Instant::now();
            self.cmd_continue(None);
            let run_time = run_start.elapsed();
            
            println!("Load time {:10.3} sec. Run time: {:10.3} sec.", 
                     load_time.as_secs_f64(), run_time.as_secs_f64());
        } else {
            println!("No program file to benchmark");
        }
    }
    
    /// Format command
    fn cmd_format(&mut self, _args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            let program = interpreter.get_program();
            
            // Display formatted lines using canonical form from statements
            for line in &program.lines {
                // Use the Display implementation of ProgramLine to get canonical form
                println!("{:5} {}", line.line_number, {
                    let mut stmt_str = String::new();
                    for (i, stmt) in line.statements.iter().enumerate() {
                        stmt_str.push_str(&format!("{}", stmt));
                        if i < line.statements.len() - 1 {
                            stmt_str.push_str(" : ");
                        }
                    }
                    stmt_str
                });
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Renumber command
    fn cmd_renum(&mut self, _args: Option<&str>) {
        if self.interpreter.is_none() {
            println!("No program has been loaded yet.");
            return;
        }
        
        // TODO: Implement program renumbering
        println!("Program renumbering not yet implemented");
    }
    
    /// Break command
    fn cmd_break(&mut self, args: Option<&str>) {
        match args {
            Some("clear") => {
                self.breakpoints.clear();
                self.data_breakpoints.clear();
                println!("All breakpoints cleared");
            }
            Some("list") | None => {
                if !self.breakpoints.is_empty() {
                    println!("Breakpoints:");
                    for (line, offset) in &self.breakpoints {
                        println!("\t{} {}", line, offset);
                    }
                }
                if !self.data_breakpoints.is_empty() {
                    println!("Data breakpoints:");
                    for bp in &self.data_breakpoints {
                        println!("\t{}", bp);
                    }
                }
            }
            Some(args) => {
                let parts: Vec<&str> = args.split_whitespace().collect();
                if parts.is_empty() {
                    self.usage("break");
                    return;
                }
                
                if let Ok(line_number) = parts[0].parse::<usize>() {
                    let offset = if parts.len() > 1 {
                        parts[1].parse::<usize>().unwrap_or(0)
                    } else {
                        0
                    };
                    
                    self.breakpoints.push((line_number, offset));
                    println!("Added breakpoint at line: {} clause: {}", line_number, offset);
                } else {
                    self.data_breakpoints.push(args.to_string());
                    println!("Added data breakpoint: {}", args);
                }
            }
        }
    }
    
    /// Help command
    fn cmd_help(&self, args: Option<&str>) {
        if let Some(cmd) = args {
            if let Some(help_text) = self.get_help_text(cmd) {
                println!("{}", help_text);
            } else {
                println!("Unknown command: {}", cmd);
            }
        } else {
            println!("General Commands:");
            println!("\t?         - Evaluate expression");
            println!("\tbenchmark - Run program with timing");
            println!("\tclear     - Clear program and state");
            println!("\tcontinue  - Continue execution");
            println!("\thelp      - Show help");
            println!("\tlist      - List program");
            println!("\tload      - Load program");
            println!("\tquit      - Exit shell");
            println!("\trun       - Run program");
            println!("\tsave      - Save program");
            println!("\tstop      - Stop execution");
            println!();
            println!("Debug Commands:");
            println!("\tbreak     - Set breakpoint");
            println!("\tcoverage  - Show coverage");
            println!("\tforstack  - Show FOR stack");
            println!("\tgosubs    - Show GOSUB stack");
            println!("\tnext      - Execute next line");
            println!("\tsymbols   - Show symbols");
            println!();
            println!("Commands can be abbreviated to shortest unique prefix.");
            println!("For convenience, 'r' works for 'run', and 'c' for 'continue'");
            println!();
            println!("BASIC Line Entry:");
            println!("\t<number> <statements> - Insert/replace program line");
            println!("\t<number>              - Delete program line");
            println!("\tExamples:");
            println!("\t\t100 PRINT \"HELLO\"");
            println!("\t\t200 FOR I=1 TO 10: PRINT I: NEXT I");
            println!("\t\t100                  (deletes line 100)");
            println!("\tLine numbers must be 1-65536");
        }
    }
    
    /// Statements command
    fn cmd_stmts(&self, args: Option<&str>) {
        if let Some(ref interpreter) = self.interpreter {
            let program = interpreter.get_program();
            
            // Parse optional line number argument
            let target_line_number = if let Some(args) = args {
                match args.trim().parse::<usize>() {
                    Ok(line_num) => Some(line_num),
                    Err(_) => {
                        println!("Invalid line number: {}", args.trim());
                        self.usage("statements");
                        return;
                    }
                }
            } else {
                None
            };
            
            // Display statements
            for line in &program.lines {
                // If they give us a line number, only print that line's statements
                if let Some(target) = target_line_number {
                    if target != line.line_number {
                        continue;
                    }
                }
                
                print!("{} ", line.line_number);
                for (i, statement) in line.statements.iter().enumerate() {
                    if i > 0 {
                        print!("|");
                    }
                    print!("\t{}", statement);
                }
                println!();
            }
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Stop command
    fn cmd_stop(&mut self, _args: Option<&str>) {
        if let Some(ref mut interpreter) = self.interpreter {
            interpreter.restart();
            println!("Program execution reset to beginning");
        } else {
            println!("No program has been loaded yet.");
        }
    }
    
    /// Handle BASIC line entry (e.g., "100 PRINT A")
    fn handle_line_entry(&mut self, line_input: &str) {
        // Parse line number and content
        let parts: Vec<&str> = line_input.splitn(2, ' ').collect();
        if let Ok(line_number) = parts[0].parse::<usize>() {
            if line_number < 1 || line_number > 65536 {
                println!("Line number {} out of range (1-65536)", line_number);
                return;
            }
            
            if parts.len() == 1 || parts[1].trim().is_empty() {
                // Delete line
                if let Some(ref mut interpreter) = self.interpreter {
                    let mut program = interpreter.get_program().clone();
                    program.remove_line(line_number);
                    let mut new_interpreter = Interpreter::new(program);
                    self.transfer_breakpoints_to_interpreter(&mut new_interpreter);
                    self.interpreter = Some(new_interpreter);
                    println!("Line {} deleted", line_number);
                } else {
                    println!("No program loaded");
                }
            } else {
                // Insert/replace line
                if let Some(ref mut interpreter) = self.interpreter {
                    let line_content = parts[1].trim();
                    let full_line = format!("{} {}", line_number, line_content);
                    
                    // Parse the new line
                    let mut lexer = Lexer::new(&full_line);
                    match lexer.tokenize() {
                        Ok(tokens) => {
                            let mut parser = Parser::new(tokens);
                            match parser.parse() {
                                                                 Ok(temp_program) => {
                                     if let Some(new_line) = temp_program.lines.first() {
                                         let mut program = interpreter.get_program().clone();
                                         program.add_line(line_number, line_content.to_string(), new_line.statements.clone());
                                         let mut new_interpreter = Interpreter::new(program);
                                         self.transfer_breakpoints_to_interpreter(&mut new_interpreter);
                                         self.interpreter = Some(new_interpreter);
                                         println!("Line {} updated", line_number);
                                     } else {
                                         println!("Error: Could not parse line");
                                     }
                                 }
                                Err(e) => {
                                    println!("Parse error: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Lexer error: {}", e);
                        }
                    }
                                 } else {
                     // No program loaded yet - create a new one
                     let line_content = parts[1].trim();
                     let full_line = format!("{} {}", line_number, line_content);
                     let mut lexer = Lexer::new(&full_line);
                     match lexer.tokenize() {
                         Ok(tokens) => {
                             let mut parser = Parser::new(tokens);
                             match parser.parse() {
                                 Ok(temp_program) => {
                                     if let Some(new_line) = temp_program.lines.first() {
                                         let mut program = Program::new();
                                         program.add_line(line_number, line_content.to_string(), new_line.statements.clone());
                                         let mut new_interpreter = Interpreter::new(program);
                                         self.transfer_breakpoints_to_interpreter(&mut new_interpreter);
                                         self.interpreter = Some(new_interpreter);
                                         println!("Line {} added to new program", line_number);
                                     } else {
                                         println!("Error: Could not parse line");
                                     }
                                 }
                                 Err(e) => {
                                     println!("Parse error: {}", e);
                                 }
                             }
                         }
                         Err(e) => {
                             println!("Lexer error: {}", e);
                         }
                     }
                 }
            }
        } else {
            println!("Invalid line number");
        }
    }
    
    /// Find command by prefix
    fn find_command(&self, prefix: &str) -> Option<String> {
        // Handle abbreviations
        let prefix = match prefix {
            "r" => "run",
            "c" => "continue",
            _ => prefix,
        };
        
        let commands = [
            "?", "benchmark", "break", "clear", "continue", "coverage",
            "exit", "format", "forstack", "gosubs", "help", "list",
            "load", "next", "quit", "renumber", "run", "save",
            "statements", "stop", "symbols"
        ];
        
        let matches: Vec<&str> = commands.iter()
            .filter(|cmd| cmd.starts_with(prefix))
            .cloned()
            .collect();
        
        if matches.len() == 1 {
            Some(matches[0].to_string())
        } else {
            None
        }
    }
    
    /// Execute a command
    fn execute_command(&mut self, cmd: &str, args: Option<&str>) {
        match cmd {
            "?" => self.cmd_print(args),
            "benchmark" => self.cmd_benchmark(args),
            "break" => self.cmd_break(args),
            "clear" => self.cmd_clear(args),
            "continue" => self.cmd_continue(args),
            "coverage" => self.cmd_coverage(args),
            "exit" | "quit" => self.cmd_quit(args),
            "format" => self.cmd_format(args),
            "forstack" => self.cmd_for_stack(args),
            "gosubs" => self.cmd_gosub_stack(args),
            "help" => self.cmd_help(args),
            "list" => self.cmd_list(args),
            "load" => self.cmd_load(args),
            "next" => self.cmd_next(args),
            "renumber" => self.cmd_renum(args),
            "run" => self.cmd_run(args),
            "save" => self.cmd_save(args),
            "statements" => self.cmd_stmts(args),
            "stop" => self.cmd_stop(args),
            "symbols" => self.cmd_symbols(args),
            _ => println!("Unknown command: {}", cmd),
        }
    }
    
    /// Main command loop
    pub fn run(&mut self) {
        println!("BASIC Shell - Rust Version");
        println!("Type 'help' for available commands");
        
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    if input.is_empty() {
                        continue;
                    }
                    
                    // Check if this is a BASIC line entry (starts with digits)
                    if input.chars().next().unwrap_or(' ').is_ascii_digit() {
                        self.handle_line_entry(input);
                        continue;
                    }
                    
                    // Handle ? command specially
                    let input = if input.starts_with('?') && input.len() > 1 && !input.chars().nth(1).unwrap().is_whitespace() {
                        format!("? {}", &input[1..])
                    } else {
                        input.to_string()
                    };
                    
                    // Parse command and arguments
                    let parts: Vec<&str> = input.splitn(2, ' ').collect();
                    let cmd = parts[0];
                    let args = if parts.len() > 1 { Some(parts[1]) } else { None };
                    
                    // Find command by prefix
                    let full_cmd = self.find_command(cmd);
                    if let Some(full_cmd) = full_cmd {
                        self.execute_command(&full_cmd, args);
                    } else {
                        println!("Unknown command: {}", cmd);
                        self.cmd_help(None);
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_file = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    
    let mut shell = BasicShell::new(program_file);
    shell.run();
} 