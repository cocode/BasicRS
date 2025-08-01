use std::env;
use std::fs;
use std::path::Path;
use std::io::Write;

fn main() {
    println!("cargo:rerun-if-changed=test_suite");
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    // Find all .bas files in test_suite directory
    let test_suite_dir = Path::new("test_suite");
    let mut basic_programs = Vec::new();
    
    if let Ok(entries) = fs::read_dir(test_suite_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("bas") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        basic_programs.push((name.to_string(), path.file_name().unwrap().to_str().unwrap().to_string()));
                    }
                }
            }
        }
    }
    
    basic_programs.sort();

    // Generate the test functions
    writeln!(f, "// This file is automatically generated by build.rs").unwrap();
    writeln!(f, "use std::process::Command;").unwrap();
    writeln!(f, "use std::path::Path;").unwrap();
    writeln!(f, "use std::time::Duration;").unwrap();
    writeln!(f, "use wait_timeout::ChildExt;").unwrap();
    writeln!(f, "use std::fs;").unwrap();
    writeln!(f, "").unwrap();

    // Helper functions
    writeln!(f, "const TEST_TIMEOUT_SECS: u64 = 30;").unwrap();
    writeln!(f, "").unwrap();
    
    writeln!(f, "fn get_expected_exit_code(program_path: &Path) -> i32 {{").unwrap();
    writeln!(f, "    if let Ok(content) = fs::read_to_string(program_path) {{").unwrap();
    writeln!(f, "        if let Some(first_line) = content.lines().next() {{").unwrap();
    writeln!(f, "            if let Some(pos) = first_line.find(\"@EXPECT_EXIT_CODE\") {{").unwrap();
    writeln!(f, "                let after = &first_line[pos + \"@EXPECT_EXIT_CODE=\".len()..];").unwrap();
    writeln!(f, "                if let Ok(code) = after.trim().parse() {{").unwrap();
    writeln!(f, "                    return code;").unwrap();
    writeln!(f, "                }}").unwrap();
    writeln!(f, "            }}").unwrap();
    writeln!(f, "        }}").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "    0 // Default to 0 (success)").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f, "").unwrap();

    writeln!(f, "fn run_basic_test(file_name: &str) -> Result<(), String> {{").unwrap();
    writeln!(f, "    let test_suite_dir = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"test_suite\");").unwrap();
    writeln!(f, "    let program_path = test_suite_dir.join(file_name);").unwrap();
    writeln!(f, "    ").unwrap();
    writeln!(f, "    if !program_path.exists() {{").unwrap();
    writeln!(f, "        return Err(format!(\"Test file {{}} not found\", file_name));").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "    ").unwrap();
    writeln!(f, "    let expected_exit_code = get_expected_exit_code(&program_path);").unwrap();
    writeln!(f, "    let mut command = Command::new(env!(\"CARGO_BIN_EXE_basic_rs\"));").unwrap();
    writeln!(f, "    command.arg(&program_path);").unwrap();
    writeln!(f, "    ").unwrap();
    writeln!(f, "    match command.spawn() {{").unwrap();
    writeln!(f, "        Ok(mut child) => {{").unwrap();
    writeln!(f, "            match child.wait_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {{").unwrap();
    writeln!(f, "                Ok(Some(status)) => {{").unwrap();
    writeln!(f, "                    let actual_exit_code = status.code().unwrap_or(-1);").unwrap();
    writeln!(f, "                    if actual_exit_code == expected_exit_code {{").unwrap();
    writeln!(f, "                        Ok(())").unwrap();
    writeln!(f, "                    }} else {{").unwrap();
    writeln!(f, "                        Err(format!(\"Expected exit code: {{}}, got: {{}}\", expected_exit_code, actual_exit_code))").unwrap();
    writeln!(f, "                    }}").unwrap();
    writeln!(f, "                }}").unwrap();
    writeln!(f, "                Ok(None) => {{").unwrap();
    writeln!(f, "                    let _ = child.kill();").unwrap();
    writeln!(f, "                    Err(format!(\"Test timed out after {{}} seconds\", TEST_TIMEOUT_SECS))").unwrap();
    writeln!(f, "                }}").unwrap();
    writeln!(f, "                Err(e) => Err(format!(\"Error waiting for process: {{}}\", e)),").unwrap();
    writeln!(f, "            }}").unwrap();
    writeln!(f, "        }}").unwrap();
    writeln!(f, "        Err(e) => Err(format!(\"Failed to spawn process: {{}}\", e)),").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f, "").unwrap();

    // Generate individual test functions
    for (test_name, file_name) in basic_programs {
        // Convert file name to valid Rust identifier
        let rust_test_name = test_name.replace("-", "_").replace(".", "_");
        
        writeln!(f, "#[test]").unwrap();
        writeln!(f, "fn test_basic_{}() {{", rust_test_name).unwrap();
        writeln!(f, "    match run_basic_test(\"{}\") {{", file_name).unwrap();
        writeln!(f, "        Ok(()) => {{}}, // Test passed").unwrap();
        writeln!(f, "        Err(error) => panic!(\"BASIC test failed: {{}}\", error),").unwrap();
        writeln!(f, "    }}").unwrap();
        writeln!(f, "}}").unwrap();
        writeln!(f, "").unwrap();
    }
} 