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
use core_data::character_type::CharacterType;
use core_data::numerics::{Energy, Spark};

#[derive(Debug, Clone)]
pub enum Predicate {
    /// Predicate which only matches the owning card.
    This,

    /// Cards controlled by the enemy matching a given predicate
    Enemy(CardPredicate),

    /// Another card controlled by the owner matching a predicate
    AnotherYouControl(CardPredicate),

    /// Any card controlled by the owner matching a predicate
    You(CardPredicate),

    /// Any card matching a predicate
    AnyMatching(CardPredicate),

    /// Any other card matching a predicate
    AnyOther(CardPredicate),
}

#[derive(Debug, Clone)]
pub enum CardPredicate {
    Card,
    Character,
    Event,
    CharacterType(CharacterType),
    NotCharacterType(CharacterType),
    CharacterWithSpark(Operator, Spark),
    CharacterWithCost(Operator, Energy),
}

#[derive(Debug, Clone)]
pub enum Operator {
    LessThan,
    LessThanOrEqualTo,
    EqualTo,
    GreaterThanOrEqualTo,
    GreaterThan,
}
