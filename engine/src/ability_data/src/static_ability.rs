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

use core_data::numerics::Energy;
use serde::{Deserialize, Serialize};

use crate::predicate::CardPredicate;

/// A static ability represents something which modifies the rules of the game,
/// either for this specific card or globally. Static abilities do not 'happen',
/// they're just something that is always true.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StaticAbility {
    EnemyAddedCostToPlay(CardPredicate, Energy),
    OncePerTurnPlayFromVoid(CardPredicate),
}
