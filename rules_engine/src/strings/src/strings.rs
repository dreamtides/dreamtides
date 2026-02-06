use std::sync::LazyLock;

use rlf::Locale;

static LOCALE: LazyLock<Locale> = LazyLock::new(|| {
    let mut locale = Locale::new();
    register_source_phrases(&mut locale);
    locale
});

/// Returns a reference to the shared RLF locale with all source phrases
/// registered.
pub fn locale() -> &'static Locale {
    &LOCALE
}

rlf::rlf! {
    primary_button_submit_void_card_targets = "Submit";
}
