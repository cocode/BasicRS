use std::collections::HashMap;
use crate::basic_types::{Program, Statement, Expression, ExpressionType, PrintItem};
use crate::llvm_ir_builder::LLVMIRBuilder;

pub struct LLVMCodeGenerator {
    builder: LLVMIRBuilder,
    symbol_table: HashMap<String, String>, // variable name -> LLVM variable name
    array_info: HashMap<String, ArrayInfo>,
    line_blocks: HashMap<usize, String>, // line number -> block name
    current_line_index: usize,
    program: Program,
    debug: bool,
    trace: bool,
}

#[derive(Clone)]
struct ArrayInfo {
    global_name: String,
    dimensions: Vec<usize>,
    element_type: String, // "double" or "i8*"
}

impl LLVMCodeGenerator {
    pub fn new(program: Program, debug: bool, trace: bool) -> Self {
        let mut builder = LLVMIRBuilder::new();
        
        // Set up module header
        builder.add_module_header("basic_program");
        
        // Declare external C functions
        Self::declare_external_functions(&mut builder);
        
        Self {
            builder,
            symbol_table: HashMap::new(),
            array_info: HashMap::new(),
            line_blocks: HashMap::new(),
            current_line_index: 0,
            program,
            debug,
            trace,
        }
    }
    
    pub fn generate_ir(&mut self) -> String {
        // Allocate variables
        self.allocate_variables();
        
        // Create main function
        self.builder.add_main_function();
        
        // Initialize runtime (seed random, etc.)
        self.init_runtime();
        
        // Create basic blocks for each line
        for line in &self.program.lines {
            let block_name = format!("line_{}", line.line_number);
            self.line_blocks.insert(line.line_number, block_name.clone());
        }
        
        // Branch to first line if program exists
        if !self.program.lines.is_empty() {
            let first_line = self.program.lines[0].line_number;
            let first_block = self.line_blocks.get(&first_line).unwrap();
            self.builder.add_branch(first_block);
        } else {
            self.builder.add_return(Some("0"));
            self.builder.end_function();
            return self.builder.build();
        }
        
        // Generate code for each line
        let line_info: Vec<_> = self.program.lines.iter().enumerate()
            .map(|(i, line)| (i, line.line_number, line.statements.clone()))
            .collect();
        for (i, line_number, statements) in line_info {
            self.current_line_index = i;
            let block_name = self.line_blocks.get(&line_number).unwrap();
            self.builder.add_basic_block(block_name);
            
            // Add trace output if enabled
            if self.trace {
                self.emit_trace(line_number);
            }
            
            // Generate statements for this line
            self.generate_line_statements(&statements);
            
            // If not terminated, branch to next line
            if i + 1 < self.program.lines.len() {
                let next_line = self.program.lines[i + 1].line_number;
                let next_block = self.line_blocks.get(&next_line).unwrap();
                self.builder.add_branch(next_block);
            } else {
                self.builder.add_return(Some("0"));
            }
        }
        
        self.builder.end_function();
        self.builder.build()
    }
    
    fn declare_external_functions(builder: &mut LLVMIRBuilder) {
        // I/O functions
        builder.declare_function("printf", "i32", &["i8*".to_string()], true);
        builder.declare_function("scanf", "i32", &["i8*".to_string()], true);
        builder.declare_function("sprintf", "i32", &["i8*".to_string(), "i8*".to_string()], true);
        builder.declare_function("sscanf", "i32", &["i8*".to_string(), "i8*".to_string()], true);
        
        // Memory management
        builder.declare_function("malloc", "i8*", &["i64".to_string()], false);
        builder.declare_function("strlen", "i64", &["i8*".to_string()], false);
        builder.declare_function("strcmp", "i32", &["i8*".to_string(), "i8*".to_string()], false);
        builder.declare_function("strcat", "i8*", &["i8*".to_string(), "i8*".to_string()], false);
        builder.declare_function("strcpy", "i8*", &["i8*".to_string(), "i8*".to_string()], false);
        builder.declare_function("strncpy", "i8*", &["i8*".to_string(), "i8*".to_string(), "i64".to_string()], false);
        
        // Math functions
        builder.declare_function("sin", "double", &["double".to_string()], false);
        builder.declare_function("cos", "double", &["double".to_string()], false);
        builder.declare_function("sqrt", "double", &["double".to_string()], false);
        builder.declare_function("exp", "double", &["double".to_string()], false);
        builder.declare_function("log", "double", &["double".to_string()], false);
        builder.declare_function("fabs", "double", &["double".to_string()], false);
        builder.declare_function("pow", "double", &["double".to_string(), "double".to_string()], false);
        builder.declare_function("floor", "double", &["double".to_string()], false);
        
        // Random functions
        builder.declare_function("rand", "i32", &[], false);
        builder.declare_function("srand", "void", &["i32".to_string()], false);
        builder.declare_function("time", "i64", &["i64*".to_string()], false);
        
        builder.line(""); // Add blank line after declarations
    }
    
    fn allocate_variables(&mut self) {
        // Scan program for variables and arrays
        let mut variables = HashMap::new();
        let mut arrays = HashMap::new();
        
        for line in &self.program.lines {
            for statement in &line.statements {
                self.collect_variables_from_statement(statement, &mut variables, &mut arrays);
            }
        }
        
        // Allocate global variables
        for (var_name, _var_type) in variables {
            let global_name = format!("@global_{}", var_name);
            let llvm_type = if var_name.ends_with('$') { "i8*" } else { "double" };
            let initializer = if var_name.ends_with('$') { 
                Some("null") 
            } else { 
                Some("0.0") 
            };
            
            self.builder.add_global_variable(&global_name, llvm_type, initializer, false);
            self.symbol_table.insert(var_name, global_name);
        }
        
        // Allocate arrays
        for (array_name, dimensions) in arrays {
            let global_name = format!("@array_{}", array_name);
            let element_type = if array_name.ends_with('$') { "i8*" } else { "double" };
            
            // For now, create a simple array - in practice this would be more complex
            let array_size = dimensions.iter().product::<usize>();
            let array_type = format!("[{} x {}]", array_size, element_type);
            
            self.builder.add_global_variable(&global_name, &array_type, None, false);
            
            self.array_info.insert(array_name, ArrayInfo {
                global_name,
                dimensions,
                element_type: element_type.to_string(),
            });
        }
    }
    
    fn collect_variables_from_statement(&self, statement: &Statement, variables: &mut HashMap<String, String>, arrays: &mut HashMap<String, Vec<usize>>) {
        match statement {
            Statement::Let { var, value: _ } => {
                if let ExpressionType::Variable(name) = &var.expr_type {
                    let var_type = if name.ends_with('$') { "string" } else { "number" };
                    variables.insert(name.clone(), var_type.to_string());
                }
            },
            Statement::Dim { arrays: dim_arrays } => {
                for array_decl in dim_arrays {
                    let _array_type = if array_decl.name.ends_with('$') { "string" } else { "number" };
                    arrays.insert(array_decl.name.clone(), array_decl.dimensions.clone());
                }
            },
            _ => {}
        }
    }
    
    fn init_runtime(&mut self) {
        // Seed random number generator
        let null_ptr = "null";
        let time_call = self.builder.add_call("time", &[null_ptr.to_string()], "i64", "time_val");
        let time_int = self.builder.add_trunc(&time_call, "i32", "time_int");
        self.builder.add_call_void("srand", &[time_int]);
    }
    
    fn emit_trace(&mut self, line_number: usize) {
        let debug_str = format!("Executing line {}\\n", line_number);
        let debug_name = format!("debug_str_{}", line_number);
        self.builder.add_string_constant(&debug_name, &debug_str);
        
        let debug_ptr = self.builder.add_bitcast(&format!("@{}", debug_name), "i8*", "debug_ptr");
        self.builder.add_call_void("printf", &[debug_ptr]);
    }
    
    fn generate_line_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.generate_statement(statement);
        }
    }
    
    fn generate_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Let { var, value } => self.codegen_let(var, value),
            Statement::Print { items } => self.codegen_print(items),
            Statement::End => {
                self.builder.add_return(Some("0"));
            },
            Statement::Stop => {
                self.builder.add_return(Some("1"));
            },
            Statement::Rem { comment: _ } => {
                // Comments are no-ops in generated code
            },
            _ => {
                if self.debug {
                    self.builder.comment(&format!("TODO: Statement {:?} not yet implemented", statement));
                }
            }
        }
    }
    
    fn codegen_let(&mut self, var: &Expression, value: &Expression) {
        // TODO: Implement LET statement
        if self.debug {
            self.builder.comment(&format!("LET statement: {:?} = {:?}", var, value));
        }
    }
    
    fn codegen_print(&mut self, items: &[PrintItem]) {
        if self.debug {
            self.builder.comment(&format!("PRINT statement with {} items", items.len()));
        }
        
        for item in items {
            match item {
                PrintItem::Expression(expr) => {
                    match &expr.expr_type {
                        ExpressionType::String(s) => {
                            // Generate string constant at module level
                            let str_name = format!("str_{}", self.builder.next_global().replace("@", ""));
                            self.builder.add_string_constant(&str_name, s);
                            
                            let str_ptr = self.builder.add_bitcast(&format!("@{}", str_name), "i8*", "str_ptr");
                            self.builder.add_call_void("printf", &[str_ptr]);
                        }
                        ExpressionType::Number(n) => {
                            // Handle number literals
                            let format_str = self.builder.next_global();
                            self.builder.add_string_constant(&format_str, "%.2f\n");
                            
                            let format_ptr = self.builder.add_bitcast(&format_str, "i8*", "format_ptr");
                            self.builder.add_call_void("printf", &[format_ptr, format!("{:.2}", n)]);
                        }
                        _ => {
                            // For other expression types, use the expression codegen
                            let result = self.codegen_expression(expr);
                            let format_str = self.builder.next_global();
                            self.builder.add_string_constant(&format_str, "%f\n");
                            
                            let format_ptr = self.builder.add_bitcast(&format_str, "i8*", "format_ptr");
                            self.builder.add_call_void("printf", &[format_ptr, result]);
                        }
                    }
                }
                PrintItem::Tab(_) => {
                    // TODO: Implement tab functionality
                    if self.debug {
                        self.builder.comment("TODO: Implement TAB");
                    }
                }
                PrintItem::Comma => {
                    // TODO: Implement comma spacing
                    if self.debug {
                        self.builder.comment("TODO: Implement comma spacing");
                    }
                }
                PrintItem::Semicolon => {
                    // TODO: Implement semicolon (no spacing)
                    if self.debug {
                        self.builder.comment("TODO: Implement semicolon");
                    }
                }
            }
        }
    }
    
    fn codegen_expression(&mut self, expr: &Expression) -> String {
        match &expr.expr_type {
            ExpressionType::Number(n) => {
                // Return the number as a string for printf
                format!("{:.2}", n)
            }
            ExpressionType::String(s) => {
                let str_name = self.builder.next_global();
                self.builder.add_string_constant(&str_name, s);
                
                let str_ptr = self.builder.add_bitcast(&str_name, "i8*", "str_ptr");
                self.builder.add_call("printf", &[str_ptr], "i32", "print_result");
                
                "0".to_string() // Return dummy value for now
            }
            ExpressionType::Variable(_name) => {
                // For now, just return a dummy value
                // TODO: Implement variable lookup
                "0".to_string()
            }
            ExpressionType::BinaryOp { left, op, right } => {
                let left_val = self.codegen_expression(left);
                let right_val = self.codegen_expression(right);
                
                let temp = self.builder.next_temp();
                match op.as_str() {
                    "+" => {
                        self.builder.add_binary_op("fadd", &left_val, &right_val, "double", &temp[1..]);
                    }
                    "-" => {
                        self.builder.add_binary_op("fsub", &left_val, &right_val, "double", &temp[1..]);
                    }
                    "*" => {
                        self.builder.add_binary_op("fmul", &left_val, &right_val, "double", &temp[1..]);
                    }
                    "/" => {
                        self.builder.add_binary_op("fdiv", &left_val, &right_val, "double", &temp[1..]);
                    }
                    _ => {
                        // Default to addition for unknown operators
                        self.builder.add_binary_op("fadd", &left_val, &right_val, "double", &temp[1..]);
                    }
                }
                temp
            }
            _ => {
                // For other expression types, return a dummy value
                "0".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_types::{ProgramLine, Expression};

    fn create_test_program() -> Program {
        let mut program = Program::new();
        
        // Add a simple program: 10 PRINT "HELLO" : 20 END
        let line1 = ProgramLine {
            line_number: 10,
            source: "10 PRINT \"HELLO\"".to_string(),
            statements: vec![
                Statement::Print {
                    items: vec![PrintItem::Expression(Expression::new_string("HELLO".to_string()))]
                }
            ],
        };
        
        let line2 = ProgramLine {
            line_number: 20,
            source: "20 END".to_string(),
            statements: vec![Statement::End],
        };
        
        program.add_line(10, line1.source.clone(), line1.statements.clone());
        program.add_line(20, line2.source.clone(), line2.statements.clone());
        
        program
    }

    fn create_variable_test_program() -> Program {
        let mut program = Program::new();
        
        // Add a program with variables: 10 LET A = 42 : 20 PRINT A : 30 END
        let line1 = ProgramLine {
            line_number: 10,
            source: "10 LET A = 42".to_string(),
            statements: vec![
                Statement::Let {
                    var: Expression::new_variable("A".to_string()),
                    value: Expression::new_number(42.0),
                }
            ],
        };
        
        let line2 = ProgramLine {
            line_number: 20,
            source: "20 PRINT A".to_string(),
            statements: vec![
                Statement::Print {
                    items: vec![PrintItem::Expression(Expression::new_variable("A".to_string()))]
                }
            ],
        };
        
        let line3 = ProgramLine {
            line_number: 30,
            source: "30 END".to_string(),
            statements: vec![Statement::End],
        };
        
        program.add_line(10, line1.source.clone(), line1.statements.clone());
        program.add_line(20, line2.source.clone(), line2.statements.clone());
        program.add_line(30, line3.source.clone(), line3.statements.clone());
        
        program
    }

    fn create_array_test_program() -> Program {
        let mut program = Program::new();
        
        // Add a program with arrays: 10 DIM A(10) : 20 LET A(5) = 42 : 30 END
        let line1 = ProgramLine {
            line_number: 10,
            source: "10 DIM A(10)".to_string(),
            statements: vec![
                Statement::Dim {
                    arrays: vec![crate::basic_types::ArrayDecl {
                        name: "A".to_string(),
                        dimensions: vec![10],
                    }]
                }
            ],
        };
        
        let line2 = ProgramLine {
            line_number: 20,
            source: "20 LET A(5) = 42".to_string(),
            statements: vec![
                Statement::Let {
                    var: Expression::new_array("A".to_string(), vec![Expression::new_number(5.0)]),
                    value: Expression::new_number(42.0),
                }
            ],
        };
        
        let line3 = ProgramLine {
            line_number: 30,
            source: "30 END".to_string(),
            statements: vec![Statement::End],
        };
        
        program.add_line(10, line1.source.clone(), line1.statements.clone());
        program.add_line(20, line2.source.clone(), line2.statements.clone());
        program.add_line(30, line3.source.clone(), line3.statements.clone());
        
        program
    }

    #[test]
    fn test_basic_program_generation() {
        let program = create_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        
        let ir = codegen.generate_ir();
        
        // Verify basic structure
        assert!(ir.contains("define i32 @main()"));
        assert!(ir.contains("entry:"));
        assert!(ir.contains("line_10:"));
        assert!(ir.contains("line_20:"));
        assert!(ir.contains("ret i32 0"));
        
        // Verify external function declarations
        assert!(ir.contains("declare i32 @printf"));
        assert!(ir.contains("declare i32 @scanf"));
        assert!(ir.contains("declare i8* @malloc"));
    }

    #[test]
    fn test_program_with_variables() {
        let program = create_variable_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        
        let ir = codegen.generate_ir();
        
        // Verify variable allocation
        assert!(ir.contains("@global_A = global double double = 0.0"));
        
        // Verify program structure
        assert!(ir.contains("line_10:"));
        assert!(ir.contains("line_20:"));
        assert!(ir.contains("line_30:"));
    }

    #[test]
    fn test_program_with_arrays() {
        let program = create_array_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        
        let ir = codegen.generate_ir();
        
        // Verify array allocation
        assert!(ir.contains("@array_A = global"));
        
        // Verify program structure
        assert!(ir.contains("line_10:"));
        assert!(ir.contains("line_20:"));
        assert!(ir.contains("line_30:"));
    }

    #[test]
    fn test_debug_mode() {
        let program = create_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, true, false);
        
        let ir = codegen.generate_ir();
        println!("Debug mode IR:\n{}", ir);
        
        // Verify debug comments are added
        assert!(ir.contains("; PRINT statement with"));
    }

    #[test]
    fn test_trace_mode() {
        let program = create_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, false, true);
        
        let ir = codegen.generate_ir();
        
        // Verify trace statements are added
        assert!(ir.contains("Executing line 10"));
        assert!(ir.contains("Executing line 20"));
        assert!(ir.contains("call void @printf"));
    }

    #[test]
    fn test_empty_program() {
        let program = Program::new();
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        
        let ir = codegen.generate_ir();
        
        // Verify empty program still generates valid IR
        assert!(ir.contains("define i32 @main()"));
        assert!(ir.contains("entry:"));
        assert!(ir.contains("ret i32 0"));
    }

    #[test]
    fn test_stop_statement() {
        let mut program = Program::new();
        program.add_line(10, "10 STOP".to_string(), vec![Statement::Stop]);
        
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        let ir = codegen.generate_ir();
        
        // Verify STOP generates return 1
        assert!(ir.contains("ret i32 1"));
    }

    #[test]
    fn test_rem_statement() {
        let mut program = Program::new();
        program.add_line(10, "10 REM This is a comment".to_string(), vec![
            Statement::Rem { comment: "This is a comment".to_string() }
        ]);
        program.add_line(20, "20 END".to_string(), vec![Statement::End]);
        
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        let ir = codegen.generate_ir();
        
        // Verify REM statements are ignored (no-op)
        assert!(ir.contains("line_10:"));
        assert!(ir.contains("line_20:"));
        assert!(ir.contains("ret i32 0"));
    }

    #[test]
    fn test_variable_collection() {
        let mut program = Program::new();
        
        // Add variables of different types
        program.add_line(10, "10 LET A = 42".to_string(), vec![
            Statement::Let {
                var: Expression::new_variable("A".to_string()),
                value: Expression::new_number(42.0),
            }
        ]);
        
        program.add_line(20, "20 LET B$ = \"HELLO\"".to_string(), vec![
            Statement::Let {
                var: Expression::new_variable("B$".to_string()),
                value: Expression::new_string("HELLO".to_string()),
            }
        ]);
        
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        let ir = codegen.generate_ir();
        
        // Verify numeric variable allocation
        assert!(ir.contains("@global_A = global double double = 0.0"));
        
        // Verify string variable allocation
        assert!(ir.contains("@global_B$ = global i8* i8* = null"));
    }

    #[test]
    fn test_array_collection() {
        let mut program = Program::new();
        
        // Add arrays of different types
        program.add_line(10, "10 DIM A(10, 5)".to_string(), vec![
            Statement::Dim {
                arrays: vec![crate::basic_types::ArrayDecl {
                    name: "A".to_string(),
                    dimensions: vec![10, 5],
                }]
            }
        ]);
        
        program.add_line(20, "20 DIM B$(5)".to_string(), vec![
            Statement::Dim {
                arrays: vec![crate::basic_types::ArrayDecl {
                    name: "B$".to_string(),
                    dimensions: vec![5],
                }]
            }
        ]);
        
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        let ir = codegen.generate_ir();
        
        // Verify array allocations
        assert!(ir.contains("@array_A = global"));
        assert!(ir.contains("@array_B$ = global"));
    }

    #[test]
    fn test_external_function_declarations() {
        let program = create_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        
        let ir = codegen.generate_ir();
        
        // Verify all external function declarations
        assert!(ir.contains("declare i32 @printf"));
        assert!(ir.contains("declare i32 @scanf"));
        assert!(ir.contains("declare i32 @sprintf"));
        assert!(ir.contains("declare i32 @sscanf"));
        assert!(ir.contains("declare i8* @malloc"));
        assert!(ir.contains("declare i64 @strlen"));
        assert!(ir.contains("declare i32 @strcmp"));
        assert!(ir.contains("declare i8* @strcat"));
        assert!(ir.contains("declare i8* @strcpy"));
        assert!(ir.contains("declare i8* @strncpy"));
        
        // Math functions
        assert!(ir.contains("declare double @sin"));
        assert!(ir.contains("declare double @cos"));
        assert!(ir.contains("declare double @sqrt"));
        assert!(ir.contains("declare double @exp"));
        assert!(ir.contains("declare double @log"));
        assert!(ir.contains("declare double @fabs"));
        assert!(ir.contains("declare double @pow"));
        assert!(ir.contains("declare double @floor"));
        
        // Random functions
        assert!(ir.contains("declare i32 @rand"));
        assert!(ir.contains("declare void @srand"));
        assert!(ir.contains("declare i64 @time"));
    }

    #[test]
    fn test_runtime_initialization() {
        let program = create_test_program();
        let mut codegen = LLVMCodeGenerator::new(program, false, false);
        
        let ir = codegen.generate_ir();
        
        // Verify runtime initialization (random seed)
        assert!(ir.contains("call i64 @time"));
        assert!(ir.contains("call void @srand"));
    }
} 