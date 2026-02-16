use strings::strings;

fn main() {
    strings::register_source_phrases();
    rlf::with_locale(rlf::run_lints);
}
