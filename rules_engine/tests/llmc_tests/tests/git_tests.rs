use llmc::git::strip_agent_attribution;

#[test]
fn test_strip_agent_attribution() {
    let message = "Fix bug\n\nCo-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>";
    assert_eq!(strip_agent_attribution(message), "Fix bug");
    let message2 = "Add feature\n\n🤖 Generated with [Claude Code]";
    assert_eq!(strip_agent_attribution(message2), "Add feature");
    let message3 = "Simple commit";
    assert_eq!(strip_agent_attribution(message3), "Simple commit");
}
