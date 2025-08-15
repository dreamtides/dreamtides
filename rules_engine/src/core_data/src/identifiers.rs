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

/// Identifies a card with given rules text.
///
/// Two cards with the same identity are considered to be "the same card", in
/// the same sense that two copies of Lightning Bolt are "the same card" in
/// Magic. Those two cards would have the same CardIdentity, but different
/// CardIds.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct CardIdentity(pub usize);

impl CardIdentity {
    pub fn tmp_to_card_name(self) -> CardName {
        match self.0 {
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
            _ => panic!("Invalid card identity: {}", self.0),
        }
    }
}

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

impl CardName {
    pub fn tmp_to_card_identity(self) -> CardIdentity {
        CardIdentity(self as usize)
    }
}

/// Number of an ability within a card.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
pub struct AbilityNumber(pub usize);
