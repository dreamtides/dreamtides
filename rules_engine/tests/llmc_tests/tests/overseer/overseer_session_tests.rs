use llmc::overseer_mode::overseer_session::{OVERSEER_SESSION_NAME, is_overseer_session};

#[test]
fn overseer_session_name_is_llmc_overseer() {
    assert_eq!(
        OVERSEER_SESSION_NAME, "llmc-overseer",
        "Overseer session name should be 'llmc-overseer'"
    );
}

#[test]
fn is_overseer_session_returns_true_for_overseer_name() {
    let result = is_overseer_session("llmc-overseer");

    assert!(result, "Should return true for the overseer session name");
}

#[test]
fn is_overseer_session_returns_false_for_worker_sessions() {
    let worker_sessions = ["llmc-adam", "llmc-baker", "llmc-auto-1", "llmc-console1"];

    for session in worker_sessions {
        let result = is_overseer_session(session);
        assert!(!result, "Should return false for worker session '{}', but got true", session);
    }
}

#[test]
fn is_overseer_session_returns_false_for_similar_names() {
    let similar_names =
        ["llmc-overseer1", "llmc-overseer-backup", "overseer", "LLMC-OVERSEER", "llmc_overseer"];

    for name in similar_names {
        let result = is_overseer_session(name);
        assert!(!result, "Should return false for similar name '{}', but got true", name);
    }
}

#[test]
fn is_overseer_session_returns_false_for_empty_string() {
    let result = is_overseer_session("");

    assert!(!result, "Should return false for empty string");
}
