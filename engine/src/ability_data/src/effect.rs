/*
 * Copyright (c) dreamcaller 2025-present
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::condition::Condition;

/// Provides a sequence of effects to apply, as well as modifiers which affect how those
/// effects are applied.
#[derive(Default, Debug, Clone)]
pub struct EffectList {
    /// Sequences of effects to apply in the provided order, usually written as complete sentences
    /// or separated by the words "then" or "and" to indicate order.
    pub effects: Vec<Effect>,

    /// True if this is an effect which the controller may choose to apply, usually
    /// prefixed with "You may..."
    pub optional: bool,

    /// Indicates an effect set which occurs only if some condition is met, usually phrased as
    /// "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

impl EffectList {
    /// Creates a new effect list containing a single provided effect.
    pub fn single(effect: Effect) -> Self {
        Self {
            effects: vec![effect],
            ..Self::default()
        }
    }
}

/// Effects are the primary way in which cards modify the game state. This can be as part of the
/// resolution of an event card, or via the effect text of a triggered or activated ability on
/// a character card.
#[derive(Debug, Clone)]
pub enum Effect {
    /// Draw N cards.
    DrawCards(u32),
    ThisCharacterGainsPlus1Spark,
}
