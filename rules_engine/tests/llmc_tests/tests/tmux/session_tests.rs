use llmc::tmux::session::any_llmc_sessions_running;

#[test]
fn test_any_llmc_sessions_running() {
    let result = any_llmc_sessions_running();
    assert!(result.is_ok());
}
