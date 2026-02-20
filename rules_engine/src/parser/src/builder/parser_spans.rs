use chumsky::span::SimpleSpan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedText {
    pub text: String,
    pub span: SimpleSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpannedAbility {
    Event(SpannedEventAbility),
    Static { text: SpannedText },
    Activated(SpannedActivatedAbility),
    Triggered(SpannedTriggeredAbility),
    Named { name: SpannedText },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedEventAbility {
    pub additional_cost: Option<SpannedText>,
    pub effect: SpannedEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedActivatedAbility {
    pub cost: SpannedText,
    pub effect: SpannedEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedTriggeredAbility {
    pub until_end_of_turn: Option<SpannedText>,
    pub once_per_turn: Option<SpannedText>,
    pub trigger: SpannedText,
    pub effect: SpannedEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpannedEffect {
    Effect(SpannedText),
    Modal(Vec<SpannedModalEffectChoice>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedModalEffectChoice {
    pub cost: SpannedText,
    pub effect: SpannedText,
}

impl SpannedText {
    pub fn new(text: String, span: SimpleSpan) -> Self {
        Self { text, span }
    }
}
