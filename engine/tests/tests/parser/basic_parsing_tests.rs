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

use ability_data::ability::Ability;
use insta::assert_ron_snapshot;
use parser::ability_parser;

fn parse(text: &str) -> Ability {
    match ability_parser::parse(text) {
        Ok(parsed) => parsed,
        Err(errors) => panic!("Parse error: {:?}", errors),
    }
}

#[test]
fn test_materialize_warrior_gain_spark() {
    let result = parse("Whenever you materialize another warrior, this character gains +1 spark.");
    assert_ron_snapshot!(
        result,
        @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Another(CharacterType(Warrior))),
      effect: Effect(GainSpark(This, Spark(1))),
    ))
    "###
    );
}
