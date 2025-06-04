#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptContext {
    SelectTargetCharacter,
    SelectTargetStackCard,
    PayCostToPreventNegation,
    PickAdditionalEnergyCost,
}
