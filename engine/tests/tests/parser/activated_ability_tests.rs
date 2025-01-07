use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_banish_from_void_dissolve_enemy_character() {
    let result = parse("$activated Banish 3 cards from your void: Dissolve an enemy character with cost $2 or less.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        cost: BanishCardsFromYourVoid(3),
        effect: Effect(DissolveCharacter(Enemy(CharacterWithCost(Energy(2), OrLess)))),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_fast_activated_grant_aegis() {
    let result = parse("$fastActivated: Another character you control gains {kw: aegis} this turn. {reminder: (it cannot be affected by the enemy)} {flavor: She stands where others would fall.}");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        cost: None,
        effect: Effect(GainsAegisThisTurn(Another(Character))),
        options: Some(ActivatedAbilityOptions(
          is_fast: true,
          is_immediate: false,
          is_multi: false,
        )),
      )),
    ]
    "###
    );
}
