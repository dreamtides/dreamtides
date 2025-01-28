use display_data::command::CommandSequence;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(CommandSequence);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
