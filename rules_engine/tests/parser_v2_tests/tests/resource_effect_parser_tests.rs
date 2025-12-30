use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_dissolve_enemy_you_lose_points() {
    let result = parse_ability("{Dissolve} an enemy. You lose {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DissolveCharacter(
            target: Enemy(Character),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: LosePoints(
            loses: Points(1),
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_opponent_gains_points() {
    let result = parse_ability("{Dissolve} an enemy. The opponent gains {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DissolveCharacter(
            target: Enemy(Character),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: EnemyGainsPoints(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_judgment_draw_cards_opponent_gains_points() {
    let result = parse_ability(
        "{Judgment} Draw {cards}. The opponent gains {points}.",
        "cards: 2, points: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: EnemyGainsPoints(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}
