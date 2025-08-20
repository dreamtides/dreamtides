use core_data::identifiers::{BaseCardId, CardIdentity, CardName};

/// Describes a card and the set of modifications applied to it.
///
/// A "base" card is a card that appears in the Tabula database with no
/// modifications. The `CardDescriptor` adds a set of modifications to a base
/// card. A `CardDescriptor` thus represents a `CardIdentity`, and we use the
/// `CardIdentity` in the rules engine as an efficient way to refer back to "a
/// card and its modifications". Card descriptors always map directly to an
/// immutable list of abilities for a card.
///
/// A "deck" of cards is hence always a collection of `CardDescriptor`s.
#[derive(Debug, Clone)]
pub struct CardDescriptor {
    pub base_id: BaseCardId,
    pub is_upgraded: bool,
}

/// Returns the [CardIdentity] for a given base card with no modifications.
pub fn get_base_identity(base_id: CardName) -> CardIdentity {
    match base_id {
        CardName::TestVanillaCharacter => CardIdentity(0),
        CardName::TestDissolve => CardIdentity(1),
        CardName::TestNamedDissolve => CardIdentity(2),
        CardName::TestCounterspellUnlessPays => CardIdentity(3),
        CardName::TestCounterspell => CardIdentity(4),
        CardName::TestCounterspellCharacter => CardIdentity(5),
        CardName::TestVariableEnergyDraw => CardIdentity(6),
        CardName::TestDrawOne => CardIdentity(7),
        CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter => CardIdentity(8),
        CardName::TestTriggerGainSparkOnPlayCardEnemyTurn => CardIdentity(9),
        CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn => CardIdentity(10),
        CardName::TestActivatedAbilityDrawCard => CardIdentity(11),
        CardName::TestMultiActivatedAbilityDrawCardCharacter => CardIdentity(12),
        CardName::TestFastActivatedAbilityDrawCardCharacter => CardIdentity(13),
        CardName::TestFastMultiActivatedAbilityDrawCardCharacter => CardIdentity(14),
        CardName::TestActivatedAbilityDissolveCharacter => CardIdentity(15),
        CardName::TestDualActivatedAbilityCharacter => CardIdentity(16),
        CardName::TestForeseeOne => CardIdentity(17),
        CardName::TestForeseeTwo => CardIdentity(18),
        CardName::TestForeseeOneDrawACard => CardIdentity(19),
        CardName::TestDrawOneReclaim => CardIdentity(20),
        CardName::TestForeseeOneReclaim => CardIdentity(21),
        CardName::TestForeseeOneDrawReclaim => CardIdentity(22),
        CardName::TestReturnVoidCardToHand => CardIdentity(23),
        CardName::TestReturnOneOrTwoVoidEventCardsToHand => CardIdentity(24),
        CardName::TestModalDrawOneOrDrawTwo => CardIdentity(25),
        CardName::TestModalDrawOneOrDissolveEnemy => CardIdentity(26),
        CardName::TestModalReturnToHandOrDrawTwo => CardIdentity(27),
        CardName::TestReturnToHand => CardIdentity(28),
        CardName::TestPreventDissolveThisTurn => CardIdentity(29),
    }
}

/// Returns the [BaseCardId] for a given [CardIdentity].
pub fn get_base_card_id(identity: CardIdentity) -> CardName {
    match identity.0 {
        0 => CardName::TestVanillaCharacter,
        1 => CardName::TestDissolve,
        2 => CardName::TestNamedDissolve,
        3 => CardName::TestCounterspellUnlessPays,
        4 => CardName::TestCounterspell,
        5 => CardName::TestCounterspellCharacter,
        6 => CardName::TestVariableEnergyDraw,
        7 => CardName::TestDrawOne,
        8 => CardName::TestTriggerGainSparkWhenMaterializeAnotherCharacter,
        9 => CardName::TestTriggerGainSparkOnPlayCardEnemyTurn,
        10 => CardName::TestTriggerGainTwoSparkOnPlayCardEnemyTurn,
        11 => CardName::TestActivatedAbilityDrawCard,
        12 => CardName::TestMultiActivatedAbilityDrawCardCharacter,
        13 => CardName::TestFastActivatedAbilityDrawCardCharacter,
        14 => CardName::TestFastMultiActivatedAbilityDrawCardCharacter,
        15 => CardName::TestActivatedAbilityDissolveCharacter,
        16 => CardName::TestDualActivatedAbilityCharacter,
        17 => CardName::TestForeseeOne,
        18 => CardName::TestForeseeTwo,
        19 => CardName::TestForeseeOneDrawACard,
        20 => CardName::TestDrawOneReclaim,
        21 => CardName::TestForeseeOneReclaim,
        22 => CardName::TestForeseeOneDrawReclaim,
        23 => CardName::TestReturnVoidCardToHand,
        24 => CardName::TestReturnOneOrTwoVoidEventCardsToHand,
        25 => CardName::TestModalDrawOneOrDrawTwo,
        26 => CardName::TestModalDrawOneOrDissolveEnemy,
        27 => CardName::TestModalReturnToHandOrDrawTwo,
        28 => CardName::TestReturnToHand,
        29 => CardName::TestPreventDissolveThisTurn,
        _ => panic!("Invalid card identity: {}", identity.0),
    }
}
