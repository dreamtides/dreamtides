use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies a human player of the game.
///
/// Equivalently, this identifies a save file.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    JsonSchema,
)]
pub struct UserId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct BattleId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct QuestId(pub Uuid);

/// Identifies a card with given rules text, i.e. a base card and a set of
/// card modifications.
///
/// Two cards with the same identity are considered to be "the same card", in
/// the same sense that two copies of Lightning Bolt are "the same card" in
/// Magic. Those two cards would have the same CardIdentity, but different
/// `CardId`s.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct CardIdentity(pub usize);

/// Identifies the base rules for a card.
///
/// The 'base card' describes the abilities of a card before any modifications
/// are applied to it, i.e. a card that appears in the Tabula database. A base
/// card with zero or more modifications forms a [CardIdentity], and an instance
/// of that card within a battle has a `CardId`.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct BaseCardId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub enum CardName {
    TestVanillaCharacter,
    TestDissolve,
    TestNamedDissolve,
    TestCounterspellUnlessPays,
    TestCounterspell,
    TestCounterspellCharacter,
    TestVariableEnergyDraw,
    TestDrawOne,
    TestTriggerGainSparkWhenMaterializeAnotherCharacter,
    TestTriggerGainSparkOnPlayCardEnemyTurn,
    TestTriggerGainTwoSparkOnPlayCardEnemyTurn,
    TestActivatedAbilityDrawCard,
    TestMultiActivatedAbilityDrawCardCharacter,
    TestFastActivatedAbilityDrawCardCharacter,
    TestFastMultiActivatedAbilityDrawCardCharacter,
    TestActivatedAbilityDissolveCharacter,
    TestDualActivatedAbilityCharacter,
    TestForeseeOne,
    TestForeseeTwo,
    TestForeseeOneDrawACard,
    TestDrawOneReclaim,
    TestForeseeOneReclaim,
    TestForeseeOneDrawReclaim,
    TestReturnVoidCardToHand,
    TestReturnOneOrTwoVoidEventCardsToHand,
    TestModalDrawOneOrDrawTwo,
    TestModalDrawOneOrDissolveEnemy,
    TestModalReturnToHandOrDrawTwo,
    TestReturnToHand,
    TestPreventDissolveThisTurn,
}

/// Number of an ability within a card.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct AbilityNumber(pub usize);
