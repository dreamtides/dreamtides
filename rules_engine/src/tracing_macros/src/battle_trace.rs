/// Macro for adding a tracing event to a battle.
///
/// This macro does two things:
/// 1. If tracing is enabled for the battle, it records an event in the battle's
///    trace history.
/// 2. It emits a debug-level trace event via the 'tracing' crate.
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
///   (Note that these forms cannot be combined)
///
/// Example:
/// ```rust
/// // With simple variables:
/// battle_trace!("Drawing cards", battle, player, count);
///
/// // With expressions:
/// battle_trace!("Resolving card", battle, card_id = card_id, controller = source.controller());
/// ```
#[macro_export]
macro_rules! battle_trace {
    ($message:expr, $battle:expr) => {{
        tracing::debug!($message);
    }};
    ($message:expr, $battle:expr, $($key:ident),* $(,)?) => {{
        $( let $key = &$key; )*
        tracing::debug!(message = %$message, $($key = ?$key),*);
    }};
    ($message:expr, $battle:expr, $($key:ident = $value:expr),* $(,)?) => {{
        tracing::debug!(message = %$message, $($key = ?$value),*);
    }};
}
