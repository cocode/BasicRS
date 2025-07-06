use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use rand::prelude::*;
// use crate::basic_symbols::SymbolTable;

use crate::basic_types::{
    Program, ProgramLine, Statement, Expression, BasicError,
    ExpressionType, RunStatus, SymbolValue,
    Token,
};

use crate::basic_symbols:: {
    SymbolTable
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
}

impl Interpreter {
    pub fn new(program: Program) -> Self {
        // Build line number map
        let mut line_number_map = HashMap::new();
        for (i, line) in program.lines.iter().enumerate() {
            line_number_map.insert(line.line_number, i);
        }
        
        let mut internal_symbols = SymbolTable::new();
        let symbols = internal_symbols.get_nested_scope();
        
        Interpreter {
            program,
            location: ControlLocation { index: 0, offset: 0 },
            internal_symbols,
            symbols,
            for_stack: Vec::new(),
            gosub_stack: Vec::new(),
            data_pointer: 0,
            data_values: Vec::new(),
            run_status: RunStatus::Run,
            trace_file: None,
            coverage: None,
            breakpoints: HashSet::new(),
            data_breakpoints: HashSet::new(),
            line_number_map,
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
        self.run_status.clone()
    }

    pub fn set_run_status(&mut self, status: RunStatus) {
        self.run_status = status;
    }

    pub fn run(&mut self) -> RunStatus {
        while self.run_status == RunStatus::Run {
            // Get all the values we need before any mutable operations
            let current_line = self.get_current_line_number();
            let current_offset = self.location.offset;
            let current_stmt = self.get_current_stmt().clone();
            let current_source = self.get_current_line().source.clone();
            
            // Check breakpoints
            if self.breakpoints.contains(&(current_line, current_offset)) {
                self.run_status = RunStatus::BreakCode;
                return self.run_status.clone();
            }
            
            // Write trace
            if let Some(ref mut file) = self.trace_file {
                if self.location.offset == 0 {
                    writeln!(file, ">{}", current_source).ok();
                }
                writeln!(file, "\t{:?}", &current_stmt).ok();
            }
            
            // Update coverage before executing
            if let Some(ref mut cov) = self.coverage {
                cov.entry(current_line)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
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
                    return self.run_status.clone();
                }
            }
        }
        
        self.run_status.clone()
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<(), BasicError> {
        match stmt {
            Statement::Let { var, expr } => {
                let result = self.evaluate_expression(expr)?;
                self.put_symbol(var.clone(), result);
                Ok(())
            }
            Statement::Print { exprs } => {
                for (i, expr) in exprs.iter().enumerate() {
                    let value = self.evaluate_expression(expr)?;
                    print!("{}", value);
                    if i < exprs.len() - 1 {
                        print!(" ");
                    }
                }
                println!();
                Ok(())
            }
            Statement::Input { var } => {
                let mut input = String::new();
                print!("? ");
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
            Statement::If { condition, then_stmt, else_stmt } => {
                let result = self.evaluate_expression(condition)?;
                match result {
                    SymbolValue::Number(n) => {
                        if n != 0.0 {
                            self.execute_statement(then_stmt)?;
                        } else if let Some(else_s) = else_stmt {
                            self.execute_statement(else_s)?;
                        }
                    }
                    _ => return Err(BasicError::Type {
                        message: "IF condition must evaluate to a number".to_string(),
                        line_number: None,
                    }),
                }
                Ok(())
            }
            Statement::For { var, start, stop, step } => {
                let start_value = self.evaluate_expression(start)?;
                let stop_expr = stop.clone();
                let step_expr = step.clone().unwrap_or_else(|| Expression {
                    expr_type: ExpressionType::Number(1.0),
                    line_number: None,
                });
                
                // Get numeric values
                let current = match start_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop start value must be a number".to_string(),
                        line_number: None,
                    }),
                };
                
                let stop_value = self.evaluate_expression(&stop_expr)?;
                let stop = match stop_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop stop value must be a number".to_string(),
                        line_number: None,
                    }),
                };
                
                let step_value = self.evaluate_expression(&step_expr)?;
                let step = match step_value {
                    SymbolValue::Number(n) => n,
                    _ => return Err(BasicError::Runtime {
                        message: "FOR loop step must be a number".to_string(),
                        line_number: None,
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
                            line_number: None,
                        }),
                    };

                    // Get step value
                    let step_value = self.evaluate_expression(&for_record.step)?;
                    let step = match step_value {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop step must be numeric".to_string(),
                            line_number: None,
                        }),
                    };

                    // Get stop value
                    let stop_value = self.evaluate_expression(&for_record.stop)?;
                    let stop = match stop_value {
                        SymbolValue::Number(n) => n,
                        _ => return Err(BasicError::Runtime {
                            message: "FOR loop stop value must be numeric".to_string(),
                            line_number: None,
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
                        line_number: None,
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
                    self.advance_location();
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "RETURN without GOSUB".to_string(),
                        line_number: None,
                    })
                }
            }
            Statement::End => {
                self.run_status = RunStatus::EndNormal;
                Ok(())
            }
            Statement::Stop => {
                self.run_status = RunStatus::Stop;
                Ok(())
            }
            Statement::Rem { .. } => Ok(()),
            Statement::Data { values } => {
                // Evaluate each expression to get SymbolValues
                let mut evaluated_values = Vec::new();
                for expr in values {
                    let value = self.evaluate_expression(expr)?;
                    evaluated_values.push(value);
                }
                self.data_values.extend(evaluated_values);
                Ok(())
            }
            Statement::Read { vars } => {
                for var in vars {
                    if self.data_pointer >= self.data_values.len() {
                        return Err(BasicError::Runtime {
                            message: "Out of DATA values".to_string(),
                            line_number: None,
                        });
                    }
                    let value = self.data_values[self.data_pointer].clone();
                    self.put_symbol(var.clone(), value);
                    self.data_pointer += 1;
                }
                Ok(())
            }
            Statement::Restore => {
                self.data_pointer = 0;
                Ok(())
            }
            Statement::Dim { arrays } => {
                for array in arrays {
                    if array.dimensions.is_empty() {
                        return Err(BasicError::Runtime {
                            message: format!("Array '{}' requires at least one dimension", array.name),
                            line_number: None,
                        });
                    }
                    // Calculate total size
                    let total_size: usize = array.dimensions.iter().product();
                    // Create array with default values
                    let values = vec![SymbolValue::Number(0.0); total_size];
                    self.symbols.put_symbol(array.name.clone(), SymbolValue::Array(values));
                }
                Ok(())
            }
            Statement::OnGoto { expr, line_numbers } => {
                let value = match self.evaluate_expression(expr)? {
                    SymbolValue::Number(n) if n >= 1.0 && n.fract() == 0.0 => n as usize,
                    _ => return Err(BasicError::Runtime {
                        message: "ON index must be a positive integer".to_string(),
                        line_number: None,
                    })
                };
                
                if value <= line_numbers.len() {
                    self.goto_line(line_numbers[value - 1])?;
                }
                Ok(())
            }
            Statement::Def { name, params, expr } => {
                // Store function definition as a Function symbol value
                self.internal_symbols.put_symbol(name.clone(), SymbolValue::Function(Box::new(expr.clone())));
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
                    .map(|idx| {
                        match self.evaluate_expression(idx)? {
                            SymbolValue::Number(n) => Ok(n as usize),
                            _ => Err(BasicError::Type {
                                message: "Array index must be a number".to_string(),
                                line_number: None,
                            }),
                        }
                    })
                    .collect();
                
                let indices = idx_values?;
                match self.get_symbol(name)? {
                    SymbolValue::Array(values) => {
                        // Calculate linear index from multi-dimensional indices
                        let mut linear_index = 0;
                        let mut multiplier = 1;
                        for &idx in indices.iter().rev() {
                            linear_index += idx * multiplier;
                            multiplier *= values.len();
                        }
                        
                        if linear_index >= values.len() {
                            return Err(BasicError::Runtime {
                                message: format!("Array index out of bounds: {}", linear_index),
                                line_number: None,
                            });
                        }
                        
                        Ok(values[linear_index].clone())
                    },
                    _ => Err(BasicError::Type {
                        message: format!("{} is not an array", name),
                        line_number: None,
                    }),
                }
            },
            ExpressionType::BinaryOp { op, left, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                
                // Handle binary operations
                match op.as_str() {
                    "+" => match (left_val, right_val) {
                        (SymbolValue::Number(a), SymbolValue::Number(b)) => Ok(SymbolValue::Number(a + b)),
                        (SymbolValue::String(a), SymbolValue::String(b)) => Ok(SymbolValue::String(format!("{}{}", a, b))),
                        _ => Err(BasicError::Type {
                            message: "Type mismatch in addition".to_string(),
                            line_number: None,
                        }),
                    },
                    "-" => match (left_val, right_val) {
                        (SymbolValue::Number(a), SymbolValue::Number(b)) => Ok(SymbolValue::Number(a - b)),
                        _ => Err(BasicError::Type {
                            message: "Can only subtract numbers".to_string(),
                            line_number: None,
                        }),
                    },
                    "*" => match (left_val, right_val) {
                        (SymbolValue::Number(a), SymbolValue::Number(b)) => Ok(SymbolValue::Number(a * b)),
                        _ => Err(BasicError::Type {
                            message: "Can only multiply numbers".to_string(),
                            line_number: None,
                        }),
                    },
                    "/" => match (left_val, right_val) {
                        (SymbolValue::Number(a), SymbolValue::Number(b)) => {
                            if b == 0.0 {
                                Err(BasicError::Runtime {
                                    message: "Division by zero".to_string(),
                                    line_number: None,
                                })
                            } else {
                                Ok(SymbolValue::Number(a / b))
                            }
                        },
                        _ => Err(BasicError::Type {
                            message: "Can only divide numbers".to_string(),
                            line_number: None,
                        }),
                    },
                    "^" => match (left_val, right_val) {
                        (SymbolValue::Number(a), SymbolValue::Number(b)) => Ok(SymbolValue::Number(a.powf(b))),
                        _ => Err(BasicError::Type {
                            message: "Can only raise numbers to powers".to_string(),
                            line_number: None,
                        }),
                    },
                    _ => Err(BasicError::Internal {
                        message: format!("Unknown binary operator: {}", op),
                    }),
                }
            },
            ExpressionType::UnaryOp { op, expr } => {
                let val = self.evaluate_expression(expr)?;
                match op.as_str() {
                    "-" => match val {
                        SymbolValue::Number(n) => Ok(SymbolValue::Number(-n)),
                        _ => Err(BasicError::Type {
                            message: "Can only negate numbers".to_string(),
                            line_number: None,
                        }),
                    },
                    "NOT" => match val {
                        SymbolValue::Number(n) => Ok(SymbolValue::Number(if n == 0.0 { -1.0 } else { 0.0 })),
                        _ => Err(BasicError::Type {
                            message: "Can only apply NOT to numbers".to_string(),
                            line_number: None,
                        }),
                    },
                    _ => Err(BasicError::Internal {
                        message: format!("Unknown unary operator: {}", op),
                    }),
                }
            },
            ExpressionType::FunctionCall { name, args } => {
                // Evaluate all arguments
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                
                // Convert arguments to strings for predefined functions
                let string_args: Vec<String> = evaluated_args.iter().map(|arg| {
                    match arg {
                        SymbolValue::Number(n) => n.to_string(),
                        SymbolValue::String(s) => s.clone(),
                        _ => "0".to_string(), // Default for unsupported types
                    }
                }).collect();
                
                // Convert to slice of string slices for function call
                let str_slices: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();
                
                // Create predefined functions instance
                let predef = PredefinedFunctions::new();
                
                // Try calling as predefined function
                if predef.functions().contains(&name.to_string()) {
                    if let Some(result) = predef.call(&name.to_string(), &str_slices) {
                        // Try to parse result as number first
                        if let Ok(n) = result.parse::<f64>() {
                            Ok(SymbolValue::Number(n))
                        } else {
                            Ok(SymbolValue::String(result))
                        }
                    } else {
                        Err(BasicError::Runtime {
                            message: format!("Error calling function: {}", name),
                            line_number: None,
                        })
                    }
                } else {
                    // Must be a user-defined function
                    match self.internal_symbols.get_symbol(name) {
                        Some(SymbolValue::Function(func)) => {
                            // TODO: Implement user-defined function calls
                            Err(BasicError::Internal {
                                message: "User-defined functions not yet implemented".to_string(),
                            })
                        },
                        _ => Err(BasicError::Runtime {
                            message: format!("Unknown function: {}", name),
                            line_number: None,
                        }),
                    }
                }
            },
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
                line_number: None,
            })
        }
    }

    fn put_symbol(&mut self, name: String, value: SymbolValue) {
        // Check if this is a data breakpoint
        if self.data_breakpoints.contains(&name) {
            self.run_status = RunStatus::BreakData;
        }
        self.symbols.put_symbol(name.clone(), value);
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
                line_number: None,
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
                self.run_status = RunStatus::EndNormal;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{Token, Statement, Expression, ExpressionType};

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
            (10, vec![Statement::Let {
                var: "X".to_string(),
                expr: Expression {
                    expr_type: ExpressionType::Number(1.0),
                    line_number: None,
                }
            }]),
            (20, vec![Statement::Let {
                var: "Y".to_string(),
                expr: Expression {
                    expr_type: ExpressionType::Number(2.0),
                    line_number: None,
                }
            }]),
            (30, vec![Statement::Let {
                var: "Z".to_string(),
                expr: Expression {
                    expr_type: ExpressionType::Number(3.0),
                    line_number: None,
                }
            }]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run();
        
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert_eq!(interpreter.get_symbol("Y")?, SymbolValue::Number(2.0));
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        
        Ok(())
    }

    #[test]
    fn test_goto_line() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![Statement::new_let("X".to_string(), Expression {
                expr_type: ExpressionType::Number(1.0),
                line_number: None,
            })]),
            (20, vec![Statement::new_goto(40)]),
            (30, vec![Statement::new_let("Y".to_string(), Expression {
                expr_type: ExpressionType::Number(2.0),
                line_number: None,
            })]),
            (40, vec![Statement::new_let("Z".to_string(), Expression {
                expr_type: ExpressionType::Number(3.0),
                line_number: None,
            })]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run();
        
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert!(interpreter.get_symbol("Y").is_err()); // Line 30 should be skipped
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        
        Ok(())
    }

    #[test]
    fn test_rem_statement() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![Statement::new_let("X".to_string(), Expression {
                expr_type: ExpressionType::Number(1.0),
                line_number: None,
            })]),
            (20, vec![
                Statement::new_rem("This is a comment".to_string()),
                Statement::new_let("Y".to_string(), Expression {
                    expr_type: ExpressionType::Number(2.0),
                    line_number: None,
                }), // Should be ignored
            ]),
            (30, vec![Statement::new_let("Z".to_string(), Expression {
                expr_type: ExpressionType::Number(3.0),
                line_number: None,
            })]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run();
        
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
        let status = interpreter.run();
        
        assert_eq!(status, RunStatus::EndErrorRuntime);
    }

    #[test]
    fn test_multiple_statements() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![
                Statement::new_let("X".to_string(), Expression {
                    expr_type: ExpressionType::Number(1.0),
                    line_number: None,
                }),
                Statement::new_let("Y".to_string(), Expression {
                    expr_type: ExpressionType::Number(2.0),
                    line_number: None,
                }),
                Statement::new_let("Z".to_string(), Expression {
                    expr_type: ExpressionType::Number(3.0),
                    line_number: None,
                }),
            ]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run();
        
        assert_eq!(interpreter.get_symbol("X")?, SymbolValue::Number(1.0));
        assert_eq!(interpreter.get_symbol("Y")?, SymbolValue::Number(2.0));
        assert_eq!(interpreter.get_symbol("Z")?, SymbolValue::Number(3.0));
        
        Ok(())
    }

    #[test]
    fn test_array_indexing() -> Result<(), BasicError> {
        let program = create_test_program(vec![
            (10, vec![Statement::new_dim("A".to_string(), vec![3, 3])]),
            (20, vec![Statement::new_let(
                "A".to_string(),
                Expression {
                    expr_type: ExpressionType::Array {
                        name: "A".to_string(),
                        indices: vec![
                            Expression {
                                expr_type: ExpressionType::Number(1.0),
                                line_number: None,
                            },
                            Expression {
                                expr_type: ExpressionType::Number(1.0),
                                line_number: None,
                            },
                        ],
                    },
                    line_number: None,
                },
            )]),
        ]);
        
        let mut interpreter = Interpreter::new(program);
        interpreter.run();
        
        // Test array access
        if let SymbolValue::Array(arr) = interpreter.get_symbol("A")? {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0].len(), 3);
        } else {
            panic!("Expected array");
        }
        
        Ok(())
    }
} 
