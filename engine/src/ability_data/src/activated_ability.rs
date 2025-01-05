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
use crate::cost::Cost;
use crate::effect::EffectList;

/// An activated ability is present on a character card and allows the controlling player
/// to pay some cost in order to achieve an effect. This is written as "> cost: effect".
pub struct ActivatedAbility {
    /// Cost to activate this ability, paid before it is put on the stack.
    pub cost: Cost,

    /// Effect of this ability, applied as it resolves on the stack.
    pub effect: EffectList,

    /// True if this ability can be activated in response to enemy game actions.
    pub is_fast: bool,

    /// True if this ability can be used on the turn in which its controlling character was played.
    pub is_immediate: bool,

    /// True if this ability can be used multiple times per turn.
    pub is_multi: bool,
}
