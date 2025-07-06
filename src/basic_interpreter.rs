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
        self.symbols.get(name)
    }

    pub fn set_symbol_value(&mut self, name: String, value: SymbolValue) {
        self.symbols.put(name, value);
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

    pub fn run(&mut self) -> RunStatus {
        while self.run_status == RunStatus::Run {
            let current_line = self.get_current_line().line_number;
            let current_offset = self.location.offset;
            
            // Check breakpoints
            if self.breakpoints.contains(&(current_line, current_offset)) {
                self.run_status = RunStatus::BreakCode;
                return self.run_status;
            }
            
            // Get current statement before any trace/coverage operations
            let current_stmt = self.get_current_stmt().clone();
            
            // Write trace
            if let Some(ref mut file) = self.trace_file {
                if self.location.offset == 0 {
                    writeln!(file, ">{}", self.get_current_line().source).ok();
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
                    return self.run_status;
                }
            }
        }
        
        self.run_status
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
                let step_expr = step.clone().unwrap_or_else(|| Expression::Number(1.0));
                
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
                self.run_status = RunStatus::End;
                Ok(())
            }
            Statement::Stop => {
                self.run_status = RunStatus::Stop;
                Ok(())
            }
            Statement::Rem { .. } => Ok(()),
            Statement::Data { values } => {
                self.data_values.extend(values.iter().cloned());
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
                    self.symbols.create_array(array.name.clone(), array.dimensions.clone())?;
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
                self.internal_symbols.define_function(name.clone(), params.clone(), expr.clone())?;
                Ok(())
            }
        }
    }

    fn evaluate_expression(&mut self, expr: &Expression) -> Result<SymbolValue, BasicError> {
        match expr {
            Expression::Number(n) => Ok(SymbolValue::Number(*n)),
            Expression::String(s) => Ok(SymbolValue::String(s.clone())),
            Expression::Variable(name) => self.get_symbol(name),
            Expression::Array { name, indices } => {
                let idx_values: Result<Vec<usize>, BasicError> = indices.iter()
                    .map(|expr| match self.evaluate_expression(expr)? {
                        SymbolValue::Number(n) if n >= 0.0 && n.fract() == 0.0 => Ok(n as usize),
                        _ => Err(BasicError::Runtime {
                            message: "Array index must be a non-negative integer".to_string(),
                            line_number: None,
                        })
                    })
                    .collect();
                
                let indices = idx_values?;
                self.symbols.get_array_element(name, &indices)
            },
            Expression::Function { name, args } => {
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                
                if let Some(func) = PredefinedFunctions::get(name) {
                    func.call(&evaluated_args)
                } else {
                    self.internal_symbols.call_function(name, &evaluated_args)
                }
            }
        }
    }

    fn get_symbol(&self, name: &str) -> Result<SymbolValue, BasicError> {
        // Try current scope first, then parent scopes
        if let Some(value) = self.symbols.get(name) {
            Ok(value.clone())
        } else if let Some(value) = self.internal_symbols.get(name) {
            Ok(value.clone())
        } else {
            Err(BasicError::Runtime {
                message: format!("Undefined variable: {}", name),
                line_number: None,
            })
        }
    }

    fn put_symbol(&mut self, name: String, value: SymbolValue) {
        // Always put in current scope
        self.symbols.put(name, value);
        if self.data_breakpoints.contains(&name) {
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
                self.run_status = RunStatus::End;
            }
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
