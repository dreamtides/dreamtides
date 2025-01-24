use display_data::battle_view::BattleView;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(BattleView);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
