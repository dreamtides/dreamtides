/// Strip bidirectional control characters from a string.
pub fn normalize_display_string(s: &str) -> String {
    s.replace(['\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}'], "")
}

#[macro_export]
macro_rules! assert_display_string_contains {
    ($haystack:expr, $needle:expr, $($arg:tt)+) => {{
        let normalized_haystack = $crate::assertions::normalize_display_string(&$haystack);
        let normalized_needle = $crate::assertions::normalize_display_string(&$needle);
        if !normalized_haystack.contains(&normalized_needle) {
            panic!($($arg)+);
        }
    }};
    ($haystack:expr, $needle:expr) => {{
        let normalized_haystack = $crate::assertions::normalize_display_string(&$haystack);
        let normalized_needle = $crate::assertions::normalize_display_string(&$needle);
        if !normalized_haystack.contains(&normalized_needle) {
            panic!("Rules text should contain '{}', but got: '{}'", normalized_needle, $haystack);
        }
    }};
}
