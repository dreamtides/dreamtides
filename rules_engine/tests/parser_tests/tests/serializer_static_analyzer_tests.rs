use std::path::PathBuf;

const SERIALIZER_DIR: &str = "../../src/parser/src/serializer";
const BASELINE_PATH: &str =
    "tests/round_trip_tests/fixtures/serializer_static_analyzer_baseline.toml";

/// Files within the serializer directory to scan.
const SERIALIZER_FILES: &[&str] = &[
    "ability_serializer.rs",
    "condition_serializer.rs",
    "cost_serializer.rs",
    "effect_serializer.rs",
    "predicate_serializer.rs",
    "serializer_utils.rs",
    "static_ability_serializer.rs",
    "trigger_serializer.rs",
];

/// Legacy helper functions that must be fully eliminated from serializer code.
/// These functions bypass RLF and construct English text directly.
/// Each entry is a function name (without the opening paren). Matched using
/// word-boundary logic to avoid false positives from longer names that happen
/// to contain the pattern as a suffix.
const BANNED_LEGACY_HELPERS: &[&str] =
    &["text_phrase", "make_phrase", "make_phrase_non_vowel", "with_article", "phrase_plural"];

/// Patterns indicating English-specific grammar logic that should not exist
/// in language-neutral serializer code.
const BANNED_ENGLISH_GRAMMAR: &[&str] = &[
    "starts_with(['a','e','i','o','u'])",
    "starts_with(['a', 'e', 'i', 'o', 'u'])",
    r#"starts_with(&['a','e','i','o','u'])"#,
    r#"starts_with(&['a', 'e', 'i', 'o', 'u'])"#,
];

/// Maximum number of violations to report in output before truncating.
const MAX_REPORTED_VIOLATIONS: usize = 50;

#[derive(Debug)]
struct Violation {
    file: String,
    line_number: usize,
    rule: &'static str,
    line_text: String,
}

#[derive(Debug, serde::Deserialize)]
struct Baseline {
    max_allowed_legacy_helper_violations: usize,
    max_allowed_trim_end_period_violations: usize,
    max_allowed_hardcoded_english_violations: usize,
    max_allowed_english_grammar_violations: usize,
}

fn serializer_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SERIALIZER_DIR)
}

fn baseline_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(BASELINE_PATH)
}

fn read_serializer_file(filename: &str) -> String {
    let path = serializer_dir().join(filename);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read serializer file {}: {e}", path.display()))
}

/// Returns true if a line is a comment (starts with `//` after trimming).
fn is_comment_line(line: &str) -> bool {
    line.trim_start().starts_with("//")
}

/// Returns true if this line is inside a function signature (contains `fn `
/// as a definition, not a call).
fn is_fn_definition(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub(super) fn ")
        || trimmed.starts_with("pub(crate) fn ")
}

/// Returns true if `name` appears as a standalone function call in `line`.
/// A standalone call means the character immediately before the name (if any)
/// is not alphanumeric or underscore, and the character immediately after is
/// `(`. This prevents matching `card_predicate_base_phrase_plural` when
/// searching for `phrase_plural`.
fn contains_standalone_call(line: &str, name: &str) -> bool {
    let call_pattern = format!("{name}(");
    let mut search_start = 0;
    while let Some(pos) = line[search_start..].find(&call_pattern) {
        let abs_pos = search_start + pos;
        let before_ok = if abs_pos == 0 {
            true
        } else {
            let prev_char = line.as_bytes()[abs_pos - 1] as char;
            !prev_char.is_alphanumeric() && prev_char != '_'
        };
        if before_ok {
            return true;
        }
        search_start = abs_pos + 1;
    }
    false
}

/// Checks for banned legacy helper function calls.
fn check_legacy_helpers(filename: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        if is_comment_line(line) || is_fn_definition(line) {
            continue;
        }
        for pattern in BANNED_LEGACY_HELPERS {
            if contains_standalone_call(line, pattern) {
                violations.push(Violation {
                    file: filename.to_string(),
                    line_number: line_number + 1,
                    rule: "banned-legacy-helper",
                    line_text: line.trim().to_string(),
                });
            }
        }
    }
    violations
}

/// Checks for `trim_end_matches('.')` which indicates the effect fragment
/// convention has not been applied. Effects should return periodless fragments
/// and assembly code should add punctuation via RLF phrases.
fn check_trim_end_period(filename: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        if line.contains("trim_end_matches('.')")
            || line.contains("trim_end_matches(\".\")")
            || line.contains("ends_with('.')")
            || line.contains("ends_with(\".\")")
        {
            violations.push(Violation {
                file: filename.to_string(),
                line_number: line_number + 1,
                rule: "trim-end-period",
                line_text: line.trim().to_string(),
            });
        }
    }
    violations
}

/// Checks for hardcoded English string literals that bypass RLF. Detects
/// patterns like `"allies".to_string()` and `"{choose_one}".to_string()` and
/// `"Judgment".to_string()` where the string contains alphabetic text.
///
/// This rule intentionally allows:
/// - `strings::*` phrase calls (the correct pattern)
/// - `.to_string()` on phrase results
/// - `format!("{}.", ...)` (period-only format strings, tracked by
///   trim-end-period rule)
/// - Empty strings and pure-punctuation strings
/// - String literals used for variable names, error messages, etc.
fn check_hardcoded_english(filename: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        if is_comment_line(line) || is_fn_definition(line) {
            continue;
        }
        let trimmed = line.trim();
        // Detect `"some english text".to_string()` where the string contains
        // alphabetic characters and is not a format string placeholder.
        if let Some(violation) = check_hardcoded_english_to_string(trimmed) {
            violations.push(Violation {
                file: filename.to_string(),
                line_number: line_number + 1,
                rule: "hardcoded-english-literal",
                line_text: trimmed.to_string(),
            });
            let _ = violation;
        }
    }
    violations
}

/// Returns Some(()) if the line contains a hardcoded English string literal
/// being converted to a String value (not a format template, not a
/// strings:: phrase call).
fn check_hardcoded_english_to_string(line: &str) -> Option<()> {
    // Look for patterns like `"word".to_string()` where the string contains
    // alphabetic characters outside of curly braces.
    let pattern = ".to_string()";
    let mut search_start = 0;
    while let Some(to_string_pos) = line[search_start..].find(pattern) {
        let abs_pos = search_start + to_string_pos;
        // Walk backwards to find the opening quote
        if let Some(literal) = extract_string_literal_before(line, abs_pos) {
            if contains_bare_english_text(&literal) && !is_allowed_string_literal(&literal, line) {
                return Some(());
            }
        }
        search_start = abs_pos + pattern.len();
    }
    None
}

/// Extracts the string literal content immediately before the given position.
/// Returns None if no matching quoted string is found.
fn extract_string_literal_before(line: &str, before_pos: usize) -> Option<String> {
    let prefix = &line[..before_pos];
    let close_quote = prefix.rfind('"')?;
    if close_quote == 0 {
        return None;
    }
    // Find the opening quote, skipping escaped quotes
    let mut pos = close_quote - 1;
    loop {
        if line.as_bytes()[pos] == b'"' && (pos == 0 || line.as_bytes()[pos - 1] != b'\\') {
            return Some(line[pos + 1..close_quote].to_string());
        }
        if pos == 0 {
            break;
        }
        pos -= 1;
    }
    None
}

/// Returns true if the string contains alphabetic text that is NOT inside
/// curly braces (which would indicate an RLF template reference, not
/// hardcoded English).
fn contains_bare_english_text(s: &str) -> bool {
    let mut in_braces = 0usize;
    for ch in s.chars() {
        match ch {
            '{' => in_braces += 1,
            '}' => in_braces = in_braces.saturating_sub(1),
            c if c.is_alphabetic() && in_braces == 0 => return true,
            _ => {}
        }
    }
    false
}

/// Returns true if this string literal is allowed (not an English leakage
/// concern). Allows error messages, string format patterns, variable names,
/// pure punctuation/whitespace, and similar non-user-facing strings.
fn is_allowed_string_literal(literal: &str, line: &str) -> bool {
    // Allow literals used in error messages (panic!, format!, eprintln!)
    if line.contains("panic!(") || line.contains("eprintln!(") || line.contains("unreachable!(") {
        return true;
    }
    // Allow string interpolation format strings that only contain braces and
    // punctuation (e.g., `format!("{}{}", ...)`)
    if literal.chars().all(|c| !c.is_alphabetic()) {
        return true;
    }
    // Allow string literals in `const` or `static` declarations
    if line.trim_start().starts_with("const ") || line.trim_start().starts_with("static ") {
        return true;
    }
    // Allow `assert!`, `assert_eq!`, `debug_assert!` messages
    if line.contains("assert") {
        return true;
    }
    false
}

/// Checks for banned English grammar patterns in serializer code.
fn check_english_grammar(filename: &str, content: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        if is_comment_line(line) {
            continue;
        }
        for pattern in BANNED_ENGLISH_GRAMMAR {
            if line.contains(pattern) {
                violations.push(Violation {
                    file: filename.to_string(),
                    line_number: line_number + 1,
                    rule: "english-grammar-logic",
                    line_text: line.trim().to_string(),
                });
            }
        }
    }
    violations
}

fn format_violations(violations: &[Violation]) -> String {
    let mut output = Vec::new();
    for v in violations.iter().take(MAX_REPORTED_VIOLATIONS) {
        output.push(format!("  {}:{}: [{}] {}", v.file, v.line_number, v.rule, v.line_text));
    }
    if violations.len() > MAX_REPORTED_VIOLATIONS {
        output.push(format!(
            "  ... {} more violations omitted",
            violations.len() - MAX_REPORTED_VIOLATIONS
        ));
    }
    output.join("\n")
}

fn load_baseline() -> Baseline {
    let path = baseline_path();
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read baseline at {}: {e}", path.display()));
    toml::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse baseline at {}: {e}", path.display()))
}

#[test]
fn test_serializer_static_analyzer() {
    let baseline = load_baseline();

    let mut legacy_helper_violations = Vec::new();
    let mut trim_end_period_violations = Vec::new();
    let mut hardcoded_english_violations = Vec::new();
    let mut english_grammar_violations = Vec::new();

    for filename in SERIALIZER_FILES {
        let content = read_serializer_file(filename);
        legacy_helper_violations.extend(check_legacy_helpers(filename, &content));
        trim_end_period_violations.extend(check_trim_end_period(filename, &content));
        hardcoded_english_violations.extend(check_hardcoded_english(filename, &content));
        english_grammar_violations.extend(check_english_grammar(filename, &content));
    }

    println!("\n========================================");
    println!("Serializer Static Analyzer Results");
    println!("========================================");
    println!(
        "  Legacy helper violations:    {} (max allowed: {})",
        legacy_helper_violations.len(),
        baseline.max_allowed_legacy_helper_violations
    );
    println!(
        "  trim_end_matches violations: {} (max allowed: {})",
        trim_end_period_violations.len(),
        baseline.max_allowed_trim_end_period_violations
    );
    println!(
        "  Hardcoded English violations:{} (max allowed: {})",
        hardcoded_english_violations.len(),
        baseline.max_allowed_hardcoded_english_violations
    );
    println!(
        "  English grammar violations:  {} (max allowed: {})",
        english_grammar_violations.len(),
        baseline.max_allowed_english_grammar_violations
    );
    println!("========================================\n");

    if legacy_helper_violations.len() > baseline.max_allowed_legacy_helper_violations {
        panic!(
            "Legacy helper violations ({}) exceed baseline ({}):\n{}",
            legacy_helper_violations.len(),
            baseline.max_allowed_legacy_helper_violations,
            format_violations(&legacy_helper_violations)
        );
    }

    if trim_end_period_violations.len() > baseline.max_allowed_trim_end_period_violations {
        panic!(
            "trim_end_matches('.') violations ({}) exceed baseline ({}):\n{}",
            trim_end_period_violations.len(),
            baseline.max_allowed_trim_end_period_violations,
            format_violations(&trim_end_period_violations)
        );
    }

    if hardcoded_english_violations.len() > baseline.max_allowed_hardcoded_english_violations {
        panic!(
            "Hardcoded English literal violations ({}) exceed baseline ({}):\n{}",
            hardcoded_english_violations.len(),
            baseline.max_allowed_hardcoded_english_violations,
            format_violations(&hardcoded_english_violations)
        );
    }

    if english_grammar_violations.len() > baseline.max_allowed_english_grammar_violations {
        panic!(
            "English grammar logic violations ({}) exceed baseline ({}):\n{}",
            english_grammar_violations.len(),
            baseline.max_allowed_english_grammar_violations,
            format_violations(&english_grammar_violations)
        );
    }
}

/// Verifies that the detector catches legacy helper function calls.
#[test]
fn test_seeded_legacy_helper_detection() {
    let seeded_code = r#"
fn example() {
    let p = text_phrase("hello");
    let q = make_phrase("world");
    let r = with_article(p);
    let s = phrase_plural(q);
}
"#;
    let violations = check_legacy_helpers("seeded_test.rs", seeded_code);
    assert_eq!(
        violations.len(),
        4,
        "Should detect exactly 4 legacy helper violations in seeded code, got: {violations:?}"
    );
    assert!(violations.iter().all(|v| v.rule == "banned-legacy-helper"));
}

/// Verifies that the detector catches trim_end_matches('.') patterns.
#[test]
fn test_seeded_trim_end_period_detection() {
    let seeded_code = r#"
fn example() {
    let s = text.trim_end_matches('.');
    if base.ends_with('.') {
        return base;
    }
}
"#;
    let violations = check_trim_end_period("seeded_test.rs", seeded_code);
    assert_eq!(
        violations.len(),
        2,
        "Should detect exactly 2 trim_end_matches violations in seeded code, got: {violations:?}"
    );
}

/// Verifies that the detector catches hardcoded English string literals.
#[test]
fn test_seeded_hardcoded_english_detection() {
    let seeded_code = r#"
fn example() {
    let s = "allies".to_string();
    let t = "Judgment".to_string();
    let u = "enemy character".to_string();
}
"#;
    let violations = check_hardcoded_english("seeded_test.rs", seeded_code);
    assert_eq!(
        violations.len(),
        3,
        "Should detect exactly 3 hardcoded English violations in seeded code, got: {violations:?}"
    );
}

/// Verifies that the detector does NOT flag allowed patterns such as
/// strings:: phrase calls and format strings without bare English text.
#[test]
fn test_allowed_patterns_not_flagged() {
    let clean_code = r#"
fn example() {
    let s = strings::ally().to_string();
    let t = strings::character().to_string();
    let u = format!("{}.", text);
    let v = format!("{}{}", a, b);
}
"#;
    let violations = check_hardcoded_english("clean_test.rs", clean_code);
    assert!(violations.is_empty(), "Should not flag allowed patterns, but got: {violations:?}");
}

/// Verifies that the detector catches English vowel-detection grammar logic.
#[test]
fn test_seeded_english_grammar_detection() {
    let seeded_code = r#"
fn example(s: &str) {
    if s.starts_with(['a','e','i','o','u']) {
        format!("an {s}")
    } else {
        format!("a {s}")
    }
}
"#;
    let violations = check_english_grammar("seeded_test.rs", seeded_code);
    assert_eq!(
        violations.len(),
        1,
        "Should detect exactly 1 English grammar violation in seeded code, got: {violations:?}"
    );
}

/// Verifies that comments are excluded from all checks.
#[test]
fn test_comments_excluded() {
    let code_with_comments = r#"
// text_phrase("commented out")
// trim_end_matches('.')
fn example() {
    // "allies".to_string()
    let x = 1;
}
"#;
    assert!(check_legacy_helpers("test.rs", code_with_comments).is_empty());
    assert!(check_trim_end_period("test.rs", code_with_comments).is_empty());
    assert!(check_hardcoded_english("test.rs", code_with_comments).is_empty());
}

/// Verifies that function definitions are excluded from legacy helper and
/// hardcoded English checks (the declaration of `text_phrase` itself is not
/// a violation; only its usage as a call is).
#[test]
fn test_fn_definitions_excluded() {
    let fn_def_code = r#"
fn text_phrase(text: &str) -> Phrase {
    Phrase::new(text)
}
pub fn make_phrase(text: &str) -> Phrase {
    Phrase::new(text)
}
"#;
    assert!(
        check_legacy_helpers("test.rs", fn_def_code).is_empty(),
        "Function definitions should not trigger legacy helper violations"
    );
}
