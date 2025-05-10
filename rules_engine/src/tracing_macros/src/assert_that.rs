/// Macro for checking invariants with tracing
///
/// This macro does three things if the provided boolean condition is false:
/// 1. If tracing is enabled for the battle, it records an event in the battle's
///    trace history.
/// 2. It emits a error-level trace event via the 'tracing' crate.
/// 3. It fires a standard Rust panic.
///
/// Arguments:
/// - `$message`: A message describing the event.
/// - `$battle`: The battle data, must be a mutable reference.
/// - The remaining arguments are symbols to include in the trace and can take
///   either of two forms:
///
///   - A list of variable names
///   - A list of expressions with the form `name = expr`
///
///  Note that these forms cannot be combined, and that the provided 'expr'
///  expressions will only be evaluated in the event that the boolean condition
///  provided is false, in order to avoid expensive calculations.
///
/// Example:
/// ```rust
/// // With simple variables:
/// assert_that!(1 + 1 > 0, "Addition works", battle, player, count);
///
/// // With expressions:
/// assert_that!(2 != 4, "Equality", battle, card_id = card_id, controller = source.controller());
/// ```
#[macro_export]
macro_rules! assert_that {
    ($condition:expr, $message:expr, $battle:expr) => {{
        if !$condition {
            tracing::error!($message);
            $crate::snapshot($battle);
            panic!("Assertion failed: {}", $message);
        }
    }};
    ($condition:expr, $message:expr, $battle:expr, $($key:ident),* $(,)?) => {{
        if !$condition {
            $( let $key = &$key; )*
            tracing::error!(message = %$message, $($key = ?$key),*);
            $crate::snapshot($battle);
            panic!("Assertion failed: {}", $message);
        }
    }};
    ($condition:expr, $message:expr, $battle:expr, $($key:ident = $value:expr),* $(,)?) => {{
        if !$condition {
            tracing::error!(message = %$message, $($key = ?$value),*);
            $crate::snapshot($battle);
            panic!("Assertion failed: {}", $message);
        }
    }};
}
