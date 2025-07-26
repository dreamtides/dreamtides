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
#[serde(rename_all = "camelCase")]
pub struct UserId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct BattleId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct QuestId(pub Uuid);

/// Identifies a named card with given rules text.
///
/// Two cards with the same identity are considered to be "the same card", in
/// the same sense that two copies of Lightning Bolt are "the same card" in
/// Magic even though they may be in different game positions.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct CardIdentity(pub Uuid);

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
#[serde(rename_all = "camelCase")]
pub struct AbilityNumber(pub usize);
