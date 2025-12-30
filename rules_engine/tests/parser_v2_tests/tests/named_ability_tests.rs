use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_reclaim_for_cost() {
    let result = parse_ability("{ReclaimForCost}", "reclaim: 2");
    assert_ron_snapshot!(result, @r###"
    Named(Reclaim(Some(Energy(2))))
    "###);
}

#[test]
fn test_foresee_draw_cards_reclaim() {
    let result = parse_abilities(
        "{Foresee}. Draw {cards}.\n\n{ReclaimForCost}",
        "foresee: 3, cards: 2, reclaim: 2",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: Foresee(
            count: 3,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Named(Reclaim(Some(Energy(2))))
    "###);
}
