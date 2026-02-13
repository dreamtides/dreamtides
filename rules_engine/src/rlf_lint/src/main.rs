use rlf::{EvalWarning, LoadWarning, Value};
use strings::strings;

/// Runs RLF static and runtime lint checks against the project's phrase
/// definitions, printing warnings and exiting non-zero if any are found.
fn main() {
    strings::register_source_phrases();

    let mut all_warnings: Vec<String> = Vec::new();

    // Static lints: analyze phrase definition ASTs without evaluation.
    rlf::with_locale(|locale| {
        let Some(registry) = locale.registry() else {
            eprintln!("No phrase registry found for current language");
            std::process::exit(1);
        };

        let definitions: Vec<_> =
            registry.phrase_names().filter_map(|name| registry.get(name).cloned()).collect();

        let static_warnings = rlf::lint_definitions(&definitions, locale.language());
        for warning in &static_warnings {
            all_warnings.push(format_load_warning(warning));
        }

        // Runtime lints: evaluate each phrase with representative arguments
        // and collect any EvalWarning values produced.
        for name in registry.phrase_names() {
            let Some(def) = registry.get(name) else {
                continue;
            };

            let args: Vec<Value> = def.parameters.iter().map(|_| Value::Number(1)).collect();

            match locale.call_phrase_with_warnings(name, &args) {
                Ok((_phrase, warnings)) => {
                    for warning in &warnings {
                        all_warnings.push(format_eval_warning(name, warning));
                    }
                }
                Err(_) => {
                    // Evaluation errors are expected for some phrases when
                    // called with placeholder arguments (e.g. phrases that
                    // reference other phrases with incompatible argument
                    // counts). Skip these silently since we are only
                    // interested in lint warnings.
                }
            }
        }
    });

    if all_warnings.is_empty() {
        println!("RLF lint passed: no warnings found");
    } else {
        println!("RLF lint found {} warning(s):\n", all_warnings.len());
        for warning in &all_warnings {
            println!("  {warning}");
        }
        std::process::exit(1);
    }
}

/// Formats a static [LoadWarning] as a human-readable string.
fn format_load_warning(warning: &LoadWarning) -> String {
    format!("[static] {warning}")
}

/// Formats a runtime [EvalWarning] as a human-readable string, including the
/// phrase name that produced it.
fn format_eval_warning(phrase_name: &str, warning: &EvalWarning) -> String {
    format!("[runtime] phrase '{phrase_name}': {warning}")
}
