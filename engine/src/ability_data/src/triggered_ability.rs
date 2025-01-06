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

use serde::{Deserialize, Serialize};

use crate::effect::Effect;
use crate::trigger_event::TriggerEvent;

/// A triggered ability is an effect which happens when some triggering
/// event occurs, typically while its card is in play. Indicated in card
/// text by "When", "Whenever", "At", or by a trigger keyword.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredAbility {
    pub trigger: TriggerEvent,
    pub effect: Effect,
}

impl TriggeredAbility {
    pub fn new(trigger: TriggerEvent, effect: Effect) -> Self {
        Self { trigger, effect }
    }
}
