use ability_data::predicate::Predicate;
use ability_data::standard_effect::StandardEffect;

/// Returns true if a standard effect requires a target to resolve.
pub fn has_targets(effect: &StandardEffect) -> bool {
    get_target_predicate(effect).is_some()
}

/// Returns the targeting predicate required to resolve a standard effect, if
/// any.
pub fn get_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    get_character_target_predicate(effect)
        .or_else(|| get_stack_target_predicate(effect))
        .or_else(|| get_void_target_predicate(effect))
}

/// Extracts a character target predicate from a standard effect, if any.
pub fn get_character_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::AbandonAndGainEnergyForSpark { target, .. } => Some(target),
        StandardEffect::BanishCharacter { target } => Some(target),
        StandardEffect::BanishCharacterUntilLeavesPlay { target, .. } => Some(target),
        StandardEffect::BanishUntilNextMain { target } => Some(target),
        StandardEffect::BanishCollection { target, .. } => Some(target),
        StandardEffect::Copy { target } => Some(target),
        StandardEffect::DisableActivatedAbilitiesWhileInPlay { target } => Some(target),
        StandardEffect::DissolveCharacter { target } => Some(target),
        StandardEffect::DissolveCharactersCount { target, .. } => Some(target),
        StandardEffect::DissolveCharactersQuantity { target, .. } => Some(target),
        StandardEffect::GainControl { target } => Some(target),
        StandardEffect::GainsAegisThisTurn { target } => Some(target),
        StandardEffect::GainsReclaimUntilEndOfTurn { target, .. } => Some(target),
        StandardEffect::GainsSpark { target, .. } => Some(target),
        StandardEffect::GainsSparkForQuantity { target, .. } => Some(target),
        StandardEffect::GainsSparkUntilYourNextMainForEach { target, .. } => Some(target),
        StandardEffect::MaterializeCharacterAtEndOfTurn { target } => Some(target),
        StandardEffect::MaterializeCollection { target, .. } => Some(target),
        StandardEffect::MaterializeSilentCopy { target, .. } => Some(target),
        StandardEffect::PreventDissolveThisTurn { target } => Some(target),
        StandardEffect::PutOnTopOfEnemyDeck { target } => Some(target),
        StandardEffect::ReturnToHand { target } => Some(target),

        StandardEffect::AbandonAtEndOfTurn { .. } => None,
        StandardEffect::BanishCardsFromEnemyVoid { .. } => None,
        StandardEffect::BanishEnemyVoid => None,
        StandardEffect::BanishWhenLeavesPlay { .. } => None,
        StandardEffect::CardsInVoidGainReclaimThisTurn { .. } => None,
        StandardEffect::CopyNextPlayed { .. } => None,
        StandardEffect::Counterspell { .. } => None,
        StandardEffect::CounterspellUnlessPaysCost { .. } => None,
        StandardEffect::CreateTriggerUntilEndOfTurn { .. } => None,
        StandardEffect::DiscardCardFromEnemyHand { .. } => None,
        StandardEffect::DiscardCardFromEnemyHandThenTheyDraw { .. } => None,
        StandardEffect::DiscardCards { .. } => None,
        StandardEffect::Discover { .. } => None,
        StandardEffect::DiscoverAndThenMaterialize { .. } => None,
        StandardEffect::DoubleYourEnergy => None,
        StandardEffect::DrawCards { .. } => None,
        StandardEffect::DrawCardsForEach { .. } => None,
        StandardEffect::DrawMatchingCard { .. } => None,
        StandardEffect::EachMatchingGainsSparkForEach { .. } => None,
        StandardEffect::EachMatchingGainsSpark { .. } => None,
        StandardEffect::EachPlayerAbandonsCharacters { .. } => None,
        StandardEffect::EachPlayerDiscardCards { .. } => None,
        StandardEffect::EnemyGainsPoints { .. } => None,
        StandardEffect::EnemyGainsPointsEqualToItsSpark => None,
        StandardEffect::EnemyLosesPoints { .. } => None,
        StandardEffect::Foresee { .. } => None,
        StandardEffect::GainEnergy { .. } => None,
        StandardEffect::GainEnergyForEach { .. } => None,
        StandardEffect::GainPoints { .. } => None,
        StandardEffect::GainPointsForEach { .. } => None,
        StandardEffect::GainTwiceThatMuchEnergyInstead => None,
        StandardEffect::Kindle { .. } => None,
        StandardEffect::LosePoints { .. } => None,
        StandardEffect::MaterializeCharacter { .. } => None,
        StandardEffect::MaterializeCharacterFromVoid { .. } => None,
        StandardEffect::MaterializeRandomFromDeck { .. } => None,
        StandardEffect::NoEffect => None,
        StandardEffect::OpponentPaysCost { .. } => None,
        StandardEffect::PayCost { .. } => None,
        StandardEffect::PutCardsFromVoidOnTopOfDeck { .. } => None,
        StandardEffect::PutCardsFromYourDeckIntoVoid { .. } => None,
        StandardEffect::ReturnCharactersToHandDrawCardForEach { .. } => None,
        StandardEffect::ReturnFromYourVoidToHand { .. } => None,
        StandardEffect::ReturnFromYourVoidToPlay { .. } => None,
        StandardEffect::ReturnUpToCountFromYourVoidToHand { .. } => None,
        StandardEffect::ShuffleHandAndDeckAndDraw { .. } => None,
        StandardEffect::SparkBecomes { .. } => None,
        StandardEffect::SpendAllEnergyDissolveEnemy => None,
        StandardEffect::SpendAllEnergyDrawAndDiscard => None,
        StandardEffect::TakeExtraTurn => None,
        StandardEffect::ThenMaterializeIt => None,
        StandardEffect::TriggerJudgmentAbility { .. } => None,
        StandardEffect::YouWinTheGame => None,
    }
}

/// Extracts a stack target predicate from a standard effect, if any.
pub fn get_stack_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::AbandonAtEndOfTurn { target } => Some(target),
        StandardEffect::BanishWhenLeavesPlay { target } => Some(target),
        StandardEffect::Counterspell { target, .. } => Some(target),
        StandardEffect::CounterspellUnlessPaysCost { target, .. } => Some(target),
        _ => None,
    }
}

/// Extracts a void target predicate from a standard effect, if any.
pub fn get_void_target_predicate(effect: &StandardEffect) -> Option<&Predicate> {
    match effect {
        StandardEffect::ReturnFromYourVoidToHand { target } => Some(target),
        StandardEffect::ReturnUpToCountFromYourVoidToHand { target, .. } => Some(target),
        _ => None,
    }
}
