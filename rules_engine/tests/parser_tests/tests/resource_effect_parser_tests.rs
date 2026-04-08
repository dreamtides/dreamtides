use insta::assert_ron_snapshot;
use parser_tests::test_helpers::*;

#[test]
fn test_multiply_your_energy() {
    let result = parse_ability("{multiply_by} the amount of {energy_symbol} you have.", "n: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(MultiplyYourEnergy(
        multiplier: 2,
      )),
    ))
    "###);
}

#[test]
fn test_judgment_multiply_your_energy() {
    let result =
        parse_ability("{Judgment} {multiply_by} the amount of {energy_symbol} you have.", "n: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(MultiplyYourEnergy(
        multiplier: 3,
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_you_lose_points() {
    let result = parse_ability("{Dissolve} an enemy. You lose {points}.", "p: 1");
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
    let result = parse_ability("{Dissolve} an enemy. The opponent gains {points}.", "p: 1");
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
    let result =
        parse_ability("{Judgment} Draw {cards}. The opponent gains {points}.", "c: 2, p: 1");
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

#[test]
fn test_dissolve_enemy_draw_cards_with_cost_reduction() {
    let result = parse_abilities(
        "{Dissolve} an enemy. Draw {cards}.\n\nThis event costs {e} if a character dissolved this turn.",
        "c: 1, e: 1",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: DissolveCharacter(
              target: Enemy(Character),
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
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayForAlternateCost(AlternateCost(
          energy_cost: Energy(1),
          card_type: Some(Event),
        )),
        condition: Some(DissolvedThisTurn(
          predicate: Any(Character),
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_multiply_energy_gain_from_card_effects() {
    let result = parse_ability(
        "{multiply_by} the amount of {energy_symbol} you gain from card effects this turn.",
        "n: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CreateStaticAbilityUntilEndOfTurn(
        ability: MultiplyEnergyGainFromCardEffects(
          multiplier: 2,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_multiply_card_draw_from_card_effects() {
    let result = parse_ability(
        "{multiply_by} the number of cards you draw from card effects this turn.",
        "n: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CreateStaticAbilityUntilEndOfTurn(
        ability: MultiplyCardDrawFromCardEffects(
          multiplier: 3,
        ),
      )),
    ))
    "###);
}
