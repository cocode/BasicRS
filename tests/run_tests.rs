use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use wait_timeout::ChildExt;

const TEST_TIMEOUT_SECS: u64 = 30;

fn find_basic_programs(test_suite_dir: &Path) -> Vec<PathBuf> {
    let mut programs = Vec::new();
    if let Ok(entries) = fs::read_dir(test_suite_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("bas") {
                    programs.push(path);
                }
            }
        }
    }
    programs.sort();
    programs
}

fn get_expected_exit_code(program_path: &Path) -> i32 {
    if let Ok(content) = fs::read_to_string(program_path) {
        if let Some(first_line) = content.lines().next() {
            if let Some(pos) = first_line.find("@EXPECT_EXIT_CODE") {
                if let Ok(code) = first_line[..pos].trim().parse() {
                    return code;
                }
            }
        }
    }
    0 // Default to 0 (success)
}

fn run_test_with_command(command: &mut Command, expected_exit_code: i32) -> Result<(), String> {
    match command.spawn() {
        Ok(mut child) => {
            match child.wait_timeout(Duration::from_secs(TEST_TIMEOUT_SECS)) {
                Ok(Some(status)) => {
                    let actual_exit_code = status.code().unwrap_or(-1);
                    if actual_exit_code == expected_exit_code {
                        Ok(())
                    } else {
                        Err(format!(
                            "Expected exit code: {}, got: {}",
                            expected_exit_code, actual_exit_code
                        ))
                    }
                }
                Ok(None) => {
                    // Test timed out, kill the process
                    let _ = child.kill();
                    Err(format!("Test timed out after {} seconds", TEST_TIMEOUT_SECS))
                }
                Err(e) => Err(format!("Error waiting for process: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to spawn process: {}", e)),
    }
}

fn run_test_suite(test_suite_dir: &Path) -> bool {
    println!("Running BASIC test suite...");
    println!("==========================");

    let programs = find_basic_programs(test_suite_dir);
    if programs.is_empty() {
        println!("No BASIC programs found!");
        return false;
    }

    let mut passed = 0;
    let mut failed = 0;

    for program_path in programs {
        let program_name = program_path.file_name()
            .unwrap_or_default()
            .to_string_lossy();
        
        print!("Testing {}... ", program_name);
        
        let expected_exit_code = get_expected_exit_code(&program_path);

        let mut command = Command::new(env!("CARGO_BIN_EXE_basic_rs"));

        command.arg(&program_path);
        
        match run_test_with_command(&mut command, expected_exit_code) {
            Ok(()) => {
                println!("PASS");
                passed += 1;
            }
            Err(error) => {
                println!("FAIL");
                println!("  {}", error);
                failed += 1;
            }
        }
    }

    println!("==========================");
    println!("Results: {} passed, {} failed", passed, failed);

    failed == 0
}

#[test]
fn run_all_tests() {
    let test_suite_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_suite");
    
    assert!(
        run_test_suite(&test_suite_dir),
        "Some tests failed"
    );
} 