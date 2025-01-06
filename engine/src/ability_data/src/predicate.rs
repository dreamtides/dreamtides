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

use core_data::character_type::CharacterType;
use core_data::numerics::{Energy, Spark};
use serde::{Deserialize, Serialize};

/// Specifies which game object is being effected by a card.
///
/// This is used for both targeting constraints as well as describing the
/// implicit target of an effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    /// Predicate which only matches the owning card.
    This,

    /// Card targeted by this effect. This is typically used for applying
    /// multiple effects to the same target, e.g. "banish a character you
    /// control, then materialize it."
    Target,

    /// Card which triggered an event. Used for applying effects to the
    /// triggering card, e.g. "Whenever you materialize a spirit animal, that
    /// character gains +1 spark."
    That,

    /// Cards controlled by the enemy matching a given predicate.
    Enemy(CardPredicate),

    /// Another card controlled by the owner matching a predicate.
    Another(CardPredicate),

    /// Any card controlled by the owner matching a predicate.
    You(CardPredicate),

    /// Any card matching a predicate.
    Any(CardPredicate),

    /// Any other card matching a predicate, including enemy cards.
    AnyOther(CardPredicate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CardPredicate {
    Card,
    Character,
    Event,
    CharacterType(CharacterType),
    NotCharacterType(CharacterType),
    CharacterWithSpark(Spark, Operator),
    CharacterWithCost(Energy, Operator),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator {
    OrMore,
    Exactly,
    OrLess,
}
