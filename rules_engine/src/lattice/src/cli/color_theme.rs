use std::env;
use std::fmt::{self, Display};
use std::io::IsTerminal;
use std::sync::OnceLock;

use owo_colors::{OwoColorize, Rgb, Style};

/// Green for success states (Ayu green)
pub const AYU_SUCCESS: Rgb = Rgb(149, 230, 203);

/// Yellow/amber for warning states (Ayu orange)
pub const AYU_WARNING: Rgb = Rgb(255, 180, 84);

/// Red/coral for error states (Ayu red)
pub const AYU_ERROR: Rgb = Rgb(240, 113, 120);

/// Blue for accent/highlighting (Ayu blue)
pub const AYU_ACCENT: Rgb = Rgb(89, 194, 255);

/// Gray for muted/secondary text (Ayu comment)
pub const AYU_MUTED: Rgb = Rgb(99, 106, 114);

/// Brighter gray for de-emphasized but readable text
pub const AYU_DIM: Rgb = Rgb(140, 145, 152);

/// Purple for special highlighting (Ayu purple)
pub const AYU_SPECIAL: Rgb = Rgb(217, 149, 255);

/// A colored text wrapper that conditionally applies styling.
///
/// When colors are disabled (non-TTY output or NO_COLOR set), the text is
/// displayed without any ANSI escape codes.
pub struct Styled<T> {
    value: T,
    style: Style,
}

/// Determines whether colors should be enabled for output.
///
/// Returns `true` if any of the following are true:
/// - The `FORCE_COLOR` environment variable is set (common convention for
///   forcing colors in piped output)
///
/// Otherwise, returns `true` if all of the following are true:
/// - The `LATTICE_NO_COLOR` environment variable is not set
/// - The standard `NO_COLOR` environment variable is not set (per no-color.org)
/// - Stdout is connected to a terminal (not piped)
///
/// This function caches its result after the first call.
pub fn colors_enabled() -> bool {
    static COLORS_ENABLED: OnceLock<bool> = OnceLock::new();
    *COLORS_ENABLED.get_or_init(|| {
        if env::var("FORCE_COLOR").is_ok() {
            return true;
        }
        let no_color_env = env::var("LATTICE_NO_COLOR").is_ok() || env::var("NO_COLOR").is_ok();
        let is_tty = std::io::stdout().is_terminal();
        !no_color_env && is_tty
    })
}

/// Formats text in the success color (green).
pub fn success<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_SUCCESS) }
}

/// Formats text in the warning color (yellow/amber).
pub fn warning<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_WARNING) }
}

/// Formats text in the error color (red).
pub fn error<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_ERROR) }
}

/// Formats text in the accent color (blue).
pub fn accent<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_ACCENT) }
}

/// Formats text in the muted color (gray).
pub fn muted<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_MUTED) }
}

/// Formats text in a dimmed but readable style.
pub fn dim<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_DIM) }
}

/// Formats text in the special highlight color (purple).
pub fn special<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_SPECIAL) }
}

/// Formats text in bold.
pub fn bold<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().bold() }
}

/// Formats a Lattice ID (accent color, bold).
pub fn lattice_id<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_ACCENT).bold() }
}

/// Formats a task type label (e.g., "bug", "feature").
pub fn task_type<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_SPECIAL) }
}

/// Formats a priority indicator.
pub fn priority<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_WARNING) }
}

/// Formats a file path.
pub fn path<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_DIM) }
}

/// Formats a label/tag.
pub fn label<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_MUTED) }
}

/// Formats the "open" status indicator.
pub fn status_open<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_SUCCESS) }
}

/// Formats the "blocked" status indicator.
pub fn status_blocked<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_WARNING) }
}

/// Formats the "closed" status indicator.
pub fn status_closed<T: Display>(value: T) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_MUTED) }
}

/// Formats a priority bar based on priority level.
pub fn priority_bar<T: Display>(value: T, _priority: u8) -> Styled<T> {
    Styled { value, style: Style::new().color(AYU_ACCENT) }
}

/// Creates a custom-styled text wrapper.
pub fn styled<T: Display>(value: T, color: Rgb) -> Styled<T> {
    Styled { value, style: Style::new().color(color) }
}

/// Creates a custom-styled text wrapper with additional style options.
pub fn styled_with<T: Display>(value: T, style: Style) -> Styled<T> {
    Styled { value, style }
}

impl<T: Display> Display for Styled<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if colors_enabled() {
            write!(f, "{}", self.value.style(self.style))
        } else {
            write!(f, "{}", self.value)
        }
    }
}
