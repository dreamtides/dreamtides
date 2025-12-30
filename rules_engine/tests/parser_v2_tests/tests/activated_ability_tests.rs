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
