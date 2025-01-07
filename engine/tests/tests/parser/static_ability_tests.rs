use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_enemy_events_cost_more_to_play() {
    let result = parse("The enemy's events cost an additional $1 to play.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Static(EnemyAddedCostToPlay(Event, Energy(1))),
    ]
    "###
    );
}

#[test]
fn test_once_per_turn_play_2_or_less_from_void() {
    let result =
        parse("Once per turn, you may play a character with cost $2 or less from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(OncePerTurnPlayFromVoid(CharacterWithCost(Energy(2), OrLess))),
    ]
    "###);
}
