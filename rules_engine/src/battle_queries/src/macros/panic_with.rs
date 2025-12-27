/// Macro for firing panics with tracing
///
/// This macro does three things:
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
/// panic_with!("Error", battle, player, count);
///
/// // With expressions:
/// panic_with!("Another error", battle, card_id = card_id, controller = source.controller());
/// ```
#[macro_export]
macro_rules! panic_with {
    ($message:expr, $battle:expr) => {{
        tracing::error!($message);
        eprintln!("Error: {}", $message);
        $crate::macros::write_tracing_event::write_panic_snapshot(
            $battle,
            $message.to_string(),
            std::collections::BTreeMap::new())
        ;
        panic!("Error: {}", $message);
    }};
    ($message:expr, $battle:expr, $($key:ident),* $(,)?) => {{
        $( let $key = &$key; )*
        tracing::error!(message = %$message, $($key = ?$key),*);
        eprintln!("Error: {}", $message);
        $(eprintln!("  {}: {:?}", stringify!($key), $key);)*

        let mut values = std::collections::BTreeMap::new();
        $(
            values.insert(stringify!($key).to_string(), format!("{:?}", $key));
        )*

        $crate::macros::write_tracing_event::write_panic_snapshot(
            $battle,
            $message.to_string(),
            values
        );

        let panic_message = format!("Error: {}{}", $message, {
            let mut parts = Vec::new();
            $(
                parts.push(format!(", {}: {:?}", stringify!($key), $key));
            )*
            parts.join("")
        });
        panic!("{}", panic_message);
    }};
    ($message:expr, $battle:expr, $($key:ident = $value:expr),* $(,)?) => {{
        tracing::error!(message = %$message, $($key = ?$value),*);
        eprintln!("Error: {}", $message);
        $(eprintln!("  {}: {:?}", stringify!($key), $value);)*

        let mut values = std::collections::BTreeMap::new();
        $(
            values.insert(stringify!($key).to_string(), format!("{:?}", $value));
        )*

        $crate::macros::write_tracing_event::write_panic_snapshot(
            $battle,
            $message.to_string(),
            values
        );

        let panic_message = format!("Error: {}{}", $message, {
            let mut parts = Vec::new();
            $(
                parts.push(format!(", {}: {:?}", stringify!($key), $value));
            )*
            parts.join("")
        });
        panic!("{}", panic_message);
    }};
}
