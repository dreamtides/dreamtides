use parser_v2_tests::test_helpers;

const BRACKET_LANGUAGE: &str = "en-x-bracket";

#[test]
fn bracket_locale_loads_all_source_phrases() {
    let loaded = test_helpers::register_bracket_test_locale()
        .expect("bracket locale should load from locale file");
    rlf::with_locale(|locale| {
        let source_phrase_count = locale
            .registry_for("en")
            .expect("source phrases should be registered in English")
            .phrase_names()
            .count();
        assert_eq!(
            loaded, source_phrase_count,
            "bracket locale phrase count should match English source phrase count"
        );
    });
}

#[test]
fn registering_bracket_locale_keeps_english_as_default() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    rlf::with_locale(|locale| {
        assert_eq!(
            locale.language(),
            "en",
            "registering bracket locale should not switch language"
        );
        assert_eq!(
            locale.get_phrase("card").expect("card phrase should resolve in English").to_string(),
            "card"
        );
    });
}

#[test]
fn bracket_locale_can_be_selected_and_restored() {
    test_helpers::register_bracket_test_locale().expect("bracket locale should load");
    rlf::with_locale_mut(|locale| {
        let previous_language = locale.language().to_string();
        let english_card = locale
            .get_phrase("card")
            .expect("card phrase should resolve before locale switch")
            .to_string();
        locale.set_language(BRACKET_LANGUAGE);
        let bracket_card = locale
            .get_phrase("card")
            .expect("card phrase should resolve in bracket locale")
            .to_string();
        locale.set_language(previous_language);
        let restored_card = locale
            .get_phrase("card")
            .expect("card phrase should resolve after restoring language")
            .to_string();
        assert_eq!(english_card, "card");
        assert_eq!(bracket_card, "[card]");
        assert_eq!(restored_card, english_card);
    });
}
