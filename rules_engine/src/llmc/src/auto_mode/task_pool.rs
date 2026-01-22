#![allow(dead_code, reason = "TaskPoolError fields stored for diagnostic purposes")]

use std::process::Command;
use std::time::Instant;

use tracing::{debug, error, info};

use crate::auto_mode::auto_logging::{AutoLogger, CommandResult};

/// Result of executing the task pool command.
#[derive(Debug, Clone)]
pub enum TaskPoolResult {
    /// A task is available. Contains the task description from stdout.
    Task(String),
    /// No tasks are available (exit 0 with empty stdout).
    NoTasksAvailable,
    /// An error occurred (non-zero exit or execution failure).
    Error(TaskPoolError),
}

/// Error details from task pool command execution.
#[derive(Debug, Clone)]
pub struct TaskPoolError {
    pub message: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Executes the task pool command and returns the result.
///
/// The command is executed using the system shell (sh -c on Unix) to ensure
/// proper PATH resolution and shell environment.
///
/// Exit code interpretation:
/// - Exit 0 + non-empty stdout = Task available (stdout is the task)
/// - Exit 0 + empty stdout = No tasks available (not an error)
/// - Exit 3 (E039 - claim limit exceeded) = Error condition (claim leak bug)
/// - Exit 4 (E038 - no ready tasks) = No tasks available (not an error)
/// - Other non-zero exit = Error condition (triggers daemon shutdown)
pub fn execute_task_pool_command(command: &str, logger: &AutoLogger) -> TaskPoolResult {
    info!(command = %command, "Executing task pool command");
    let start = Instant::now();
    let output = Command::new("sh").arg("-c").arg(command).output();
    let duration = start.elapsed();
    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            let cmd_result = CommandResult {
                command: command.to_string(),
                exit_code,
                duration,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            };
            logger.log_task_pool(&cmd_result);
            if output.status.success() {
                handle_successful_execution(command, &stdout, &stderr, duration)
            } else {
                handle_failed_execution(command, exit_code, &stdout, &stderr)
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to execute task pool command: {}", e);
            error!(command = %command, error = %e, "Task pool command execution failed");
            let cmd_result = CommandResult {
                command: command.to_string(),
                exit_code: -1,
                duration,
                stdout: String::new(),
                stderr: error_msg.clone(),
            };
            logger.log_task_pool(&cmd_result);
            TaskPoolResult::Error(TaskPoolError {
                message: error_msg,
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }
}

fn handle_successful_execution(
    command: &str,
    stdout: &str,
    stderr: &str,
    duration: std::time::Duration,
) -> TaskPoolResult {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        debug!(
            command = %command,
            duration_ms = %duration.as_millis(),
            "Task pool command returned no tasks"
        );
        TaskPoolResult::NoTasksAvailable
    } else {
        info!(
            command = %command,
            duration_ms = %duration.as_millis(),
            task_length = trimmed.len(),
            "Task pool command returned a task"
        );
        if !stderr.trim().is_empty() {
            debug!(stderr = %stderr.trim(), "Task pool command produced stderr (but succeeded)");
        }
        TaskPoolResult::Task(trimmed.to_string())
    }
}

fn handle_failed_execution(
    command: &str,
    exit_code: i32,
    stdout: &str,
    stderr: &str,
) -> TaskPoolResult {
    // Exit code 4 (E038 - no ready tasks) is not an error - it just means no
    // tasks are available right now.
    //
    // Note: Exit code 3 (E039 - claim limit exceeded) IS treated as an error
    // because it indicates workers are not properly releasing claims, which is
    // a bug that the overseer should investigate.
    if exit_code == 4 {
        debug!(
            command = %command,
            exit_code = exit_code,
            "Task pool command returned no tasks (expected exit code)"
        );
        return TaskPoolResult::NoTasksAvailable;
    }

    error!(
        command = %command,
        exit_code = exit_code,
        stdout = %stdout.trim(),
        stderr = %stderr.trim(),
        "Task pool command failed with non-zero exit code"
    );
    let message = if stderr.trim().is_empty() {
        format!("Task pool command '{}' failed with exit code {}", command, exit_code)
    } else {
        format!(
            "Task pool command '{}' failed with exit code {}: {}",
            command,
            exit_code,
            stderr.trim()
        )
    };
    TaskPoolResult::Error(TaskPoolError {
        message,
        exit_code: Some(exit_code),
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    })
}

impl std::fmt::Display for TaskPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TaskPoolError {}
