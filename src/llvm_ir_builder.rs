use std::collections::HashMap;

/// Text-based LLVM-IR builder that generates LLVM-IR as strings
/// Similar to the Java LLVMBuilder approach
pub struct LLVMIRBuilder {
    buffer: String,
    temp_counter: u32,
    global_counter: u32,
    function_counter: u32,
    block_counter: u32,
    declared_functions: HashMap<String, FunctionSignature>,
    global_variables: HashMap<String, GlobalVariable>,
}

#[derive(Clone)]
pub struct FunctionSignature {
    pub return_type: String,
    pub params: Vec<String>,
    pub is_vararg: bool,
}

#[derive(Clone)]
pub struct GlobalVariable {
    pub var_type: String,
    pub initializer: Option<String>,
    pub is_constant: bool,
}

impl LLVMIRBuilder {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            temp_counter: 0,
            global_counter: 0,
            function_counter: 0,
            block_counter: 0,
            declared_functions: HashMap::new(),
            global_variables: HashMap::new(),
        }
    }

    pub fn build(&self) -> String {
        self.buffer.clone()
    }

    // Module and function declarations
    pub fn add_module_header(&mut self, module_name: &str) {
        self.line(&format!("; ModuleID = '{}'", module_name));
        self.line("source_filename = \"basic_program\"");
        self.line("");
    }

    pub fn declare_function(&mut self, name: &str, return_type: &str, params: &[String], is_vararg: bool) {
        let param_str = params.join(", ");
        let vararg_str = if is_vararg { ", ..." } else { "" };
        self.line(&format!("declare {} @{}({}{})", return_type, name, param_str, vararg_str));
        
        self.declared_functions.insert(name.to_string(), FunctionSignature {
            return_type: return_type.to_string(),
            params: params.to_vec(),
            is_vararg,
        });
    }

    pub fn add_main_function(&mut self) {
        self.line("define i32 @main() {");
        self.line("entry:");
    }

    pub fn end_function(&mut self) {
        self.line("}");
        self.line("");
    }

    // Basic blocks
    pub fn add_basic_block(&mut self, name: &str) {
        self.line(&format!("{}:", name));
    }

    // Instructions
    pub fn add_alloca(&mut self, var_type: &str, name: &str) {
        self.line(&format!("  {} = alloca {}", name, var_type));
    }

    pub fn add_store(&mut self, value: &str, ptr: &str) {
        self.line(&format!("  store {} {}, {}* {}", 
            self.get_value_type(value), value, self.get_value_type(value), ptr));
    }

    pub fn add_load(&mut self, var_type: &str, ptr: &str, name: &str) -> String {
        let load_name = format!("%{}", name);
        self.line(&format!("  {} = load {}, {}* {}", load_name, var_type, var_type, ptr));
        load_name
    }

    pub fn add_binary_op(&mut self, op: &str, left: &str, right: &str, result_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = {} {} {}, {}", result_name, op, result_type, left, right));
        result_name
    }

    pub fn add_call(&mut self, func_name: &str, args: &[String], return_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        let args_str = args.join(", ");
        self.line(&format!("  {} = call {} @{}({})", result_name, return_type, func_name, args_str));
        result_name
    }

    pub fn add_call_void(&mut self, func_name: &str, args: &[String]) {
        let args_str = args.join(", ");
        self.line(&format!("  call void @{}({})", func_name, args_str));
    }

    pub fn add_return(&mut self, value: Option<&str>) {
        match value {
            Some(val) => self.line(&format!("  ret i32 {}", val)),
            None => self.line("  ret void"),
        }
    }

    pub fn add_branch(&mut self, target: &str) {
        self.line(&format!("  br label %{}", target));
    }

    pub fn add_conditional_branch(&mut self, condition: &str, true_target: &str, false_target: &str) {
        self.line(&format!("  br i1 {}, label %{}, label %{}", condition, true_target, false_target));
    }

    pub fn add_icmp(&mut self, pred: &str, left: &str, right: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = icmp {} i32 {}, {}", result_name, pred, left, right));
        result_name
    }

    pub fn add_fcmp(&mut self, pred: &str, left: &str, right: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = fcmp {} double {}, {}", result_name, pred, left, right));
        result_name
    }

    pub fn add_phi(&mut self, var_type: &str, values: &[(String, String)], name: &str) -> String {
        let result_name = format!("%{}", name);
        let phi_values: Vec<String> = values.iter()
            .map(|(val, label)| format!("[ {}, %{} ]", val, label))
            .collect();
        self.line(&format!("  {} = phi {} {}", result_name, var_type, phi_values.join(", ")));
        result_name
    }

    pub fn add_getelementptr(&mut self, ptr: &str, indices: &[String], name: &str) -> String {
        let result_name = format!("%{}", name);
        let indices_str = indices.join(", ");
        self.line(&format!("  {} = getelementptr inbounds {}, {}* {}, {}", 
            result_name, self.get_pointer_base_type(ptr), self.get_pointer_base_type(ptr), ptr, indices_str));
        result_name
    }

    pub fn add_bitcast(&mut self, value: &str, target_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = bitcast {} to {}", result_name, value, target_type));
        result_name
    }

    pub fn add_trunc(&mut self, value: &str, target_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = trunc {} to {}", result_name, value, target_type));
        result_name
    }

    pub fn add_zext(&mut self, value: &str, target_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = zext {} to {}", result_name, value, target_type));
        result_name
    }

    pub fn add_fptosi(&mut self, value: &str, target_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = fptosi double {} to {}", result_name, value, target_type));
        result_name
    }

    pub fn add_sitofp(&mut self, value: &str, target_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = sitofp i32 {} to {}", result_name, value, target_type));
        result_name
    }

    pub fn add_uitofp(&mut self, value: &str, target_type: &str, name: &str) -> String {
        let result_name = format!("%{}", name);
        self.line(&format!("  {} = uitofp i32 {} to {}", result_name, value, target_type));
        result_name
    }

    // Global variables and constants
    pub fn add_global_variable(&mut self, name: &str, var_type: &str, initializer: Option<&str>, is_constant: bool) {
        let constant_str = if is_constant { "constant" } else { "global" };
        let init_str = match initializer {
            Some(init) => format!(" = {}", init),
            None => String::new(),
        };
        self.line(&format!("@{} = {} {} {}{}", name, constant_str, var_type, var_type, init_str));
        
        self.global_variables.insert(name.to_string(), GlobalVariable {
            var_type: var_type.to_string(),
            initializer: initializer.map(|s| s.to_string()),
            is_constant,
        });
    }

    pub fn add_string_constant(&mut self, name: &str, content: &str) {
        // Escape the string content for LLVM-IR
        let escaped = self.escape_string(content);
        let length = escaped.len() + 1; // +1 for null terminator
        self.line(&format!("@{} = private unnamed_addr constant [{} x i8] c\"{}\00\"", 
            name, length, escaped));
    }

    // Utility methods
    pub fn next_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("%t{}", self.temp_counter)
    }

    pub fn next_global(&mut self) -> String {
        self.global_counter += 1;
        format!("@g{}", self.global_counter)
    }

    pub fn next_block(&mut self) -> String {
        self.block_counter += 1;
        format!("bb{}", self.block_counter)
    }

    pub fn line(&mut self, content: &str) {
        self.buffer.push_str(content);
        self.buffer.push('\n');
    }

    pub fn comment(&mut self, comment: &str) {
        self.line(&format!("; {}", comment));
    }

    // Helper methods for type inference
    fn get_value_type(&self, value: &str) -> String {
        if value.contains('.') {
            "double".to_string()
        } else if value.starts_with('"') {
            "i8*".to_string()
        } else {
            "i32".to_string()
        }
    }

    fn get_pointer_base_type(&self, ptr: &str) -> String {
        // Simple heuristic - in practice this would need more sophisticated parsing
        if ptr.contains("double") {
            "double".to_string()
        } else if ptr.contains("i8") {
            "i8".to_string()
        } else {
            "i32".to_string()
        }
    }

    fn escape_string(&self, s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\n' => "\\0A".to_string(),
                '\r' => "\\0D".to_string(),
                '\t' => "\\09".to_string(),
                '"' => "\\22".to_string(),
                '\\' => "\\5C".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }
}

impl Default for LLVMIRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ir_generation() {
        let mut builder = LLVMIRBuilder::new();
        
        builder.add_module_header("test");
        builder.declare_function("printf", "i32", &["i8*".to_string()], true);
        builder.add_main_function();
        builder.add_alloca("i32", "%x");
        builder.add_store("42", "%x");
        builder.add_return(Some("0"));
        builder.end_function();
        
        let result = builder.build();
        assert!(result.contains("define i32 @main()"));
        assert!(result.contains("store i32 42, i32* %x"));
    }

    #[test]
    fn test_string_constant() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_string_constant("hello", "Hello, World!");
        
        let result = builder.build();
        assert!(result.contains("@hello = private unnamed_addr constant"));
        assert!(result.contains("Hello, World!"));
    }

    #[test]
    fn test_binary_operations() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        let result = builder.add_binary_op("add", "1", "2", "i32", "sum");
        assert_eq!(result, "%sum");
        
        let result = builder.add_binary_op("fadd", "1.0", "2.0", "double", "fsum");
        assert_eq!(result, "%fsum");
        
        let result = builder.build();
        assert!(result.contains("%sum = add i32 1, 2"));
        assert!(result.contains("%fsum = fadd double 1.0, 2.0"));
    }

    #[test]
    fn test_function_calls() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        let result = builder.add_call("printf", &["%format".to_string()], "i32", "ret");
        assert_eq!(result, "%ret");
        
        builder.add_call_void("srand", &["42".to_string()]);
        
        let result = builder.build();
        assert!(result.contains("%ret = call i32 @printf(%format)"));
        assert!(result.contains("call void @srand(42)"));
    }

    #[test]
    fn test_control_flow() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        builder.add_branch("bb1");
        builder.add_conditional_branch("%cond", "bb1", "bb2");
        
        let result = builder.build();
        assert!(result.contains("br label %bb1"));
        assert!(result.contains("br i1 %cond, label %bb1, label %bb2"));
    }

    #[test]
    fn test_comparisons() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        let result = builder.add_icmp("eq", "1", "2", "cmp");
        assert_eq!(result, "%cmp");
        
        let result = builder.add_fcmp("ogt", "1.0", "2.0", "fcmp");
        assert_eq!(result, "%fcmp");
        
        let result = builder.build();
        assert!(result.contains("%cmp = icmp eq i32 1, 2"));
        assert!(result.contains("%fcmp = fcmp ogt double 1.0, 2.0"));
    }

    #[test]
    fn test_type_conversions() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        let result = builder.add_fptosi("1.5", "i32", "int");
        assert_eq!(result, "%int");
        
        let result = builder.add_sitofp("42", "double", "float");
        assert_eq!(result, "%float");
        
        let result = builder.add_bitcast("%ptr", "i8*", "cast");
        assert_eq!(result, "%cast");
        
        let result = builder.build();
        assert!(result.contains("%int = fptosi double 1.5 to i32"));
        assert!(result.contains("%float = sitofp i32 42 to double"));
        assert!(result.contains("%cast = bitcast %ptr to i8*"));
    }

    #[test]
    fn test_global_variables() {
        let mut builder = LLVMIRBuilder::new();
        
        builder.add_global_variable("global_var", "i32", Some("42"), false);
        builder.add_global_variable("const_var", "double", Some("3.14"), true);
        builder.add_global_variable("uninit_var", "i8*", None, false);
        
        let result = builder.build();
        assert!(result.contains("@global_var = global i32 i32 = 42"));
        assert!(result.contains("@const_var = constant double double = 3.14"));
        assert!(result.contains("@uninit_var = global i8* i8*"));
    }

    #[test]
    fn test_phi_nodes() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        let values = vec![
            ("1".to_string(), "bb1".to_string()),
            ("2".to_string(), "bb2".to_string()),
        ];
        
        let result = builder.add_phi("i32", &values, "phi_var");
        assert_eq!(result, "%phi_var");
        
        let result = builder.build();
        assert!(result.contains("%phi_var = phi i32 [ 1, %bb1 ], [ 2, %bb2 ]"));
    }

    #[test]
    fn test_getelementptr() {
        let mut builder = LLVMIRBuilder::new();
        builder.add_main_function();
        
        let indices = vec!["0".to_string(), "1".to_string()];
        let result = builder.add_getelementptr("%array", &indices, "elem");
        assert_eq!(result, "%elem");
        
        let result = builder.build();
        assert!(result.contains("%elem = getelementptr inbounds"));
    }

    #[test]
    fn test_temp_counter() {
        let mut builder = LLVMIRBuilder::new();
        
        assert_eq!(builder.next_temp(), "%t1");
        assert_eq!(builder.next_temp(), "%t2");
        assert_eq!(builder.next_temp(), "%t3");
    }

    #[test]
    fn test_global_counter() {
        let mut builder = LLVMIRBuilder::new();
        
        assert_eq!(builder.next_global(), "@g1");
        assert_eq!(builder.next_global(), "@g2");
        assert_eq!(builder.next_global(), "@g3");
    }

    #[test]
    fn test_block_counter() {
        let mut builder = LLVMIRBuilder::new();
        
        assert_eq!(builder.next_block(), "bb1");
        assert_eq!(builder.next_block(), "bb2");
        assert_eq!(builder.next_block(), "bb3");
    }

    #[test]
    fn test_string_escaping() {
        let mut builder = LLVMIRBuilder::new();
        
        // Test various escape sequences
        let test_cases = vec![
            ("Hello\nWorld", "Hello\\0AWorld"),
            ("Tab\there", "Tab\\09here"),
            ("Quote\"here", "Quote\\22here"),
            ("Back\\slash", "Back\\5Cslash"),
        ];
        
        for (input, expected) in test_cases {
            let escaped = builder.escape_string(input);
            assert_eq!(escaped, expected);
        }
    }

    #[test]
    fn test_complex_program() {
        let mut builder = LLVMIRBuilder::new();
        
        // Module header
        builder.add_module_header("complex_test");
        
        // Function declarations
        builder.declare_function("printf", "i32", &["i8*".to_string()], true);
        builder.declare_function("malloc", "i8*", &["i64".to_string()], false);
        
        // String constants
        builder.add_string_constant("fmt", "Hello, %d!\n");
        
        // Main function
        builder.add_main_function();
        
        // Allocate variables
        builder.add_alloca("i32", "%x");
        builder.add_alloca("double", "%y");
        
        // Store values
        builder.add_store("42", "%x");
        builder.add_store("3.14", "%y");
        
        // Load and use values
        let x_val = builder.add_load("i32", "%x", "x_val");
        let _y_val = builder.add_load("double", "%y", "y_val");
        
        // Convert to double for printf
        let x_double = builder.add_sitofp(&x_val, "double", "x_double");
        
        // Call printf
        builder.add_call("printf", &["%fmt".to_string(), x_double], "i32", "ret");
        
        // Return
        builder.add_return(Some("0"));
        builder.end_function();
        
        let result = builder.build();
        
        // Verify key components
        assert!(result.contains("define i32 @main()"));
        assert!(result.contains("store i32 42, i32* %x"));
        assert!(result.contains("store double 3.14, double* %y"));
        assert!(result.contains("call i32 @printf"));
        assert!(result.contains("ret i32 0"));
    }
} 