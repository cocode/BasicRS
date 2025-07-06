use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Write};
use rand::prelude::*;
// use crate::basic_symbols::SymbolTable;

use crate::basic_types::{
    Program, ProgramLine, Statement, Expression, BasicError,
    ExpressionType, RunStatus, SymbolValue,
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
    internal_symbols: SymbolTable,  // Internal symbol table with nested scopes
    symbols: SymbolTable,           // Public symbol table
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
        
        let mut interpreter = Interpreter {
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
        };
        
        interpreter
    }

    pub fn run(&mut self) -> RunStatus {
        while self.run_status == RunStatus::Run {
            let current = self.get_current_line();
            
            // Check breakpoints
            for (line, offset) in &self.breakpoints {
                if current.line_number == *line && self.location.offset == *offset {
                    self.run_status = RunStatus::BreakCode;
                    return self.run_status;
                }
            }
            
            // Write trace
            if let Some(ref mut file) = self.trace_file {
                if self.location.offset == 0 {
                    writeln!(file, ">{}", current.source).ok();
                }
                writeln!(file, "\t{:?}", self.get_current_stmt()).ok();
            }
            
            // Update coverage after trace file handling
            if let Some(ref mut coverage) = self.coverage {
                coverage.entry(current.line_number).or_default();
            }
            
            // Execute statement
            match self.execute_statement(self.get_current_stmt()) {
                Ok(()) => {
                    // Update coverage
                    if let Some(ref mut coverage) = self.coverage {
                        coverage.entry(current.line_number)
                            .and_modify(|count| *count += 1)
                            .or_insert(1);
                    }
                    
                    // Move to next statement
                    self.advance_location();
                }
                Err(err) => {
                    self.run_status = match err {
                        BasicError::Syntax { .. } => RunStatus::EndErrorSyntax,
                        BasicError::Runtime { .. } => RunStatus::EndErrorRuntime,
                        BasicError::Internal { .. } => RunStatus::EndErrorInternal,
                        BasicError::Type { .. } => RunStatus::EndErrorType,
                    };
                    return self.run_status;
                }
            }
        }
        
        self.run_status
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<(), BasicError> {
        match stmt {
            Statement::Let { var, value } => {
                let result = self.evaluate_expression(value)?;
                self.put_symbol(var.clone(), result);
                Ok(())
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
                if result == SymbolValue::Number(1.0) {
                    self.execute_statement(then_stmt)?;
                } else if let Some(stmt) = else_stmt {
                    self.execute_statement(stmt)?;
                }
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
                            // Only keep the for_record if we're still looping
                            self.for_stack.pop();  // Remove the old one
                            self.for_stack.push(for_record);  // Push the current one back
                        }
                    } else {
                        // Loop is done, remove the record
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
                self.goto_line(line)?;
                Ok(())
            }
            Statement::Gosub { line } => {
                self.gosub_stack.push(self.location);
                self.goto_line(line)?;
                Ok(())
            }
            Statement::Return => {
                if let Some(return_loc) = self.gosub_stack.pop() {
                    self.location = return_loc;
                }
                Ok(())
            }
            Statement::End => {
                self.run_status = RunStatus::EndOfProgram;
                Ok(())
            }
            Statement::Stop => {
                self.run_status = RunStatus::BreakCode;
                Ok(())
            }
            Statement::Rem { comment } => {
                // REM statements are comments, do nothing
                Ok(())
            }
            Statement::Data { values } => {
                self.data_values.extend(values.iter().map(|v| self.evaluate_expression(v).unwrap()));
                Ok(())
            }
            Statement::Read { var } => {
                if let Some(value) = self.data_values.get(self.data_pointer) {
                    self.put_symbol(var.clone(), value.clone());
                    self.data_pointer += 1;
                    Ok(())
                } else {
                    Err(BasicError::Runtime {
                        message: "No more data available".to_string(),
                        line_number: None,
                    })
                }
            }
            Statement::Restore { line } => {
                self.data_pointer = 0;
                if let Some(target_line) = line {
                    for (i, line) in self.program.lines.iter().enumerate() {
                        if line.line_number >= target_line {
                            self.data_pointer = 0;
                            for stmt in &line.statements {
                                if let Statement::Data { .. } = stmt {
                                    break;
                                }
                            }
                            break;
                        }
                    }
                }
                Ok(())
            }
            Statement::Dim { arrays } => {
                for array in arrays {
                    if array.dimensions.is_empty() {
                        return Err(BasicError::Syntax {
                            message: format!("Array '{}' requires at least one dimension", array.name),
                            line_number: None,
                        });
                    }

                    self.put_symbol(array.name.clone(), SymbolValue::Array(array.dimensions.clone()));
                }
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
                let mut evaluated_indices = Vec::new();
                for idx in indices {
                    match self.evaluate_expression(idx)? {
                        SymbolValue::Number(n) => evaluated_indices.push(n as usize),
                        _ => return Err(BasicError::Runtime {
                            message: "Array index must be numeric".to_string(),
                            line_number: None,
                        }),
                    }
                }
                
                match self.get_symbol(name)? {
                    SymbolValue::Array(arr) => {
                        let mut current = &arr;
                        for &i in &evaluated_indices[..evaluated_indices.len()-1] {
                            if let Some(SymbolValue::Array(next)) = current.get(i) {
                                current = next;
                            } else {
                                return Err(BasicError::Runtime {
                                    message: "Invalid array index".to_string(),
                                    line_number: None,
                                });
                            }
                        }
                        
                        if let Some(last) = index.last() {
                            current.get(*last).cloned().ok_or_else(|| BasicError::Runtime {
                                message: "Array index out of bounds".to_string(),
                                line_number: None,
                            })
                        } else {
                            Err(BasicError::Runtime {
                                message: "Empty array index".to_string(),
                                line_number: None,
                            })
                        }
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("{} is not an array", name),
                        line_number: None,
                    }),
                }
            }
            ExpressionType::BinaryOp { op, left, right } => {
                let left = self.evaluate_expression(left)?;
                let right = self.evaluate_expression(right)?;
                
                match (left, right) {
                    (SymbolValue::Number(l), SymbolValue::Number(r)) => {
                        let result = match op.as_str() {
                            "+" => l + r,
                            "-" => l - r,
                            "*" => l * r,
                            "/" => {
                                if r == 0.0 {
                                    return Err(BasicError::Runtime {
                                        message: "Division by zero".to_string(),
                                        line_number: None,
                                    });
                                }
                                l / r
                            }
                            "^" => l.powf(r),
                            "=" => if l == r { 1.0 } else { 0.0 },
                            "<>" => if l != r { 1.0 } else { 0.0 },
                            "<" => if l < r { 1.0 } else { 0.0 },
                            "<=" => if l <= r { 1.0 } else { 0.0 },
                            ">" => if l > r { 1.0 } else { 0.0 },
                            ">=" => if l >= r { 1.0 } else { 0.0 },
                            "AND" => if l != 0.0 && r != 0.0 { 1.0 } else { 0.0 },
                            "OR" => if l != 0.0 || r != 0.0 { 1.0 } else { 0.0 },
                            _ => return Err(BasicError::Runtime {
                                message: format!("Unknown operator: {}", op),
                                line_number: None,
                            }),
                        };
                        Ok(SymbolValue::Number(result))
                    }
                    (SymbolValue::String(l), SymbolValue::String(r)) => {
                        let result = match op.as_str() {
                            "+" => l + &r,
                            "=" => if l == r { 1.0 } else { 0.0 },
                            "<>" => if l != r { 1.0 } else { 0.0 },
                            "<" => if l < r { 1.0 } else { 0.0 },
                            "<=" => if l <= r { 1.0 } else { 0.0 },
                            ">" => if l > r { 1.0 } else { 0.0 },
                            ">=" => if l >= r { 1.0 } else { 0.0 },
                            _ => return Err(BasicError::Runtime {
                                message: format!("Invalid operator {} for strings", op),
                                line_number: None,
                            }),
                        };
                        match op.as_str() {
                            "+" => Ok(SymbolValue::String(result)),
                            _ => Ok(SymbolValue::Number(result)),
                        }
                    }
                    _ => Err(BasicError::Runtime {
                        message: "Type mismatch in expression".to_string(),
                        line_number: None,
                    }),
                }
            }
            ExpressionType::UnaryOp { op, expr } => {
                let value = self.evaluate_expression(expr)?;
                match value {
                    SymbolValue::Number(n) => {
                        let result = match op.as_str() {
                            "-" => -n,
                            "NOT" => if n == 0.0 { 1.0 } else { 0.0 },
                            _ => return Err(BasicError::Runtime {
                                message: format!("Unknown operator: {}", op),
                                line_number: None,
                            }),
                        };
                        Ok(SymbolValue::Number(result))
                    }
                    _ => Err(BasicError::Runtime {
                        message: format!("Invalid operand for {}", op),
                        line_number: None,
                    }),
                }
            }
            ExpressionType::FunctionCall { name, args } => {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    let value = self.evaluate_expression(arg)?;
                    match value {
                        SymbolValue::Number(n) => evaluated_args.push(n),
                        _ => return Err(BasicError::Runtime {
                            message: "Function arguments must be numeric".to_string(),
                            line_number: None,
                        }),
                    }
                }
                
                match name.as_str() {
                    "ABS" => {
                        if evaluated_args.len() != 1 {
                            return Err(BasicError::Runtime {
                                message: "ABS requires 1 argument".to_string(),
                                line_number: None,
                            });
                        }
                        Ok(SymbolValue::Number(evaluated_args[0].abs()))
                    }
                    "RND" => {
                        if evaluated_args.len() != 1 {
                            return Err(BasicError::Runtime {
                                message: "RND requires 1 argument".to_string(),
                                line_number: None,
                            });
                        }
                        let mut rng = rand::thread_rng();
                        Ok(SymbolValue::Number(rng.gen()))
                    }
                    // Add more functions here...
                    _ => Err(BasicError::Runtime {
                        message: format!("Unknown function: {}", name),
                        line_number: None,
                    }),
                }
            }
        }
    }

    fn get_symbol(&self, name: &str) -> Result<SymbolValue, BasicError> {
        self.symbols.get(name).ok_or_else(|| BasicError::Runtime {
            message: format!("Undefined variable: {}", name),
            line_number: None,
        })
    }

    fn put_symbol(&mut self, name: String, value: SymbolValue) {
        self.symbols.put_symbol(name, value);
        if self.data_breakpoints.contains(&name) {
            self.run_status = RunStatus::BreakData;
        }
    }

    fn goto_line(&mut self, line_number: usize) -> Result<(), BasicError> {
        if let Some(&index) = self.line_number_map.get(&line_number) {
            self.location = ControlLocation { index, offset: 0 };
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
        
        // After REM, skip to next line
        if let StatementType::Rem { .. } = self.get_current_stmt().stmt_type {
            self.location.index += 1;
            self.location.offset = 0;
            return;
        }
        
        self.location.offset += 1;
        if current_offset >= current_line.statements.len() - 1 {
            self.location.index += 1;
            self.location.offset = 0;
        }
        
        // Check if we've reached the end
        if self.location.index >= self.program.lines.len() {
            self.run_status = RunStatus::EndOfProgram;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{Token, Statement, Expression, StatementType, ExpressionType};

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
            (10, vec![Statement::new_let("X".to_string(), Expression::new_number(1.0))]),
            (20, vec![Statement::new_let("Y".to_string(), Expression::new_number(2.0))]),
            (30, vec![Statement::new_let("Z".to_string(), Expression::new_number(3.0))]),
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
            (10, vec![Statement::new_let("X".to_string(), Expression::new_number(1.0))]),
            (20, vec![Statement::new_goto(40)]),
            (30, vec![Statement::new_let("Y".to_string(), Expression::new_number(2.0))]),
            (40, vec![Statement::new_let("Z".to_string(), Expression::new_number(3.0))]),
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
            (10, vec![Statement::new_let("X".to_string(), Expression::new_number(1.0))]),
            (20, vec![
                Statement::new_rem("This is a comment".to_string()),
                Statement::new_let("Y".to_string(), Expression::new_number(2.0)), // Should be ignored
            ]),
            (30, vec![Statement::new_let("Z".to_string(), Expression::new_number(3.0))]),
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
                Statement::new_let("X".to_string(), Expression::new_number(1.0)),
                Statement::new_let("Y".to_string(), Expression::new_number(2.0)),
                Statement::new_let("Z".to_string(), Expression::new_number(3.0)),
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
                Expression::new_array(
                    "A".to_string(),
                    vec![
                        Expression::new_number(1.0),
                        Expression::new_number(1.0),
                    ],
                ),
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
