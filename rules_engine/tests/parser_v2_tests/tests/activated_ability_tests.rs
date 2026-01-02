use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_abandon_an_ally_gain_energy() {
    let result = parse_ability("Abandon an ally: Gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharacters(Another(Character), 1),
      ],
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_kindle() {
    let result = parse_ability("Abandon an ally: {Kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharacters(Another(Character), 1),
      ],
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_this_character_gains_spark() {
    let result = parse_ability("Abandon an ally: This character gains +{s} spark.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharacters(Another(Character), 1),
      ],
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_count_allies_reclaim_this_character() {
    let result = parse_ability("Abandon {count-allies}: {Reclaim} this character.", "allies: 3");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(3),
        ),
      ],
      effect: Effect(ReturnFromYourVoidToPlay(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_energy_draw_cards() {
    let result = parse_ability("{e}: Draw {cards}.", "e: 1, cards: 2");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
      ],
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}
