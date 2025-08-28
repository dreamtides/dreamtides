use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_multiple_abilities_with_br() {
    let result = parse("Draw {-drawn-cards(n: 1)}.\n\nGain {-gained-energy(e: 2)}.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
        effect: Effect(GainEnergy(
          gains: Energy(2),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_ability_blocks_example() {
    let result = parse("{-Foresee(n: 1)}. Draw {-drawn-cards(n: 1)}.\n\n{-Reclaim-Cost(e: 3)}");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: Foresee(
              count: 1,
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: DrawCards(
              count: 1,
            ),
            optional: false,
          ),
        ]),
      )),
      Named(Reclaim(Some(Energy(3)))),
    ]
    "###
    );
}
