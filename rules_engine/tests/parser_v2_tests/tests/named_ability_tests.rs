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
fn test_reclaim_abandon_ally() {
    let result = parse_ability("{Reclaim} -- Abandon an ally", "");
    assert_ron_snapshot!(result, @r###"
    Named(ReclaimForCost(AbandonCharactersCount(
      target: Another(Character),
      count: Exactly(1),
    )))
    "###);
}

#[test]
fn test_dissolve_enemy_with_cost_or_less_reclaim_abandon_ally() {
    let result = parse_abilities(
        "{Dissolve} an enemy with cost {e} or less.\n\n{Reclaim} -- Abandon an ally",
        "e: 3",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(3),
        )),
      )),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Named(ReclaimForCost(AbandonCharactersCount(
      target: Another(Character),
      count: Exactly(1),
    )))
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

#[test]
fn test_draw_discard_reclaim() {
    let result = parse_abilities(
        "Draw {cards}. Discard {discards}.\n\n{ReclaimForCost}",
        "cards: 2, discards: 1, reclaim: 3",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Named(Reclaim(Some(Energy(3))))
    "###);
}

#[test]
fn test_materialized_draw_discard_reclaim() {
    let result = parse_abilities(
        "{Materialized} Draw {cards}. Discard {discards}.\n\n{ReclaimForCost}",
        "cards: 2, discards: 1, reclaim: 3",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Named(Reclaim(Some(Energy(3))))
    "###);
}

#[test]
fn test_dissolve_enemy_with_cost_reclaim() {
    let result = parse_abilities(
        "{Dissolve} an enemy with cost {e} or more.\n\n{ReclaimForCost}",
        "e: 4, reclaim: 2",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrMore,
          cost: Energy(4),
        )),
      )),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Named(Reclaim(Some(Energy(2))))
    "###);
}

#[test]
fn test_banish_from_hand_alternate_cost_dissolve_enemy() {
    let result = parse_abilities(
        "{Banish} a card from hand: Play this event for {e}.\n\n{Dissolve} an enemy.",
        "e: 0",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(BanishFromHand(Any(Card))),
    ))))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_ally_alternate_cost_materialized_dissolve_enemy() {
    let result = parse_abilities(
        "Abandon an ally: Play this character for {e}, then abandon it.\n\n{Materialized} {Dissolve} an enemy.",
        "e: 0",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(AbandonCharactersCount(
        target: Another(Character),
        count: Exactly(1),
      )),
      if_you_do: Some(Effect(PayCost(
        cost: AbandonCharactersCount(
          target: This,
          count: Exactly(1),
        ),
      ))),
    ))))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_banish_from_hand_alternate_cost_prevent_enemy_card() {
    let result = parse_abilities(
        "{Banish} a card from hand: Play this event for {e}.\n\n{Prevent} a played enemy card.",
        "e: 0",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(BanishFromHand(Any(Card))),
    ))))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Enemy(Card),
      )),
    ))
    "###);
}

#[test]
fn test_lose_maximum_energy_alternate_cost_prevent_card() {
    let result = parse_abilities(
        "Lose {maximum-energy}: Play this event for {e}.\n\n{Prevent} a played card.",
        "max: 1, e: 0",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(LoseMaximumEnergy(1)),
    ))))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Card),
      )),
    ))
    "###);
}
