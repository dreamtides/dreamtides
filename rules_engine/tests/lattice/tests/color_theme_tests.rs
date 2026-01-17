use lattice::cli::color_theme;
use owo_colors::Style;

#[test]
fn test_success_formats_value() {
    let result = format!("{}", color_theme::success("test message"));
    assert!(result.contains("test message"), "Expected formatted output to contain the value");
}

#[test]
fn test_warning_formats_value() {
    let result = format!("{}", color_theme::warning("warning text"));
    assert!(result.contains("warning text"), "Expected formatted output to contain the value");
}

#[test]
fn test_error_formats_value() {
    let result = format!("{}", color_theme::error("error text"));
    assert!(result.contains("error text"), "Expected formatted output to contain the value");
}

#[test]
fn test_accent_formats_value() {
    let result = format!("{}", color_theme::accent("accent text"));
    assert!(result.contains("accent text"), "Expected formatted output to contain the value");
}

#[test]
fn test_muted_formats_value() {
    let result = format!("{}", color_theme::muted("muted text"));
    assert!(result.contains("muted text"), "Expected formatted output to contain the value");
}

#[test]
fn test_dim_formats_value() {
    let result = format!("{}", color_theme::dim("dim text"));
    assert!(result.contains("dim text"), "Expected formatted output to contain the value");
}

#[test]
fn test_special_formats_value() {
    let result = format!("{}", color_theme::special("special text"));
    assert!(result.contains("special text"), "Expected formatted output to contain the value");
}

#[test]
fn test_bold_formats_value() {
    let result = format!("{}", color_theme::bold("bold text"));
    assert!(result.contains("bold text"), "Expected formatted output to contain the value");
}

#[test]
fn test_lattice_id_formats_value() {
    let result = format!("{}", color_theme::lattice_id("LXXXXX"));
    assert!(result.contains("LXXXXX"), "Expected Lattice ID in output");
}

#[test]
fn test_task_type_formats_value() {
    let result = format!("{}", color_theme::task_type("bug"));
    assert!(result.contains("bug"), "Expected task type in output");
}

#[test]
fn test_priority_formats_value() {
    let result = format!("{}", color_theme::priority("P1"));
    assert!(result.contains("P1"), "Expected priority in output");
}

#[test]
fn test_path_formats_value() {
    let result = format!("{}", color_theme::path("src/main.rs"));
    assert!(result.contains("src/main.rs"), "Expected path in output");
}

#[test]
fn test_label_formats_value() {
    let result = format!("{}", color_theme::label("urgent"));
    assert!(result.contains("urgent"), "Expected label in output");
}

#[test]
fn test_status_open_formats_value() {
    let result = format!("{}", color_theme::status_open("open"));
    assert!(result.contains("open"), "Expected status in output");
}

#[test]
fn test_status_blocked_formats_value() {
    let result = format!("{}", color_theme::status_blocked("blocked"));
    assert!(result.contains("blocked"), "Expected status in output");
}

#[test]
fn test_status_closed_formats_value() {
    let result = format!("{}", color_theme::status_closed("closed"));
    assert!(result.contains("closed"), "Expected status in output");
}

#[test]
fn test_styled_with_custom_color() {
    let result = format!("{}", color_theme::styled("custom", color_theme::AYU_ACCENT));
    assert!(result.contains("custom"), "Expected custom text in output");
}

#[test]
fn test_styled_with_custom_style() {
    let style = Style::new().bold().underline();
    let result = format!("{}", color_theme::styled_with("styled text", style));
    assert!(result.contains("styled text"), "Expected styled text in output");
}

#[test]
fn test_formatting_preserves_numeric_values() {
    let result = format!("{}", color_theme::priority(42));
    assert!(result.contains("42"), "Expected numeric value in output");
}

#[test]
fn test_colors_disabled_in_non_tty_environment() {
    assert!(
        !color_theme::colors_enabled(),
        "Colors should be disabled when stdout is not a terminal"
    );
}

#[test]
fn test_formatting_without_colors_produces_plain_text() {
    let result = format!("{}", color_theme::success("plain"));
    assert_eq!(result, "plain", "Non-TTY output should be plain text without ANSI codes");
}
