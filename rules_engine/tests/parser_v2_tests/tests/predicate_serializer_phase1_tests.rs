use ability_data::predicate::{CardPredicate, Operator, Predicate};
use core_data::numerics::Spark;
use parser_v2::serializer::predicate_serializer;
use strings::strings;

fn register_phrases() {
    strings::register_source_phrases();
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
        "characters in your void",
        predicate_serializer::serialize_predicate(&Predicate::YourVoid(CardPredicate::Character))
            .to_string()
    );
    assert_eq!(
        "events in the opponent's void",
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
fn serialize_predicate_plural_migrated_variants_render_expected_text() {
    register_phrases();

    assert_eq!(
        "allies",
        predicate_serializer::serialize_predicate_plural(&Predicate::Another(
            CardPredicate::Character,
        ))
        .to_string()
    );
    assert_eq!(
        "your cards",
        predicate_serializer::serialize_predicate_plural(&Predicate::Your(CardPredicate::Card))
            .to_string()
    );
    assert_eq!(
        "enemy events",
        predicate_serializer::serialize_predicate_plural(&Predicate::Enemy(CardPredicate::Event))
            .to_string()
    );
    assert_eq!(
        "these characters",
        predicate_serializer::serialize_predicate_plural(&Predicate::This).to_string()
    );
    assert_eq!(
        "those characters",
        predicate_serializer::serialize_predicate_plural(&Predicate::That).to_string()
    );
    assert_eq!(
        "other characters",
        predicate_serializer::serialize_predicate_plural(&Predicate::AnyOther(
            CardPredicate::Character,
        ))
        .to_string()
    );
}

#[test]
fn serialize_predicate_plural_keeps_complex_predicate_text() {
    register_phrases();

    assert_eq!(
        "allies with spark 3 or more",
        predicate_serializer::serialize_predicate_plural(&Predicate::Your(
            CardPredicate::CharacterWithSpark(Spark(3), Operator::OrMore),
        ))
        .to_string()
    );
}
