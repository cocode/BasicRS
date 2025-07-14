use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::{self, Write};
use crate::basic_symbols::SymbolTable;
use crate::basic_reports::CoverageData;

use crate::basic_types::{
    Program, ProgramLine, Statement, Expression, BasicError,
    ExpressionType, RunStatus, SymbolValue, Token, PrintItem,
};

use crate::basic_function_registry::FUNCTION_REGISTRY;
use crate::basic_operators::{BASIC_FALSE_F, BASIC_TRUE_F};
use crate::basic_dialect::UPPERCASE_INPUT;

const TRACE_FILE_NAME: &str = "basic_trace.txt";

// Control location in program
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlLocation {
    pub index: usize,    // Index into program lines
    pub offset: usize,   // Offset into statements in the line
}

impl fmt::Display for ControlLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ControlLocation(index={}, offset={})", self.index, self.offset)
    }
}

// For loop record
#[derive(Debug, Clone)]
pub struct ForRecord {
    pub var: String,     // Loop variable name
    pub stop: Expression,    // Stop value expression
    pub step: Expression,    // Step value expression
    pub stmt: Option<ControlLocation>, // Statement location
}

pub struct Interpreter {
    program: Program,
    location: ControlLocation,
    internal_symbols: SymbolTable,  // Internal symbol table for function definitions
    symbols: SymbolTable,           // Current scope symbol table
    for_stack: Vec<ForRecord>,
    gosub_stack: Vec<ControlLocation>,
    data_pointer: usize,
    data_values: Vec<SymbolValue>,
    data_line_map: HashMap<usize, usize>, // Maps line numbers to data positions
    run_status: RunStatus,
    trace_file: Option<File>,
    coverage: Option<CoverageData>,
    breakpoints: HashSet<(usize, usize)>,
    data_breakpoints: HashSet<String>,
    line_number_map: HashMap<usize, usize>, // Maps line numbers to program indices
    // Normally, after a statement is executed, we advance to the next line, in the main loop
    // But if we just did a control transfer, like a GOTO, we don't then want to advance to
    // the next statement in the main loop, we are already where we want to be. So set this
    // on control transfers. (GOTO, GOSUB, FOR/NEXT, IF. Anything else?)
    advance_stmt: bool,
    cursor_position: usize,     // Current cursor position for PRINT formatting
}

impl Interpreter {
    /// Helper method to add line number information to errors that don't have it
    fn add_line_info_to_error(&self, error: BasicError) -> BasicError {
        match error {
            BasicError::Syntax { message, basic_line_number: None, file_line_number } => {
                BasicError::Syntax {
                    message,
                    basic_line_number: Some(self.get_current_line().line_number),
                    file_line_number,
                }
            }
            BasicError::Runtime { message, basic_line_number: None, file_line_number } => {
                BasicError::Runtime {
                    message,
                    basic_line_number: Some(self.get_current_line().line_number),
                    file_line_number,
                }
            }
            BasicError::Type { message, basic_line_number: None, file_line_number } => {
                BasicError::Type {
                    message,
                    basic_line_number: Some(self.get_current_line().line_number),
                    file_line_number,
                }
            }
            BasicError::Internal { message, basic_line_number: None, file_line_number } => {
                BasicError::Internal {
                    message,
                    basic_line_number: Some(self.get_current_line().line_number),
                    file_line_number,
                }
            }
            // If error already has line number info, return as-is
            other => other,
        }
    }

    pub fn new(program: Program) -> Self {
        let mut line_number_map = HashMap::new();
        for (i, line) in program.lines.iter().enumerate() {
            line_number_map.insert(line.line_number, i);
        }



        let internal_symbols = SymbolTable::new();
        let symbols = internal_symbols.get_nested_scope();
        
        Interpreter {
            program,
            location: ControlLocation { index: 0, offset: 0 },
            internal_symbols,
            symbols,
            for_stack: Vec::new(),
            gosub_stack: Vec::new(),
            data_pointer: 0,
            data_values: Vec::new(), // Initialize to empty, data values are collected later
            data_line_map: HashMap::new(),
            run_status: RunStatus::Run,
            trace_file: None,
            coverage: None,
            breakpoints: HashSet::new(),
            data_breakpoints: HashSet::new(),
            line_number_map,
            advance_stmt: true,
            cursor_position: 0,
        }
    }

    // All location changes should be either through control_transfer or advance_location
    fn control_transfer(&mut self, new_loc: ControlLocation) {
        self.location = new_loc;
        self.advance_stmt = false
    }
    fn advance_location(&mut self) {
        if  !self.advance_stmt {
            self.advance_stmt = true;
            return
        }
        let current_line = self.get_current_line();
        if self.location.offset + 1 < current_line.statements.len() {
            // Move to next statement in current line
            self.location.offset += 1;
        } else {
            // Move to first statement of next line
            if self.location.index + 1 < self.program.lines.len() {
                self.location.index += 1;
                self.location.offset =  0;
            } else {
                self.run_status = RunStatus::EndOfProgram;
            }
        }
    }
    fn goto_next_line(&mut self) {
        if self.location.index + 1 < self.program.lines.len() {
            self.control_transfer(ControlLocation {
            index: self.location.index + 1,
            offset:  0,
            });
        } else {
            self.run_status = RunStatus::EndOfProgram;
        }
    }

    /// Finds the matching `NEXT` statement for a `FOR` loop variable.
    ///
    /// This function performs a lexical search starting from the current location,
    /// scanning forward through the program to find the corresponding `NEXT` for the
    /// given variable name. Nested `FOR` loops are correctly handled via depth tracking.
    ///
    /// this returns with the location of the next statement. The normal call to advance_location()
    /// in the main loop will move us to the statement after the next. Except we did a control
    /// transfer to get to the next, so the dont_adveance flag will be set.  so set it manually
    /// after callling this function
    ///
    /// # Arguments
    ///
    /// * `var` - The name of the loop variable to match with a `NEXT`.
    ///
    /// # Returns
    ///
    /// A `ControlLocation` pointing to the statement immediately after the matching `NEXT`,
    /// or an error if no match is found.
    ///
    /// # Errors
    ///
    /// Returns `BasicError::Runtime` if the `NEXT` is missing or mismatched.
    fn find_matching_next(&self, var: &str) -> Result<ControlLocation, BasicError> {

        let mut depth = 0;

        for (i, line) in self.program.lines.iter().enumerate().skip(self.location.index) {
            let start_offset = if i == self.location.index {
                self.location.offset + 1
            } else {
                0
            };

            for (j, stmt) in line.statements.iter().enumerate().skip(start_offset) {
                match stmt {
                    Statement::For { .. } => {
                        depth += 1;
                    }
                    Statement::Next { var: next_var } => {
                        if depth == 0 {
                            if next_var == var {
                                return Ok(ControlLocation { index: i, offset: j });
                            } else {
                                return Err(BasicError::Runtime {
                                    message: format!("Unexpected NEXT for '{}' while looking for NEXT for '{}'", next_var, var),
                                    basic_line_number: Some(self.program.lines[i].line_number),
                                    file_line_number: None,
                                });
                            }
                        } else {
                            depth -= 1;
                        }
                    }
                    _ => {}
                }
            }
        }

        Err(BasicError::Runtime {
            message: format!("No matching NEXT found for FOR {}", var),
            basic_line_number: Some(self.get_current_line().line_number),
            file_line_number: None,
        })
    }

    pub fn enable_trace(&mut self) -> io::Result<()> {
        self.trace_file = Some(File::create(TRACE_FILE_NAME)?);
        Ok(())
    }

    pub fn enable_coverage(&mut self) {
        self.coverage = Some(CoverageData::new());
    }

    pub fn add_breakpoint(&mut self, line: usize, offset: usize) {
        self.breakpoints.insert((line, offset));
    }

    pub fn add_data_breakpoint(&mut self, var: String) {
        self.data_breakpoints.insert(var);
    }



    pub fn get_symbol_value(&self, name: &str) -> Option<&SymbolValue> {
        // First try to find scalar variable with original name
        if let Some(value) = self.symbols.get_symbol(name) {
            Some(value)
        } else {
            // If not found, try to find array with [] suffix
            let array_key = format!("{}[]", name);
            self.symbols.get_symbol(&array_key)
        }
    }
    
    pub fn get_all_symbols(&self) -> std::collections::HashMap<String, SymbolValue> {
        self.symbols.dump()
    }
    
    pub fn get_program(&self) -> &Program {
        &self.program
    }
    
    pub fn get_current_location(&self) -> &ControlLocation {
        &self.location
    }
    
    pub fn get_for_stack(&self) -> &Vec<ForRecord> {
        &self.for_stack
    }
    
    pub fn get_gosub_stack(&self) -> &Vec<ControlLocation> {
        &self.gosub_stack
    }
    
    pub fn restart(&mut self) {
        self.location = ControlLocation { index: 0, offset: 0 };
        self.run_status = RunStatus::Run;
        self.for_stack.clear();
        self.gosub_stack.clear();
        self.cursor_position = 0;
        // Reset symbols to initial state but keep the program
        self.symbols = self.internal_symbols.get_nested_scope();
    }

    pub fn set_symbol_value(&mut self, name: String, value: SymbolValue) {
        self.symbols.put_symbol(name, value);
    }

    pub fn get_current_line_number(&self) -> usize {
        self.get_current_line().line_number
    }

    pub fn get_run_status(&self) -> RunStatus {
        self.run_status
    }

    pub fn set_run_status(&mut self, status: RunStatus) {
        self.run_status = status;
    }

    pub fn run(&mut self) -> Result<(), BasicError> {
        // Collect all data values and build line mapping
        for pl in &self.program.lines {
            for stmt in &pl.statements {
                if let Statement::Data { values } = stmt {
                    // Record the current position as the start of this line's data
                    self.data_line_map.insert(pl.line_number, self.data_values.len());
                    self.data_values.extend(values.iter().cloned());
                }
            }
        }

        while self.run_status == RunStatus::Run {
            let current_line = self.get_current_line().line_number;
            // println!("current line {}", current_line);
            let current_offset = self.location.offset;
            
            // Check breakpoints
            if self.breakpoints.contains(&(current_line, current_offset)) {
                self.run_status = RunStatus::BreakCode;
                return Ok(());
            }
            
            // Get current statement before any trace/coverage operations
            let current_stmt = self.get_current_stmt().clone();

            // Write trace
            self.do_trace(&current_stmt);

            // Update coverage before executing
            if let Some(ref mut cov) = self.coverage {
                cov.entry(current_line)
                    .or_insert_with(HashSet::new)
                    .insert(current_offset);
            }
            if false {
                println!("Symbol Table at line {} at {}", current_line, current_offset);
                let symbols = self.get_symbol_table();
                for (name, value) in symbols.dump() {
                    println!("{} = {}", name, value);
                }
                println!("Symbol Table END");
            }
            // Execute statement
            match self.execute_statement(&current_stmt) {
                Ok(()) => {
                    self.advance_location();
                }
                Err(err) => {
                    self.run_status = match err {
                        BasicError::Syntax { .. } => RunStatus::EndErrorSyntax,
                        BasicError::Runtime { .. } => RunStatus::EndErrorRuntime,
                        BasicError::Internal { .. } => RunStatus::EndErrorInternal,
                        BasicError::Type { .. } => RunStatus::EndErrorType,
                    };
                    return Err(err);
                }
            }
        }
        Ok(())
    }

    fn do_trace(&mut self, current_stmt: &Statement) {
        let current_line_number = self.get_current_line_number();
        let current_line_offset = self.location.offset;
        if let Some(ref mut file) = self.trace_file {
            if self.location.offset == 0 {
                writeln!(file, ">{}:{}", current_line_number, current_line_offset).ok();
            }
            writeln!(file, "\t{:?}", &current_stmt).ok();
        }
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<(), BasicError> {
        match stmt {
            Statement::Let { var, value } => {
                let result = self.evaluate_expression(value)?;
                match &var.expr_type {
                    ExpressionType::Variable(name) => {
                        self.put_symbol(name.clone(), result);
                        Ok(())
                    }
                    ExpressionType::Array { name, indices } => {
                        let idx_values: Result<Vec<usize>, BasicError> = indices.iter()
                            .map(|expr| match self.evaluate_expression(expr)? {
                                SymbolValue::Number(n) if n >= 0.0 => Ok(n as usize),
                                SymbolValue::Number(n) => Err(BasicError::Runtime {
                                    message: format!("Array index must be non-negative, got: {}", n),
                                    basic_line_number: Some(self.get_current_line().line_number),
                                    file_line_number: None,
                                }),
                                _ => Err(BasicError::Runtime {
                                    message: "Array index must be a number".to_string(),
                                    basic_line_number: Some(self.get_current_line().line_number),
                                    file_line_number: None,
                                })
                            })
                            .collect();
                        let indices = idx_values?;
                        self.symbols
                            .set_array_element(name, &indices, result)
                            .map_err(|mut err| {
                                if let BasicError::Runtime {
                                    ref mut basic_line_number,
                                    ref mut file_line_number,
                                    ..
                                } = err
                                {
                                    *basic_line_number = Some(self.get_current_line().line_number);
                                    *file_line_number = None;
                                }
                                err
                            })?;
                        Ok(())
                    }
                    _ => {
                        // Try to evaluate the left-hand side as an expression
                        // This handles cases like LET A = B where A might be a variable
                        if let ExpressionType::Variable(name) = &var.expr_type {
                            self.put_symbol(name.clone(), result);
                            Ok(())
                        } else {
                            Err(BasicError::Runtime {
                                message: "Invalid left-hand side in assignment".to_string(),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            })
                        }
                    }
                }
            }
            Statement::Print { items } => {
                let mut needs_newline = true;
                
                for item in items {
                    match item {
                        PrintItem::Expression(expr) => {
                            let value = self.evaluate_expression(expr)?;
                            let value_str = value.to_string();
                            print!("{}", value_str);
                            self.cursor_position += value_str.len();
                        }
                        PrintItem::Tab(n) => {
                            // Move cursor to specific column (1-based)
                            if *n > self.cursor_position {
                                let spaces_needed = *n - self.cursor_position;
                                print!("{}", " ".repeat(spaces_needed));
                                self.cursor_position = *n;
                            }
                        }
                        PrintItem::Comma => {
                            // Tab to next column (every 8 characters, standard tab stops)
                            let next_tab = ((self.cursor_position / 8) + 1) * 8;
                            if next_tab > self.cursor_position {
                                let spaces_needed = next_tab - self.cursor_position;
                                print!("{}", " ".repeat(spaces_needed));
                                self.cursor_position = next_tab;
                            }
                        }
                        PrintItem::Semicolon => {
                            // Semicolon suppresses spacing and newlines
                            // If this is the last item, suppress newline
                            if item == items.last().unwrap() {
                                needs_newline = false;
                            }
                            // Semicolons add no spacing at all
                        }
                    }
                }
                
                // Add newline unless last item was a semicolon
                if needs_newline {
                    println!();
                    self.cursor_position = 0;
                }
                
                io::stdout().flush()?;
                Ok(())
            }
            Statement::Input { vars, prompt } => {
                const MAX_RETRIES: usize = 3;
                let mut retry_count = 0;
                
                loop {
                    let mut input = String::new();
                    if let Some(p) = prompt {
                        print!("{}? ", p);
                    } else {
                        print!("? ");
                    }
                    io::stdout().flush()?;
                    io::stdin().read_line(&mut input)?;
                    
                    // Split input by commas and process each part
                    let input_parts: Vec<&str> = input.trim().split(',').collect();
                    
                    // Check if we have the right number of inputs
                    if input_parts.len() != vars.len() {
                        retry_count += 1;
                        if retry_count >= MAX_RETRIES {
                            return Err(BasicError::Runtime {
                                message: format!("Expected {} input values, got {}. Maximum retries exceeded.", vars.len(), input_parts.len()),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            });
                        }
                        println!("?Redo from start");
                        continue;
                    }
                    
                    // Process each variable and its corresponding input
                    let mut values = Vec::new();
                    let mut parse_error = false;
                    
                    for (i, var) in vars.iter().enumerate() {
                        let input_part = input_parts[i].trim();
                        let is_string_variable = var.ends_with('$');
                        
                        let value = if is_string_variable {
                            // For string variables (A$), always treat input as string
                            let processed_str = if UPPERCASE_INPUT {
                                input_part.to_uppercase()
                            } else {
                                input_part.to_string()
                            };
                            SymbolValue::String(processed_str)
                        } else {
                            // For numeric variables (A), try to parse as number
                            if let Ok(n) = input_part.parse::<f64>() {
                                SymbolValue::Number(n)
                            } else {
                                parse_error = true;
                                break;
                            }
                        };
                        values.push((var.clone(), value));
                    }
                    
                    if parse_error {
                        retry_count += 1;
                        if retry_count >= MAX_RETRIES {
                            return Err(BasicError::Runtime {
                                message: "Invalid numeric input. Maximum retries exceeded.".to_string(),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            });
                        }
                        println!("?Redo from start");
                        continue;
                    }
                    
                    // All inputs were valid, store the values
                    for (var, value) in values {
                        self.put_symbol(var, value);
                    }
                    break;
                }
                Ok(())
            }
            Statement::If { condition } => {
                let result = self.evaluate_expression(condition)?;
                match result {
                    SymbolValue::Number(n) => {
                        if n == BASIC_FALSE_F {
                            // Condition is false, skip to ELSE or next line
                            self.goto_else_or_next_line()?;
                        }
                        // If condition is true, continue to next statement
                    }
                    _ => return Err(BasicError::Type {
                        message: "IF condition must evaluate to a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number), // TODO point to correct statement on multi-statement line
                        file_line_number: None,
                    }),
                }
                Ok(())
            }
            Statement::Then => {
                Ok(())
            }
            Statement::Else => {
                // An ELSE statement acts as the end of the THEN before it.
                // We want to advance to the next else (in the case of nested IF/THEN/ELSEs)
                // Or go to the next line.
                self.advance_location();        // Skip this ELSE
                self.goto_else_or_next_line()?; // And look for the next one.
                Ok(())
            }
            Statement::For { var, start, stop, step } => {
                let start_value = self.evaluate_expression(start)?;
                let stop_expr = stop.clone(); // TODO CAN THIS BE AN EXPRESSION?
                let step_expr = step.clone().unwrap_or_else(|| Expression::new_number(1.0));
                //println!("for loop starting at {}", start_value);
                let current = match start_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop start value must be a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    }),
                };

                let stop_value = self.evaluate_expression(&stop_expr)?;
                let stop = match stop_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop stop value must be a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    }),
                };

                let step_value = self.evaluate_expression(&step_expr)?;
                let step = match step_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop step must be a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    }),
                };

                self.put_symbol(var.clone(), SymbolValue::Number(current));

                // Check if loop should run
                if (step >= 0.0 && current > stop) || (step < 0.0 && current < stop) {
                    // Loop won't run: jump past the matching NEXT
                    let next_loc = self.find_matching_next(var)?;
                    // We have the location of the next, we want to advance past it.
                    // Normally control_transfer takes you TO a location.
                    // We want the location after that.
                    self.control_transfer(next_loc);
                    self.advance_stmt = true;
                } else {
                    // Loop will run: push loop frame
                    self.for_stack.push(ForRecord {
                        var: var.clone(),
                        stop: stop_expr,
                        step: step_expr,
                        stmt: Some(self.location),
                    });
                }

                Ok(())
            }
            Statement::Next { var } => {
                if let Some(for_record) = self.for_stack.last().cloned() {
                    if &for_record.var != var {
                        return Err(BasicError::Runtime {
                            message: format!("Mismatched NEXT: expected '{}', found '{}'", for_record.var, var),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        });
                    }

                    let current_value = self.get_symbol(var)?;
                    let current = match current_value {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop variable must be numeric".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        }),
                    };

                    let step = match self.evaluate_expression(&for_record.step)? {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop step must be numeric".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        }),
                    };

                    let stop = match self.evaluate_expression(&for_record.stop)? {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop stop value must be numeric".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        }),
                    };
                    let next_value = current + step;
                    self.put_symbol(var.clone(), SymbolValue::Number(next_value));
                    let val1 = self.get_symbol(var)?;
                    if (step >= 0.0 && next_value <= stop) || (step < 0.0 && next_value >= stop) {
                        if let Some(stmt_loc) = for_record.stmt {
                            self.control_transfer(stmt_loc);
                            self.advance_stmt = true;
                            self.for_stack.pop();         // Remove old frame
                            self.for_stack.push(for_record); // Re-push updated one
                        }
                    } else {
                        self.for_stack.pop();
                    }

                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "NEXT without matching FOR".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    })
                }
            }
            Statement::Goto { line } => {
                self.goto_line(*line)?;
                Ok(())
            }
            Statement::Gosub { line } => {
                self.gosub_stack.push(self.location);
                self.goto_line(*line)?;
                Ok(())
            }
            Statement::Return => {
                if let Some(return_loc) = self.gosub_stack.pop() {
                    self.control_transfer(return_loc);
                    self.advance_stmt = true;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "RETURN without GOSUB".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    })
                }
            }
            Statement::End => {
                self.run_status = RunStatus::EndNormal;
                Ok(())
            }
            Statement::Stop => {
                self.run_status = RunStatus::EndStop;
                Ok(())
            }
            Statement::Rem { .. } => Ok(()),
            Statement::Data { .. } => Ok(()),
            Statement::Read { vars } => {
                for var_expr in vars {
                    if self.data_pointer >= self.data_values.len() {
                        return Err(BasicError::Runtime {
                            message: "Out of DATA values".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        });
                    }
                    
                    let value = self.data_values[self.data_pointer].clone();
                    self.data_pointer += 1;
                    
                    match &var_expr.expr_type {
                        ExpressionType::Variable(name) => {
                            self.put_symbol(name.clone(), value);
                        }
                        ExpressionType::Array { name, indices } => {
                            let idx_values: Result<Vec<usize>, BasicError> = indices.iter()
                                .map(|expr| match self.evaluate_expression(expr)? {
                                    SymbolValue::Number(n) if n >= 0.0 => Ok(n as usize),
                                    SymbolValue::Number(n) => Err(BasicError::Runtime {
                                        message: "Array index must be non-negative".to_string(),
                                        basic_line_number: Some(self.get_current_line().line_number),
                                        file_line_number: None,
                                    }),
                                    _ => Err(BasicError::Runtime {
                                        message: "Array index must be a number".to_string(),
                                        basic_line_number: Some(self.get_current_line().line_number),
                                        file_line_number: None,
                                    })
                                })
                                .collect();
                            let indices = idx_values?;
                            self.symbols.set_array_element(name, &indices, value)?;
                        }
                        _ => {
                            return Err(BasicError::Runtime {
                                message: "Invalid variable in READ statement".to_string(),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            });
                        }
                    }
                }
                Ok(())
            }
            Statement::Restore { line } => {
                if let Some(line_num) = line {
                    // Find the line number and reset data pointer to that line's data start
                    if let Some(&data_pos) = self.data_line_map.get(line_num) {
                        self.data_pointer = data_pos;
                    } else {
                        return Err(BasicError::Runtime {
                            message: format!("Line {} has no DATA statements", line_num),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        });
                    }
                } else {
                    // Reset to beginning
                    self.data_pointer = 0;
                }
                Ok(())
            }
            Statement::Dim { arrays } => {
                for array in arrays {
                    self.symbols.create_array(array.name.clone(), array.dimensions.clone()).map_err(|e| self.add_line_info_to_error(e))?;
                }
                Ok(())
            }
            Statement::OnGoto { expr, line_numbers } => {
                let value = match self.evaluate_expression(expr)? {
                    SymbolValue::Number(n) if n >= 1.0 && n.fract() == 0.0 => n as usize,
                    _ => return Err(BasicError::Runtime {
                        message: "ON index must be a positive integer".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    })
                };
                
                if value <= line_numbers.len() {
                    self.goto_line(line_numbers[value - 1])?;
                }
                Ok(())
            }
            Statement::OnGosub { expr, line_numbers } => {
                let value = match self.evaluate_expression(expr)? {
                    SymbolValue::Number(n) if n >= 1.0 && n.fract() == 0.0 => n as usize,
                    _ => return Err(BasicError::Runtime {
                        message: "ON index must be a positive integer".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    })
                };
                
                if value <= line_numbers.len() {
                    let return_loc = self.location;
                    self.goto_line(line_numbers[value - 1])?;
                    self.gosub_stack.push(return_loc);
                }
                Ok(())
            }
            Statement::Def { name, params, expr } => {
                self.internal_symbols.define_function(name.clone(), params.clone(), expr.clone())?;
                Ok(())
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<SymbolValue, BasicError> {
        match &expr.expr_type {
            ExpressionType::Number(n) => Ok(SymbolValue::Number(*n)),
            ExpressionType::String(s) => Ok(SymbolValue::String(s.clone())),
            ExpressionType::Variable(name) => self.get_symbol(name),

            ExpressionType::Array { name, indices } => {
                let idx_values: Result<Vec<usize>, BasicError> = indices.iter()
                    .map(|expr| match self.evaluate_expression(expr)? {
                        SymbolValue::Number(n) if n >= 0.0 => Ok(n as usize),
                        SymbolValue::Number(n) => Err(BasicError::Runtime {
                            message: "Array index must be non-negative".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        }),
                        _ => Err(BasicError::Runtime {
                            message: "Array index must be a number".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        })
                    })
                    .collect();

                let indices = idx_values?;
                self.symbols.get_array_element(name, &indices).map_err(|e| self.add_line_info_to_error(e))
            }

            ExpressionType::FunctionCall { name, args } => {
                // Check if this is a built-in function
                if FUNCTION_REGISTRY.is_function(name) {
                    let expected_types = FUNCTION_REGISTRY.get_arg_types(name).unwrap();
                    if expected_types.len() != args.len() {
                        return Err(BasicError::Runtime {
                            message: format!("Function '{}' expects {} arguments, got {}", name, expected_types.len(), args.len()),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        });
                    }
                    let mut evaluated_args = Vec::new();
                    for (arg, expected_type) in args.iter().zip(expected_types.iter()) {
                        let value = self.evaluate_expression(arg)?;
                        match (expected_type, value) {
                            (crate::basic_function_registry::ArgType::Number, SymbolValue::Number(n)) => {
                                evaluated_args.push(Token::new_number(&n.to_string()));
                            }
                            (crate::basic_function_registry::ArgType::String, SymbolValue::String(s)) => {
                                evaluated_args.push(Token::new_string(&s));
                            }
                            (crate::basic_function_registry::ArgType::Number, other) => {
                                return Err(BasicError::Runtime {
                                    message: format!("Function '{}' expects a number argument, got {:?}", name, other),
                                    basic_line_number: Some(self.get_current_line().line_number),
                                    file_line_number: None,
                                });
                            }
                            (crate::basic_function_registry::ArgType::String, other) => {
                                return Err(BasicError::Runtime {
                                    message: format!("Function '{}' expects a string argument, got {:?}", name, other),
                                    basic_line_number: Some(self.get_current_line().line_number),
                                    file_line_number: None,
                                });
                            }
                        }
                    }
                    let result = FUNCTION_REGISTRY.call_function_with_tokens(name, evaluated_args).map_err(|e| self.add_line_info_to_error(e))?;
                    match result {
                        Token::Number(n) => Ok(SymbolValue::Number(n.parse().unwrap_or(0.0))),
                        Token::String(s) => Ok(SymbolValue::String(s)),
                        _ => Err(BasicError::Runtime {
                            message: format!("Unexpected result type from function '{}'", name),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        }),
                    }
                } else {
                    // Check for user-defined functions (FNA, FNB, etc.)
                    if name.len() == 3 && name.starts_with("FN") && name.chars().nth(2).unwrap().is_ascii_uppercase() {
                        // User-defined function
                        let func_def = if let Some(SymbolValue::FunctionDef { param, expr }) = self.internal_symbols.get_symbol(name) {
                            Some((param.clone(), expr.clone()))
                        } else {
                            None
                        };

                        if let Some((param, expr)) = func_def {
                            let mut evaluated_args = Vec::new();
                            for arg in args {
                                let value = self.evaluate_expression(arg)?;
                                if let SymbolValue::Number(n) = value {
                                    evaluated_args.push(n);
                                } else {
                                    return Err(BasicError::Runtime {
                                        message: format!("User-defined function '{}' expects number arguments", name),
                                        basic_line_number: Some(self.get_current_line().line_number),
                                        file_line_number: None,
                                    });
                                }
                            }
                            
                            // Create a temporary scope with the function parameters
                            let nested_scope = self.symbols.get_nested_scope();
                            let original_symbols = std::mem::replace(&mut self.symbols, nested_scope);
                            
                            // Bind parameters to arguments
                            for (param_name, arg_value) in param.iter().zip(evaluated_args.iter()) {
                                self.symbols.put_symbol(param_name.clone(), SymbolValue::Number(*arg_value));
                            }
                            
                            // Evaluate the function body
                            let result = self.evaluate_expression(&expr)?;
                            
                            // Restore original symbol table
                            self.symbols = original_symbols;
                            
                            Ok(result)
                        } else {
                            Err(BasicError::Runtime {
                                message: format!("Undefined user function '{}'", name),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            })
                        }
                    } else {
                        Err(BasicError::Runtime {
                            message: format!("Unknown function '{}'", name),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        })
                    }
                }
            }

            ExpressionType::BinaryOp { op, left, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                match (left_val, right_val) {
                    (SymbolValue::Number(a), SymbolValue::Number(b)) => {
                        let result = match op.as_str() {
                            "+" => a + b,
                            "-" => a - b,
                            "*" => a * b,
                            "/" => {
                                if b == 0.0 {
                                    return Err(BasicError::Runtime {
                                        message: "Division by zero".to_string(),
                                        basic_line_number: Some(self.get_current_line().line_number),
                                        file_line_number: None,
                                    });
                                }
                                a / b
                            }
                            "^" => a.powf(b),
                            "=" => if a == b { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            "<>" => if a != b { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            "<" => if a < b { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            "<=" => if a <= b { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            ">" => if a > b { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            ">=" => if a >= b { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            "AND" => (a as i64 & b as i64) as f64,
                            "OR" => (a as i64 | b as i64) as f64,
                            _ => return Err(BasicError::Runtime {
                                message: format!("Unknown binary operator: {}", op),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            }),
                        };
                        Ok(SymbolValue::Number(result))
                    }
                    (SymbolValue::String(a), SymbolValue::String(b)) => {
                        let result = match op.as_str() {
                            "+" => Ok(SymbolValue::String(format!("{}{}", a, b))),
                            "<>" => Ok(SymbolValue::Number(if a != b { BASIC_TRUE_F } else { BASIC_FALSE_F })),
                            "=" => Ok(SymbolValue::Number(if a == b { BASIC_TRUE_F } else { BASIC_FALSE_F })),
                            _ => Err(BasicError::Runtime {
                                message: format!("Invalid operator '{}' for strings", op),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            }),
                        };
                        result
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("Type mismatch for operator '{}'", op),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    }),
                }
            }

            ExpressionType::UnaryOp { op, expr } => {
                let val = self.evaluate_expression(expr)?;
                
                match val {
                    SymbolValue::Number(n) => {
                        let result = match op.as_str() {
                            "-" => -n,
                            "NOT" => if n == BASIC_FALSE_F { BASIC_TRUE_F } else { BASIC_FALSE_F },
                            _ => return Err(BasicError::Runtime {
                                message: format!("Unknown unary operator: {}", op),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: None,
                            }),
                        };
                        Ok(SymbolValue::Number(result))
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("Invalid operand type for unary operator '{}'", op),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: None,
                    }),
                }
            }


        }
    }
    fn get_symbol(&self, name: &str) -> Result<SymbolValue, BasicError> {
        // Try current scope first, then parent scopes
        if let Some(value) = self.symbols.get_symbol(name) {
            Ok(value.clone())
        } else {
            // Try array with [] suffix
            let array_key = format!("{}[]", name);
            if let Some(value) = self.symbols.get_symbol(&array_key) {
                Ok(value.clone())
            } else if let Some(value) = self.internal_symbols.get_symbol(name) {
                Ok(value.clone())
            } else {
                Err(BasicError::Runtime {
                    message: format!("Undefined variable: {}", name),
                    basic_line_number: Some(self.get_current_line().line_number),
                    file_line_number: None,
                })
            }
        }
    }
    fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbols
    }

    fn put_symbol(&mut self, name: String, value: SymbolValue) {
        // In BASIC, scalar variables and arrays with the same name are separate entities
        // N and N() are different - this is legitimate BASIC behavior
        let name_copy=name.clone();
        self.symbols.put_symbol(name, value);
        if self.data_breakpoints.contains(&name_copy) {
            self.run_status = RunStatus::BreakData;
        }
    }

    fn goto_line(&mut self, line_number: usize) -> Result<(), BasicError> {
        if let Some(&index) = self.line_number_map.get(&line_number) {
            self.control_transfer(ControlLocation {
                index,
                offset: 0,
            });
            Ok(())
        } else {
            Err(BasicError::Runtime {
                message: format!("Line number {} not found", line_number),
                basic_line_number: Some(line_number),
                file_line_number: None,
            })
        }
    }

    pub fn get_current_line(&self) -> &ProgramLine {
        &self.program.lines[self.location.index]
    }

    pub fn get_coverage(&self) -> Option<&CoverageData> {
        self.coverage.as_ref()
    }
    
    /// Execute a single statement (for single-step debugging)
    pub fn step(&mut self) -> Result<(), BasicError> {
        // Allow stepping when at a breakpoint or normally running
        if self.run_status != RunStatus::Run && self.run_status != RunStatus::BreakCode && self.run_status != RunStatus::BreakData {
            return Ok(());
        }
        
        // If we're at a breakpoint, reset to running state for this step
        if self.run_status == RunStatus::BreakCode || self.run_status == RunStatus::BreakData {
            self.run_status = RunStatus::Run;
        }
        
        let current_line = self.get_current_line().line_number;
        let current_offset = self.location.offset;
        
        // Get current statement before any trace/coverage operations
        let current_stmt = self.get_current_stmt().clone();

        // Write trace
        self.do_trace(&current_stmt);

        // Update coverage before executing
        if let Some(ref mut cov) = self.coverage {
            cov.entry(current_line)
                .or_insert_with(HashSet::new)
                .insert(current_offset);
        }
        
        // Execute statement
        match self.execute_statement(&current_stmt) {
            Ok(()) => {
                self.advance_location();
                Ok(())
            }
            Err(err) => {
                self.run_status = match err {
                    BasicError::Syntax { .. } => RunStatus::EndErrorSyntax,
                    BasicError::Runtime { .. } => RunStatus::EndErrorRuntime,
                    BasicError::Internal { .. } => RunStatus::EndErrorInternal,
                    BasicError::Type { .. } => RunStatus::EndErrorType,
                };
                Err(err)
            }
        }
    }

    fn get_current_stmt(&self) -> &Statement {
        &self.get_current_line().statements[self.location.offset]
    }


    fn assign_lvalue(&mut self, expr: &Expression, value: SymbolValue) -> Result<(), BasicError> {
        match &expr.expr_type {
            ExpressionType::Variable(name) => {
                self.put_symbol(name.clone(), value);
                Ok(())
            }
            ExpressionType::Array { name, indices } => {
                let idx_values: Result<Vec<usize>, BasicError> = indices.iter()
                    .map(|expr| match self.evaluate_expression(expr)? {
                        SymbolValue::Number(n) if n >= 0.0 => Ok(n as usize),
                        SymbolValue::Number(n) => Err(BasicError::Runtime {
                            message: "Array index must be non-negative".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        }),
                        _ => Err(BasicError::Runtime {
                            message: "Array index must be a number".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: None,
                        })
                    })
                    .collect();
                let indices = idx_values?;
                self.symbols.set_array_element(name, &indices, value).map_err(|e| self.add_line_info_to_error(e))?;
                Ok(())
            }
            _ => Err(BasicError::Runtime {
                message: "Invalid lvalue in READ statement".to_string(),
                basic_line_number: Some(self.get_current_line().line_number),
                file_line_number: None,
            })
        }
    }

    fn goto_else_or_next_line(&mut self) -> Result<(), BasicError> {
        let current_line = self.get_current_line();
        let mut offset = self.location.offset + 1;
        
        // Look for ELSE statement on current line
        while offset < current_line.statements.len() {
            match &current_line.statements[offset] {
                Statement::Else => {
                    // Found ELSE, advance to it
                    self.control_transfer(ControlLocation{
                        index: self.location.index,
                        offset: offset,
                    });
                    self.advance_stmt = true;   // The above points to the else. We don't want
                                                // to execute the else, we want to start with
                                                // the statement after the else.
                    return Ok(());
                }
                _ => {
                    // Skip this statement
                    offset += 1;
                }
            }
        }
        
        // No ELSE found, go to next line. In this case, we DON'T want to advance after.
        self.goto_next_line();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::basic_lexer::Lexer;
    use crate::basic_parser::Parser;
    use super::*;
    use crate::basic_types::{Statement, Expression, ArrayDecl};

    fn create_test_program(lines: Vec<(usize, Vec<Statement>)>) -> Program {
        let mut program = Program::new();
        for (line_number, statements) in lines {
            program.add_line(line_number, format!("{}", line_number), statements);
        }
        program
    }

    #[test]
    fn test_line_number_execution() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![Statement::new_let(Expression::new_variable("X".to_string()), Expression::new_number(1.0))]),
            (20, vec![Statement::new_let(Expression::new_variable("Y".to_string()), Expression::new_number(2.0))]),
            (30, vec![Statement::new_let(Expression::new_variable("Z".to_string()), Expression::new_number(3.0))]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run()?;
        
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert_eq!(interpreter.get_symbol("Y")?, SymbolValue::Number(2.0));
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        
        Ok(())
    }

    #[test]
    fn test_goto_line() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![Statement::new_let(Expression::new_variable("X".to_string()), Expression::new_number(1.0))]),
            (20, vec![Statement::new_goto(40)]),
            (30, vec![Statement::new_let(Expression::new_variable("Y".to_string()), Expression::new_number(2.0))]),
            (40, vec![Statement::new_let(Expression::new_variable("Z".to_string()), Expression::new_number(3.0))]),
        ]);

        println!("Program has {} lines.", program.lines.len());
        println!("{}", program);
        let mut interpreter = Interpreter::new(program);
        interpreter.run()?;
        println!("SYMBOLLLLLLS");
        let symbols = interpreter.get_symbol_table();
        for (name, value) in symbols.dump() {
            println!("{} = {}", name, value);
        }
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert!(interpreter.get_symbol("Y").is_err()); // Line 30 should be skipped
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        
        Ok(())
    }

    #[test]
    fn test_rem_statement() -> Result<(), BasicError> {
        let source = "10 X=1\n20 REM This is a comment:Y=2\n30LET Z=3"; // TODO remove space before Z
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().expect("Lexing failed");
        // for token in &tokens {
        //     println!("T: {}", token);
        // }
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?; // ← You need this line to obtain the program
        let mut interpreter = Interpreter::new(program);
        interpreter.run()?;
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert!(interpreter.get_symbol("Y").is_err()); // Should be skipped after REM
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        Ok(())
    }

    #[test]
    fn test_invalid_line_number() {
        let program = create_test_program(vec![
            (10, vec![Statement::new_goto(25)]), // Line 25 doesn't exist
        ]);
        
        let mut interpreter = Interpreter::new(program);
        let result = interpreter.run();
        assert!(matches!(result, Err(BasicError::Runtime { .. })));
    }

    #[test]
    fn test_multiple_statements() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![
                Statement::new_let(Expression::new_variable("X".to_string()), Expression::new_number(1.0)),
                Statement::new_let(Expression::new_variable("Y".to_string()), Expression::new_number(2.0)),
                Statement::new_let(Expression::new_variable("Z".to_string()), Expression::new_number(3.0)),
            ]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run()?;
        
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert_eq!(interpreter.get_symbol("Y")?, SymbolValue::Number(2.0));
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        
        Ok(())
    }

    #[test]
    fn test_array_indexing() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![Statement::new_dim(vec![
                ArrayDecl { name: "A".to_string(), dimensions: vec![2, 5] }
            ])]),

            (20, vec![Statement::new_dim(vec![
                ArrayDecl { name: "B".to_string(), dimensions: vec![4] }
            ])]),

            (30, vec![Statement::new_dim(vec![
                ArrayDecl { name: "C$".to_string(), dimensions: vec![3] }
            ])]),
        ]);
        let mut interpreter = Interpreter::new(program);
        interpreter.run()?;

        // Test 2D numeric array (arrays stored with [] suffix)
        if let SymbolValue::Array2DNumber(arr) = interpreter.get_symbol("A[]")? {
            assert_eq!(arr.len(), 2);               // rows
            assert_eq!(arr[0].len(), 5);            // columns
        } else {
            panic!("Expected 2D numeric array 'A'");
        }

        // Test 1D numeric array (arrays stored with [] suffix)
        if let SymbolValue::Array1DNumber(arr) = interpreter.get_symbol("B[]")? {
            assert_eq!(arr.len(), 4);
        } else {
            panic!("Expected 1D numeric array 'B'");
        }

        // Test 1D string array (arrays stored with [] suffix)
        if let SymbolValue::Array1DString(arr) = interpreter.get_symbol("C$[]")? {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected 1D string array 'C$'");
        }

        Ok(())
    }
}
