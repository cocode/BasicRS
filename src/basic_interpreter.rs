use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use crate::basic_symbols::SymbolTable;

use crate::basic_types::{
    Program, ProgramLine, Statement, Expression, BasicError,
    ExpressionType, RunStatus, SymbolValue, Token,
};

use crate::basic_functions::PredefinedFunctions;

const TRACE_FILE_NAME: &str = "basic_trace.txt";

// Control location in program
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlLocation {
    pub index: usize,    // Index into program lines
    pub offset: usize,   // Offset into statements in the line
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
    run_status: RunStatus,
    trace_file: Option<File>,
    coverage: Option<HashMap<usize, usize>>,
    breakpoints: HashSet<(usize, usize)>,
    data_breakpoints: HashSet<String>,
    line_number_map: HashMap<usize, usize>, // Maps line numbers to program indices
    file_line_number: usize,    // Current file line number for error reporting. We don't
                                // track this currently. I think we'd need to add it to ProgramLine
}

impl Interpreter {
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
            run_status: RunStatus::Run,
            trace_file: None,
            coverage: None,
            breakpoints: HashSet::new(),
            data_breakpoints: HashSet::new(),
            line_number_map,
            file_line_number: 1,
        }
    }

    pub fn enable_trace(&mut self) -> io::Result<()> {
        self.trace_file = Some(File::create(TRACE_FILE_NAME)?);
        Ok(())
    }

    pub fn enable_coverage(&mut self) {
        self.coverage = Some(HashMap::new());
    }

    pub fn add_breakpoint(&mut self, line: usize, offset: usize) {
        self.breakpoints.insert((line, offset));
    }

    pub fn add_data_breakpoint(&mut self, var: String) {
        self.data_breakpoints.insert(var);
    }

    pub fn get_coverage(&self) -> Option<&HashMap<usize, usize>> {
        self.coverage.as_ref()
    }

    pub fn get_symbol_value(&self, name: &str) -> Option<&SymbolValue> {
        self.symbols.get_symbol(name)
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
        for pl in &self.program.lines {
            for stmt in &pl.statements {
                if let Statement::Data { values } = stmt {
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
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
            
            // Execute statement
            match self.execute_statement(&current_stmt) {
                Ok(()) => {
                    if current_stmt.should_advance_location() {
                        self.advance_location();
                    }
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
        let current_line_number = self.get_current_line().source.clone();
        if let Some(ref mut file) = self.trace_file {
            if self.location.offset == 0 {
                writeln!(file, ">{}", current_line_number).ok();
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
                                SymbolValue::Number(n) if n >= 0.0 && n.fract() == 0.0 => Ok(n as usize),
                                _ => Err(BasicError::Runtime {
                                    message: "Array index must be a non-negative integer".to_string(),
                                    basic_line_number: Some(self.get_current_line().line_number),
                                    file_line_number: Some(self.file_line_number),
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
                                    *file_line_number = Some(self.file_line_number);
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
                                file_line_number: Some(self.file_line_number),
                            })
                        }
                    }
                }
            }
            Statement::Print { expressions } => {
                for (i, expr) in expressions.iter().enumerate() {
                    let value = self.evaluate_expression(expr)?;
                    print!("{}", value);
                    if i < expressions.len() - 1 {
                        print!(" ");
                    }
                }
                println!();
                Ok(())
            }
            Statement::Input { var, prompt } => {
                let mut input = String::new();
                if let Some(p) = prompt {
                    print!("{}", p);
                } else {
                    print!("? ");
                }
                io::stdout().flush()?;
                io::stdin().read_line(&mut input)?;
                let value = if let Ok(n) = input.trim().parse::<f64>() {
                    SymbolValue::Number(n)
                } else {
                    SymbolValue::String(input.trim().to_string())
                };
                self.put_symbol(var.clone(), value);
                Ok(())
            }
            Statement::If { condition } => {
                let result = self.evaluate_expression(condition)?;
                match result {
                    SymbolValue::Number(n) => {
                        if n == 0.0 {
                            // Condition is false, skip to ELSE or next line
                            self.goto_else_or_next_line()?;
                        }
                        // If condition is true, continue to next statement
                    }
                    _ => return Err(BasicError::Type {
                        message: "IF condition must evaluate to a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number), // TODO point to correct statement on multi-statement line
                        file_line_number: Some(self.file_line_number),
                    }),
                }
                Ok(())
            }
            Statement::Then => {
                Ok(())
            }
            Statement::Else => {
                Ok(())
            }
            Statement::For { var, start, stop, step } => {
                let start_value = self.evaluate_expression(start)?;
                let stop_expr = stop.clone();
                let step_expr = step.clone().unwrap_or_else(|| Expression::new_number(1.0));
                
                // Get numeric values
                let current = match start_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop start value must be a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
                    }),
                };
                
                let stop_value = self.evaluate_expression(&stop_expr)?;
                let stop = match stop_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop stop value must be a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
                    }),
                };
                
                let step_value = self.evaluate_expression(&step_expr)?;
                let step = match step_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop step must be a number".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
                    }),
                };
                
                self.for_stack.push(ForRecord {
                    var: var.clone(),
                    stop: stop_expr,
                    step: step_expr,
                    stmt: Some(self.location),
                });
                
                self.put_symbol(var.clone(), SymbolValue::Number(current));
                
                if (step >= 0.0 && current > stop) || (step < 0.0 && current < stop) {
                    if let Some(stmt_loc) = self.for_stack.last().unwrap().stmt {
                        self.location = stmt_loc;
                    }
                    self.for_stack.pop();
                }
                Ok(())
            }
            Statement::Next { var } => {
                if let Some(for_record) = self.for_stack.last().cloned() {
                    // Get current value
                    let current_value = self.get_symbol(var)?;
                    let current = match current_value {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop variable must be numeric".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: Some(self.file_line_number),
                        }),
                    };

                    // Get step value
                    let step_value = self.evaluate_expression(&for_record.step)?;
                    let step = match step_value {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop step must be numeric".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: Some(self.file_line_number),
                        }),
                    };

                    // Get stop value
                    let stop_value = self.evaluate_expression(&for_record.stop)?;
                    let stop = match stop_value {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop stop value must be numeric".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: Some(self.file_line_number),
                        }),
                    };
                    
                    let next_value = current + step;
                    
                    if (step >= 0.0 && next_value <= stop) || (step < 0.0 && next_value >= stop) {
                        self.put_symbol(var.clone(), SymbolValue::Number(next_value));
                        if let Some(stmt_loc) = for_record.stmt {
                            self.location = stmt_loc;
                            self.for_stack.pop();
                            self.for_stack.push(for_record);
                        }
                    } else {
                        self.for_stack.pop();
                    }
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "NEXT without matching FOR".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
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
                    self.location = return_loc;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "RETURN without GOSUB".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
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
                            file_line_number: Some(self.file_line_number),
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
                                    SymbolValue::Number(n) if n >= 0.0 && n.fract() == 0.0 => Ok(n as usize),
                                    _ => Err(BasicError::Runtime {
                                        message: "Array index must be a non-negative integer".to_string(),
                                        basic_line_number: Some(self.get_current_line().line_number),
                                        file_line_number: Some(self.file_line_number),
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
                                file_line_number: Some(self.file_line_number),
                            });
                        }
                    }
                }
                Ok(())
            }
            Statement::Restore { line } => {
                if let Some(line_num) = line {
                    // Find the line number and reset data pointer
                    // For now, just reset to beginning
                    self.data_pointer = 0;
                } else {
                    // Reset to beginning
                    self.data_pointer = 0;
                }
                Ok(())
            }
            Statement::Dim { arrays } => {
                for array in arrays {
                    self.symbols.create_array(array.name.clone(), array.dimensions.clone())?;
                }
                Ok(())
            }
            Statement::OnGoto { expr, line_numbers } => {
                let value = match self.evaluate_expression(expr)? {
                    SymbolValue::Number(n) if n >= 1.0 && n.fract() == 0.0 => n as usize,
                    _ => return Err(BasicError::Runtime {
                        message: "ON index must be a positive integer".to_string(),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
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
                        file_line_number: Some(self.file_line_number),
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
                        SymbolValue::Number(n) if n >= 0.0 && n.fract() == 0.0 => Ok(n as usize),
                        _ => Err(BasicError::Runtime {
                            message: "Array index must be a non-negative integer".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: Some(self.file_line_number),
                        })
                    })
                    .collect();

                let indices = idx_values?;
                self.symbols.get_array_element(name, &indices)
            }

            ExpressionType::FunctionCall { name, args } => {
                // Check if this is a function that uses the new function system
                if let Some(function) = crate::basic_functions::get_function(name) {
                    let expected_types = function.arg_types();
                    if expected_types.len() != args.len() {
                        return Err(BasicError::Runtime {
                            message: format!("Function '{}' expects {} arguments, got {}", name, expected_types.len(), args.len()),
                            basic_line_number: None,
                            file_line_number: Some(self.file_line_number),
                        });
                    }
                    let mut evaluated_args = Vec::new();
                    for (arg, expected_type) in args.iter().zip(expected_types.iter()) {
                        let value = self.evaluate_expression(arg)?;
                        match (expected_type, value) {
                            (crate::basic_functions::ArgType::Number, SymbolValue::Number(n)) => {
                                evaluated_args.push(Token::new_number(&n.to_string()));
                            }
                            (crate::basic_functions::ArgType::String, SymbolValue::String(s)) => {
                                evaluated_args.push(Token::new_string(&s));
                            }
                            (crate::basic_functions::ArgType::Number, other) => {
                                return Err(BasicError::Runtime {
                                    message: format!("Function '{}' expects a number argument, got {:?}", name, other),
                                    basic_line_number: None,
                                    file_line_number: Some(self.file_line_number),
                                });
                            }
                            (crate::basic_functions::ArgType::String, other) => {
                                return Err(BasicError::Runtime {
                                    message: format!("Function '{}' expects a string argument, got {:?}", name, other),
                                    basic_line_number: None,
                                    file_line_number: Some(self.file_line_number),
                                });
                            }
                        }
                    }
                    let result = function.call(evaluated_args)?;
                    match result {
                        Token::Number(n) => Ok(SymbolValue::Number(n.parse().unwrap_or(0.0))),
                        Token::String(s) => Ok(SymbolValue::String(s)),
                        _ => Err(BasicError::Runtime {
                            message: format!("Unexpected result type from function '{}'", name),
                            basic_line_number: None,
                            file_line_number: Some(self.file_line_number),
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
                                        basic_line_number: None,
                                        file_line_number: Some(self.file_line_number),
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
                                basic_line_number: None,
                                file_line_number: Some(self.file_line_number),
                            })
                        }
                    } else {
                        // Fall back to old function system for math functions
                        let mut evaluated_args = Vec::new();
                        for arg in args {
                            let value = self.evaluate_expression(arg)?;
                            if let SymbolValue::Number(n) = value {
                                evaluated_args.push(n);
                            } else {
                                return Err(BasicError::Runtime {
                                    message: format!("Invalid argument for function '{}'", name),
                                    basic_line_number: None,
                                    file_line_number: Some(self.file_line_number),
                                });
                            }
                        }

                        let funcs = PredefinedFunctions::new();

                        if let Some(result) = funcs.call(name, &evaluated_args) {
                            Ok(SymbolValue::Number(result))
                        } else {
                            Err(BasicError::Runtime {
                                message: format!("Unknown function '{}'", name),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: Some(self.file_line_number),
                            })
                        }
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
                                        file_line_number: Some(self.file_line_number),
                                    });
                                }
                                a / b
                            }
                            "^" => a.powf(b),
                            "=" => if a == b { -1.0 } else { 0.0 },
                            "<>" => if a != b { -1.0 } else { 0.0 },
                            "<" => if a < b { -1.0 } else { 0.0 },
                            "<=" => if a <= b { -1.0 } else { 0.0 },
                            ">" => if a > b { -1.0 } else { 0.0 },
                            ">=" => if a >= b { -1.0 } else { 0.0 },
                            "AND" => (a as i64 & b as i64) as f64,
                            "OR" => (a as i64 | b as i64) as f64,
                            _ => return Err(BasicError::Runtime {
                                message: format!("Unknown binary operator: {}", op),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: Some(self.file_line_number),
                            }),
                        };
                        Ok(SymbolValue::Number(result))
                    }
                    (SymbolValue::String(a), SymbolValue::String(b)) => {
                        let result = match op.as_str() {
                            "+" => Ok(SymbolValue::String(format!("{}{}", a, b))),
                            "<>" => Ok(SymbolValue::Number(if a != b { -1.0 } else { 0.0 })),
                            "=" => Ok(SymbolValue::Number(if a == b { -1.0 } else { 0.0 })),
                            _ => Err(BasicError::Runtime {
                                message: format!("Invalid operator '{}' for strings", op),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: Some(self.file_line_number),
                            }),
                        };
                        result
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("Type mismatch for operator '{}'", op),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
                    }),
                }
            }

            ExpressionType::UnaryOp { op, expr } => {
                let val = self.evaluate_expression(expr)?;
                
                match val {
                    SymbolValue::Number(n) => {
                        let result = match op.as_str() {
                            "-" => -n,
                            "NOT" => if n == 0.0 { -1.0 } else { 0.0 },
                            _ => return Err(BasicError::Runtime {
                                message: format!("Unknown unary operator: {}", op),
                                basic_line_number: Some(self.get_current_line().line_number),
                                file_line_number: Some(self.file_line_number),
                            }),
                        };
                        Ok(SymbolValue::Number(result))
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("Invalid operand type for unary operator '{}'", op),
                        basic_line_number: Some(self.get_current_line().line_number),
                        file_line_number: Some(self.file_line_number),
                    }),
                }
            }


        }
    }
    fn get_symbol(&self, name: &str) -> Result<SymbolValue, BasicError> {
        // Try current scope first, then parent scopes
        if let Some(value) = self.symbols.get_symbol(name) {
            Ok(value.clone())
        } else if let Some(value) = self.internal_symbols.get_symbol(name) {
            Ok(value.clone())
        } else {
            Err(BasicError::Runtime {
                message: format!("Undefined variable: {}", name),
                basic_line_number: Some(self.get_current_line().line_number),
                file_line_number: Some(self.file_line_number),
            })
        }
    }
    fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbols
    }

    fn put_symbol(&mut self, name: String, value: SymbolValue) {
        // Always put in current scope
        let name_copy=name.clone();
        self.symbols.put_symbol(name, value);
        if self.data_breakpoints.contains(&name_copy) {
            self.run_status = RunStatus::BreakData;
        }
    }

    fn goto_line(&mut self, line_number: usize) -> Result<(), BasicError> {
        if let Some(&index) = self.line_number_map.get(&line_number) {
            self.location = ControlLocation {
                index,
                offset: 0,
            };
            Ok(())
        } else {
            Err(BasicError::Runtime {
                message: format!("Line number {} not found", line_number),
                basic_line_number: Some(line_number),
                file_line_number: Some(self.file_line_number),
            })
        }
    }

    fn get_current_line(&self) -> &ProgramLine {
        &self.program.lines[self.location.index]
    }

    fn get_current_stmt(&self) -> &Statement {
        &self.get_current_line().statements[self.location.offset]
    }

    fn advance_location(&mut self) {
        let current_line = self.get_current_line();
        if self.location.offset + 1 < current_line.statements.len() {
            // Move to next statement in current line
            self.location.offset += 1;
        } else {
            // Move to first statement of next line
            if self.location.index + 1 < self.program.lines.len() {
                self.location.index += 1;
                self.location.offset = 0;
            } else {
                self.run_status = RunStatus::EndOfProgram;
            }
        }
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
                        SymbolValue::Number(n) if n >= 0.0 && n.fract() == 0.0 => Ok(n as usize),
                        _ => Err(BasicError::Runtime {
                            message: "Array index must be a non-negative integer".to_string(),
                            basic_line_number: Some(self.get_current_line().line_number),
                            file_line_number: Some(self.file_line_number),
                        })
                    })
                    .collect();
                let indices = idx_values?;
                self.symbols.set_array_element(name, &indices, value)?;
                Ok(())
            }
            _ => Err(BasicError::Runtime {
                message: "Invalid lvalue in READ statement".to_string(),
                basic_line_number: Some(self.get_current_line().line_number),
                file_line_number: Some(self.file_line_number),
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
                    self.location.offset = offset;
                    return Ok(());
                }
                _ => {
                    // Skip this statement
                    offset += 1;
                }
            }
        }
        
        // No ELSE found, go to next line
        self.goto_next_line();
        Ok(())
    }

    fn goto_next_line(&mut self) {
        if self.location.index + 1 < self.program.lines.len() {
            self.location.index += 1;
            self.location.offset = 0;
        } else {
            self.run_status = RunStatus::EndOfProgram;
        }
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

        // Test 2D numeric array
        if let SymbolValue::Array2DNumber(arr) = interpreter.get_symbol("A")? {
            assert_eq!(arr.len(), 2);               // rows
            assert_eq!(arr[0].len(), 5);            // columns
        } else {
            panic!("Expected 2D numeric array 'A'");
        }

        // Test 1D numeric array
        if let SymbolValue::Array1DNumber(arr) = interpreter.get_symbol("B")? {
            assert_eq!(arr.len(), 4);
        } else {
            panic!("Expected 1D numeric array 'B'");
        }

        // Test 1D string array
        if let SymbolValue::Array1DString(arr) = interpreter.get_symbol("C$")? {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected 1D string array 'C$'");
        }

        Ok(())
    }
}
