use std::process::ExitCode;

/// Successful execution, no errors occurred.
pub const SUCCESS: u8 = 0;

/// System error (panic, internal invariant violation).
///
/// Indicates an error that is Lattice's fault, not the user's.
pub const SYSTEM_ERROR: u8 = 1;

/// Validation error (invalid frontmatter, malformed ID, circular dependencies).
///
/// User-provided document content is invalid.
pub const VALIDATION_ERROR: u8 = 2;

/// User input error (invalid arguments, unknown flags, bad path).
///
/// Command-line arguments or user input is invalid.
pub const USER_INPUT_ERROR: u8 = 3;

/// Not found error (unknown ID, missing file, no results).
///
/// A requested resource does not exist.
pub const NOT_FOUND: u8 = 4;

pub fn success() -> ExitCode {
    ExitCode::from(SUCCESS)
}

pub fn system_error() -> ExitCode {
    ExitCode::from(SYSTEM_ERROR)
}

pub fn validation_error() -> ExitCode {
    ExitCode::from(VALIDATION_ERROR)
}

pub fn user_input_error() -> ExitCode {
    ExitCode::from(USER_INPUT_ERROR)
}

pub fn not_found() -> ExitCode {
    ExitCode::from(NOT_FOUND)
}
