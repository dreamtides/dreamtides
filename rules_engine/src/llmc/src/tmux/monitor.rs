#![allow(dead_code)]

use std::sync::OnceLock;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use regex::Regex;
use tmux_interface::{DisplayMessage, Tmux};

use super::sender::TmuxSender;
use super::session;

/// Claude's operational state
#[derive(Debug, Clone, PartialEq)]
pub enum ClaudeState {
    Ready,
    Processing,
    AwaitingQuestion { question_type: QuestionType },
    AwaitingPermission { tool_name: String },
    Error { error_type: ErrorType, recoverable: bool },
    Exited { exit_type: ExitType },
    Unknown,
}

/// Type of question Claude is asking
#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    MultipleChoice,
    YesNo,
    FreeText,
}

/// Type of exit that occurred
#[derive(Debug, Clone, PartialEq)]
pub enum ExitType {
    UserInitiated,
    Crash,
    Timeout,
    Unknown,
}

/// Type of error encountered
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    RateLimit,
    Network,
    ToolError,
    ApiError,
}

/// Output event types for monitoring
#[derive(Debug, Clone, PartialEq)]
pub enum OutputEvent {
    Committed(String),
    CompletedNoCommit,
    AskingQuestion,
    NeedsConfirmation,
    Crashed,
}

/// Detects Claude's state from terminal output
pub struct StateDetector {
    sender: TmuxSender,
}

/// Monitors terminal output for specific events
pub struct OutputMonitor {
    sender: TmuxSender,
}

/// Process health status
#[derive(Debug, Clone, PartialEq)]
enum ProcessHealth {
    Running,
    Exited,
    Unknown,
}

/// Ready state classification
#[derive(Debug, Clone, PartialEq)]
enum ReadyStateType {
    Completed,
    WaitingForInput,
}

impl StateDetector {
    pub fn new(sender: TmuxSender) -> Self {
        Self { sender }
    }

    /// Detects the current state of Claude in the given session
    pub fn detect(&self, session: &str) -> Result<ClaudeState> {
        let health = check_process_health(session)?;
        if health == ProcessHealth::Exited {
            let output = session::capture_pane(session, 50)?;
            return Ok(ClaudeState::Exited { exit_type: classify_exit(session, &output)? });
        }
        let output = session::capture_pane(session, 50)?;
        if let Some((error_type, recoverable)) = detect_error_state(&output) {
            return Ok(ClaudeState::Error { error_type, recoverable });
        }
        if let Some(tool_name) = detect_permission_prompt(&output) {
            return Ok(ClaudeState::AwaitingPermission { tool_name });
        }
        if let Some(question_type) = detect_ask_user_question(&output) {
            return Ok(ClaudeState::AwaitingQuestion { question_type });
        }
        if is_ready_for_input(&output) {
            return Ok(match classify_ready_state(&output) {
                ReadyStateType::Completed => ClaudeState::Ready,
                ReadyStateType::WaitingForInput => {
                    ClaudeState::AwaitingQuestion { question_type: QuestionType::FreeText }
                }
            });
        }
        if health == ProcessHealth::Running {
            return Ok(ClaudeState::Processing);
        }
        Ok(ClaudeState::Unknown)
    }

    /// Accepts the bypass permissions warning dialog
    pub fn accept_bypass_warning(&self, session: &str) -> Result<()> {
        sleep(Duration::from_millis(1000));
        let output = session::capture_pane(session, 30)?;
        if !output.contains("Bypass Permissions mode") {
            return Ok(());
        }
        self.sender.send_keys_raw(session, "Down")?;
        sleep(Duration::from_millis(200));
        self.sender.send_keys_raw(session, "Enter")?;
        Ok(())
    }
}

impl OutputMonitor {
    pub fn new(sender: TmuxSender) -> Self {
        Self { sender }
    }

    /// Waits for and detects output events
    pub fn wait_for_event(&self, session: &str) -> Result<OutputEvent> {
        let detector = StateDetector::new(self.sender.clone());
        loop {
            let state = detector.detect(session)?;
            match state {
                ClaudeState::Ready => {
                    return Ok(OutputEvent::CompletedNoCommit);
                }
                ClaudeState::AwaitingQuestion { .. } => {
                    return Ok(OutputEvent::AskingQuestion);
                }
                ClaudeState::AwaitingPermission { .. } => {
                    return Ok(OutputEvent::NeedsConfirmation);
                }
                ClaudeState::Exited { exit_type } => {
                    if matches!(exit_type, ExitType::Crash) {
                        return Ok(OutputEvent::Crashed);
                    }
                    return Ok(OutputEvent::CompletedNoCommit);
                }
                ClaudeState::Processing | ClaudeState::Error { .. } | ClaudeState::Unknown => {
                    sleep(Duration::from_millis(500));
                }
            }
        }
    }
}

fn check_process_health(session: &str) -> Result<ProcessHealth> {
    let cmd = session::get_pane_command(session)?;
    if session::is_claude_process(&cmd) {
        Ok(ProcessHealth::Running)
    } else if session::is_shell(&cmd) {
        Ok(ProcessHealth::Exited)
    } else {
        Ok(ProcessHealth::Unknown)
    }
}

fn is_ready_for_input(pane_output: &str) -> bool {
    pane_output
        .lines()
        .rev()
        .take(5)
        .any(|line| matches!(line.trim_start(), s if s.starts_with("> ") || s == ">"))
}

fn detect_ask_user_question(output: &str) -> Option<QuestionType> {
    let recent_lines: Vec<&str> = output.lines().rev().take(20).collect();
    let recent_text = recent_lines.join("\n");
    static NUMBERED_OPTION_REGEX: OnceLock<Regex> = OnceLock::new();
    let has_numbered_options = NUMBERED_OPTION_REGEX
        .get_or_init(|| Regex::new(r"(?m)^\s*\d+[.)]").unwrap())
        .is_match(&recent_text);
    let has_question_marker = recent_lines.iter().any(|line| line.trim_start().starts_with("?"));
    let has_nav_hint = recent_text.contains("arrow keys")
        || recent_text.contains("Enter to select")
        || recent_text.contains("navigate");
    let has_selection_indicators =
        recent_text.contains("[ ]") || recent_text.contains("[x]") || recent_text.contains("( )");
    if has_numbered_options && (has_question_marker || has_nav_hint) {
        return Some(QuestionType::MultipleChoice);
    }
    if has_selection_indicators {
        return Some(QuestionType::YesNo);
    }
    None
}

fn detect_permission_prompt(output: &str) -> Option<String> {
    let recent_text = get_recent_lines(output, 30);
    static WANTS_TO_REGEX: OnceLock<Regex> = OnceLock::new();
    if let Some(captures) = WANTS_TO_REGEX
        .get_or_init(|| {
            Regex::new(r"Claude wants to (?:run|use|access|execute):?\s*(\w+)").unwrap()
        })
        .captures(&recent_text)
        && let Some(tool_name) = captures.get(1)
    {
        return Some(tool_name.as_str().to_string());
    }
    if recent_text.contains("Allow this action")
        || recent_text.contains("Allow this command")
        || recent_text.contains("Allow this tool")
    {
        return extract_tool_from_context(&recent_text);
    }
    if recent_text.contains("╭")
        && recent_text.contains("╯")
        && (recent_text.contains("wants to run") || recent_text.contains("wants to use"))
    {
        return extract_tool_from_context(&recent_text);
    }
    None
}

fn extract_tool_from_context(text: &str) -> Option<String> {
    static TOOL_PATTERN: OnceLock<Regex> = OnceLock::new();
    if let Some(caps) =
        TOOL_PATTERN.get_or_init(|| Regex::new(r"Tool:\s*(\w+)").unwrap()).captures(text)
        && let Some(m) = caps.get(1)
    {
        return Some(m.as_str().to_string());
    }
    static BASH_PATTERN: OnceLock<Regex> = OnceLock::new();
    if BASH_PATTERN.get_or_init(|| Regex::new(r"bash.*?Command:").unwrap()).is_match(text) {
        return Some("bash".to_string());
    }
    static TOOL_NAMES_PATTERN: OnceLock<Regex> = OnceLock::new();
    if let Some(caps) = TOOL_NAMES_PATTERN
        .get_or_init(|| Regex::new(r"(Read|Write|Edit|Glob|Grep|Bash)").unwrap())
        .captures(text)
        && let Some(m) = caps.get(1)
    {
        return Some(m.as_str().to_string());
    }
    Some("unknown".to_string())
}

fn classify_ready_state(output: &str) -> ReadyStateType {
    let recent_text = get_recent_lines(output, 50);
    if has_pending_question(&recent_text) {
        return ReadyStateType::WaitingForInput;
    }
    if has_completion_indicators(&recent_text) {
        return ReadyStateType::Completed;
    }
    if has_needs_attention_indicators(&recent_text) {
        return ReadyStateType::WaitingForInput;
    }
    ReadyStateType::Completed
}

fn has_pending_question(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let last_line = lines.last().map(|s| s.trim()).unwrap_or("");
    let has_prompt = last_line.starts_with("> ") || last_line == ">";
    let skip_count = if has_prompt { 1 } else { 0 };
    let question_at_end = text
        .lines()
        .rev()
        .skip(skip_count)
        .take(5)
        .any(|line| matches!(line.trim(), s if ! s.is_empty() && s.ends_with('?')));
    let explicit_request = [
        "please let me know",
        "please tell me",
        "could you clarify",
        "would you like me to",
        "should I",
        "do you want me to",
        "what would you prefer",
        "which option",
        "please confirm",
    ]
    .iter()
    .any(|phrase| text.to_lowercase().contains(phrase));
    let trailing_ellipsis =
        text.lines().rev().skip(skip_count).take(3).any(|line| line.trim().ends_with("..."));
    question_at_end || explicit_request || trailing_ellipsis
}

fn has_completion_indicators(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "created commit",
        "committed",
        "[master",
        "[main",
        "is now complete",
        "has been completed",
        "finished implementing",
        "done with the changes",
        "changes have been made",
        "successfully",
        "wrote to",
        "saved",
        "updated",
        "all tests pass",
        "build succeeded",
        "no errors",
    ]
    .iter()
    .any(|ind| lower.contains(ind))
}

fn has_needs_attention_indicators(text: &str) -> bool {
    let lower = text.to_lowercase();
    [
        "error:",
        "failed:",
        "could not",
        "unable to",
        "blocked",
        "need clarification",
        "unclear",
        "ambiguous",
        "need your help",
        "need input",
        "waiting for",
    ]
    .iter()
    .any(|ind| lower.contains(ind))
}

fn classify_exit(session: &str, output: &str) -> Result<ExitType> {
    if let Some(exit_code) = get_pane_exit_code(session) {
        return Ok(match exit_code {
            0 => ExitType::UserInitiated,
            130 => ExitType::UserInitiated,
            137 => ExitType::Crash,
            _ => ExitType::Crash,
        });
    }
    let recent_text = get_recent_lines(output, 30);
    if recent_text.contains("/exit")
        || recent_text.contains("Goodbye")
        || recent_text.contains("exiting")
        || recent_text.contains("session ended")
    {
        return Ok(ExitType::UserInitiated);
    }
    if recent_text.contains("panic")
        || recent_text.contains("FATAL")
        || recent_text.contains("Error:")
        || recent_text.contains("Uncaught exception")
        || recent_text.contains("SIGABRT")
        || recent_text.contains("SIGSEGV")
    {
        return Ok(ExitType::Crash);
    }
    if recent_text.contains("timeout")
        || recent_text.contains("timed out")
        || recent_text.contains("connection lost")
    {
        return Ok(ExitType::Timeout);
    }
    Ok(ExitType::Unknown)
}

fn get_pane_exit_code(session: &str) -> Option<i32> {
    Tmux::with_command(
        DisplayMessage::new().target_pane(session).print().message("#{pane_dead_status}"),
    )
    .output()
    .ok()
    .and_then(|output| output.to_string().trim().parse().ok())
}

fn detect_error_state(output: &str) -> Option<(ErrorType, bool)> {
    let recent_text = get_recent_lines(output, 20);
    if recent_text.contains("rate limit")
        || recent_text.contains("429")
        || recent_text.contains("too many requests")
    {
        return Some((ErrorType::RateLimit, true));
    }
    if recent_text.contains("network error")
        || recent_text.contains("connection refused")
        || recent_text.contains("ECONNRESET")
        || recent_text.contains("fetch failed")
    {
        return Some((ErrorType::Network, true));
    }
    if recent_text.contains("API error")
        || recent_text.contains("500")
        || recent_text.contains("502")
        || recent_text.contains("503")
    {
        return Some((ErrorType::ApiError, true));
    }
    if recent_text.contains("tool error") || recent_text.contains("command failed") {
        return Some((ErrorType::ToolError, true));
    }
    None
}

fn get_recent_lines(output: &str, count: usize) -> String {
    output
        .lines()
        .rev()
        .take(count)
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .collect::<Vec<&str>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_ready_for_input() {
        assert!(is_ready_for_input("Some output\n> "));
        assert!(is_ready_for_input("Some output\n>"));
        assert!(is_ready_for_input("Multiple\nlines\nof\noutput\n> "));
        assert!(!is_ready_for_input("Some output\nNo prompt"));
        assert!(!is_ready_for_input("Processing..."));
    }
    #[test]
    fn test_detect_ask_user_question() {
        let multiple_choice = "? Which approach?\n  1) Option A\n  2) Option B\nUse arrow keys";
        assert_eq!(detect_ask_user_question(multiple_choice), Some(QuestionType::MultipleChoice));
        let yes_no = "Some text\n[ ] Yes\n[ ] No";
        assert_eq!(detect_ask_user_question(yes_no), Some(QuestionType::YesNo));
        let no_question = "Just regular output\nNo questions here";
        assert_eq!(detect_ask_user_question(no_question), None);
    }
    #[test]
    fn test_detect_permission_prompt() {
        let prompt1 = "Claude wants to run: bash\nCommand: git status";
        assert_eq!(detect_permission_prompt(prompt1), Some("bash".to_string()));
        let prompt2 = "╭──────────╮\nClaude wants to use: Read\n╰──────────╯";
        assert_eq!(detect_permission_prompt(prompt2), Some("Read".to_string()));
        let no_prompt = "Just regular output";
        assert_eq!(detect_permission_prompt(no_prompt), None);
    }
    #[test]
    fn test_has_completion_indicators() {
        assert!(has_completion_indicators("Successfully created commit abc123"));
        assert!(has_completion_indicators("Changes have been made to the file"));
        assert!(has_completion_indicators("All tests pass"));
        assert!(!has_completion_indicators("Still processing..."));
    }
    #[test]
    fn test_has_pending_question() {
        assert!(has_pending_question("Some output\nDo you want to continue?"));
        assert!(has_pending_question("Please let me know what to do"));
        assert!(has_pending_question("Waiting for input..."));
        assert!(!has_pending_question("Task completed successfully"));
    }
    #[test]
    fn test_detect_error_state() {
        assert_eq!(
            detect_error_state("Error: rate limit exceeded"),
            Some((ErrorType::RateLimit, true))
        );
        assert_eq!(
            detect_error_state("Network error: connection refused"),
            Some((ErrorType::Network, true))
        );
        assert_eq!(detect_error_state("API error 500"), Some((ErrorType::ApiError, true)));
        assert_eq!(detect_error_state("tool error occurred"), Some((ErrorType::ToolError, true)));
        assert_eq!(detect_error_state("Everything is fine"), None);
    }
    #[test]
    fn test_classify_ready_state() {
        let completed = "Task completed successfully\nAll tests pass\n> ";
        assert_eq!(classify_ready_state(completed), ReadyStateType::Completed);
        let waiting = "Should I continue with the next step?\n> ";
        assert_eq!(classify_ready_state(waiting), ReadyStateType::WaitingForInput);
        let error_attention = "Error: could not find file\n> ";
        assert_eq!(classify_ready_state(error_attention), ReadyStateType::WaitingForInput);
    }
    #[test]
    fn test_get_recent_lines() {
        let output = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        assert_eq!(get_recent_lines(output, 3), "Line 3\nLine 4\nLine 5");
        assert_eq!(get_recent_lines(output, 10), output);
        assert_eq!(get_recent_lines(output, 1), "Line 5");
    }
}
