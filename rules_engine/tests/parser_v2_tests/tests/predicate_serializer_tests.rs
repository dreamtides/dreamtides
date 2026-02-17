use ability_data::predicate::{CardPredicate, Operator, Predicate};
use core_data::card_types::CardSubtype;
use core_data::numerics::Spark;
use parser_v2::serializer::predicate_serializer;
use strings::strings;

fn register_phrases() {
    strings::register_source_phrases();
}

/// Renders the plural (:other) variant of a predicate by passing it through
/// `collection_all`, which selects `{$target:other}`.
fn render_plural(predicate: &Predicate) -> String {
    let phrase = predicate_serializer::serialize_predicate(predicate);
    let rendered = strings::collection_all(phrase).to_string();
    // collection_all renders "all {$target:other}", so strip the "all " prefix.
    rendered.strip_prefix("all ").unwrap_or(&rendered).to_string()
}

#[test]
fn serialize_predicate_migrated_variants_render_expected_text() {
    register_phrases();

    assert_eq!(
        "this character",
        predicate_serializer::serialize_predicate(&Predicate::This).to_string()
    );
    assert_eq!(
        "that character",
        predicate_serializer::serialize_predicate(&Predicate::That).to_string()
    );
    assert_eq!("them", predicate_serializer::serialize_predicate(&Predicate::Them).to_string());
    assert_eq!("it", predicate_serializer::serialize_predicate(&Predicate::It).to_string());
    assert_eq!(
        "a character in your void",
        predicate_serializer::serialize_predicate(&Predicate::YourVoid(CardPredicate::Character))
            .to_string()
    );
    assert_eq!(
        "an event in the opponent's void",
        predicate_serializer::serialize_predicate(&Predicate::EnemyVoid(CardPredicate::Event))
            .to_string()
    );
    assert_eq!(
        "another character",
        predicate_serializer::serialize_predicate(&Predicate::AnyOther(CardPredicate::Character))
            .to_string()
    );
}

#[test]
fn serialize_predicate_preserves_indefinite_articles() {
    register_phrases();

    assert_eq!(
        "an ally",
        predicate_serializer::serialize_predicate(&Predicate::Another(CardPredicate::Character))
            .to_string()
    );
    assert_eq!(
        "an enemy",
        predicate_serializer::serialize_predicate(&Predicate::Enemy(CardPredicate::Character))
            .to_string()
    );
    assert_eq!(
        "an enemy card",
        predicate_serializer::serialize_predicate(&Predicate::Enemy(CardPredicate::Card))
            .to_string()
    );
}

#[test]
fn serialize_predicate_plural_via_other_variant() {
    register_phrases();

    assert_eq!("allies", render_plural(&Predicate::Another(CardPredicate::Character)));
    assert_eq!("your cards", render_plural(&Predicate::Your(CardPredicate::Card)));
    assert_eq!("enemy events", render_plural(&Predicate::Enemy(CardPredicate::Event)));
    assert_eq!("these characters", render_plural(&Predicate::This));
    assert_eq!("those characters", render_plural(&Predicate::That));
    assert_eq!("other characters", render_plural(&Predicate::AnyOther(CardPredicate::Character)));
}

#[test]
fn serialize_predicate_plural_keeps_complex_predicate_text() {
    register_phrases();

    assert_eq!(
        "allies with spark 3 or more",
        render_plural(&Predicate::Your(CardPredicate::CharacterWithSpark(
            Spark(3),
            Operator::OrMore
        ),))
    );
}

#[test]
fn serialize_predicate_composes_owned_could_dissolve_event() {
    register_phrases();

    assert_eq!(
        "an event which could <color=#AA00FF>dissolve</color> an enemy",
        predicate_serializer::serialize_predicate(&Predicate::Your(CardPredicate::CouldDissolve {
            target: Box::new(Predicate::Enemy(CardPredicate::Character)),
        }))
        .to_string()
    );
}

#[test]
fn serialize_predicate_your_subtype_singular_and_plural() {
    register_phrases();

    // Your(CharacterType) singular renders as "a Warrior"
    let your_phrase = predicate_serializer::serialize_predicate(&Predicate::Your(
        CardPredicate::CharacterType(CardSubtype::Warrior),
    ));
    assert_eq!("a <color=#2E7D32><b>Warrior</b></color>", your_phrase.to_string());
    // Your(CharacterType) plural renders as "allied Warriors"
    assert_eq!(
        "all allied <color=#2E7D32><b>Warriors</b></color>",
        strings::collection_all(your_phrase).to_string()
    );

    // Another(CharacterType) singular renders as "an allied Warrior"
    let another_phrase = predicate_serializer::serialize_predicate(&Predicate::Another(
        CardPredicate::CharacterType(CardSubtype::Warrior),
    ));
    assert_eq!("an allied <color=#2E7D32><b>Warrior</b></color>", another_phrase.to_string());
    // Another(CharacterType) plural renders as "allied Warriors"
    assert_eq!(
        "all allied <color=#2E7D32><b>Warriors</b></color>",
        strings::collection_all(another_phrase).to_string()
    );
}

#[test]
fn serialize_predicate_another_subtype_spark_equal_to_count() {
    register_phrases();

    // spark_equal_to_predicate_count uses :other variant for plural matching
    let another_phrase = predicate_serializer::serialize_predicate(&Predicate::Another(
        CardPredicate::CharacterType(CardSubtype::Warrior),
    ));
    let spark_text = strings::spark_equal_to_predicate_count(another_phrase).to_string();
    assert!(spark_text.contains("allied"), "spark_equal_to should contain 'allied': {spark_text}");
    assert!(
        spark_text.contains("Warriors"),
        "spark_equal_to should contain plural 'Warriors': {spark_text}"
    );
}
