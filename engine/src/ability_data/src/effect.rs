// Copyright (c) dreamcaller 2025-present
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core_data::numerics::Spark;
use serde::{Deserialize, Serialize};

use crate::condition::Condition;
use crate::predicate::Predicate;

/// Represents a mutation to the game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Effect(GameEffect),
    EffectList(EffectList),
}

/// Provides a sequence of effects to apply, as well as modifiers which affect
/// how those effects are applied.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EffectList {
    /// Sequences of effects to apply in the provided order, usually written as
    /// complete sentences or separated by the words "then" or "and" to
    /// indicate order.
    pub effects: Vec<GameEffect>,

    /// True if this is an effect which the controller may choose to apply,
    /// usually prefixed with "You may..."
    pub optional: bool,

    /// Indicates an effect set which occurs only if some condition is met,
    /// usually phrased as "If {condition}, {effect}"
    pub condition: Option<Condition>,
}

/// Effects are the primary way in which cards modify the game state. This can
/// be as part of the resolution of an event card, or via the effect text of a
/// triggered or activated ability on a character card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEffect {
    /// Controller draws N cards.
    DrawCards(u32),

    /// Matching character gains spark.
    GainSpark(Predicate, Spark),
}
