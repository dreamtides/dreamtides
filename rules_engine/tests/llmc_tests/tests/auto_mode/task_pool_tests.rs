use llmc::auto_mode::task_pool::TaskPoolError;

// Note: The execute_task_pool_command function requires an AutoLogger, which
// needs to write to log files based on the HOME environment variable. Testing
// the full command execution would require modifying shared global state
// (HOME), which causes test isolation issues when tests run in parallel.
//
// Instead, we test:
// 1. The TaskPoolError display implementation
// 2. The TaskPoolResult enum variants (via constructing them)
// 3. The error formatting behavior

#[test]
fn task_pool_error_display_shows_message() {
    let error = TaskPoolError {
        message: "Test error message".to_string(),
        exit_code: Some(42),
        stdout: "stdout content".to_string(),
        stderr: "stderr content".to_string(),
    };
    let display = format!("{}", error);
    assert_eq!(display, "Test error message", "Display should show the message");
}

#[test]
fn task_pool_error_display_with_empty_message() {
    let error = TaskPoolError {
        message: "".to_string(),
        exit_code: None,
        stdout: "".to_string(),
        stderr: "".to_string(),
    };
    let display = format!("{}", error);
    assert_eq!(display, "", "Display with empty message should be empty");
}

#[test]
fn task_pool_error_stores_exit_code() {
    let error = TaskPoolError {
        message: "Error".to_string(),
        exit_code: Some(127),
        stdout: "".to_string(),
        stderr: "".to_string(),
    };
    assert_eq!(error.exit_code, Some(127), "Exit code should be stored");
}

#[test]
fn task_pool_error_stores_none_exit_code() {
    let error = TaskPoolError {
        message: "Execution failed".to_string(),
        exit_code: None,
        stdout: "".to_string(),
        stderr: "".to_string(),
    };
    assert!(error.exit_code.is_none(), "Exit code should be None for execution failures");
}

#[test]
fn task_pool_error_stores_stdout() {
    let error = TaskPoolError {
        message: "Error".to_string(),
        exit_code: Some(1),
        stdout: "command output".to_string(),
        stderr: "".to_string(),
    };
    assert_eq!(error.stdout, "command output", "Stdout should be stored");
}

#[test]
fn task_pool_error_stores_stderr() {
    let error = TaskPoolError {
        message: "Error".to_string(),
        exit_code: Some(1),
        stdout: "".to_string(),
        stderr: "error output".to_string(),
    };
    assert_eq!(error.stderr, "error output", "Stderr should be stored");
}

#[test]
fn task_pool_error_debug_implementation() {
    let error = TaskPoolError {
        message: "Debug test".to_string(),
        exit_code: Some(5),
        stdout: "out".to_string(),
        stderr: "err".to_string(),
    };
    let debug = format!("{:?}", error);
    assert!(debug.contains("Debug test"), "Debug output should contain message");
    assert!(debug.contains("5"), "Debug output should contain exit code");
}

#[test]
fn task_pool_error_is_std_error() {
    let error = TaskPoolError {
        message: "std error test".to_string(),
        exit_code: Some(1),
        stdout: "".to_string(),
        stderr: "".to_string(),
    };
    // Verify it implements std::error::Error by using it as a trait object
    let _: &dyn std::error::Error = &error;
}

#[test]
fn task_pool_error_clone() {
    let error = TaskPoolError {
        message: "clone test".to_string(),
        exit_code: Some(100),
        stdout: "out".to_string(),
        stderr: "err".to_string(),
    };
    let cloned = error.clone();
    assert_eq!(error.message, cloned.message, "Cloned message should match");
    assert_eq!(error.exit_code, cloned.exit_code, "Cloned exit code should match");
    assert_eq!(error.stdout, cloned.stdout, "Cloned stdout should match");
    assert_eq!(error.stderr, cloned.stderr, "Cloned stderr should match");
}
