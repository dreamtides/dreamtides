use display_data::request_data::SchemaTypes;
use schemars::schema_for;

fn main() {
    let schema = schema_for!(SchemaTypes);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
