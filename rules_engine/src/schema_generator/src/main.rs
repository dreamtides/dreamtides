use display_data::request_data::SchemaTypes;
use schemars::generate::SchemaSettings;

fn main() {
    let settings = SchemaSettings::draft07();
    let mut generator = settings.into_generator();
    let schema = generator.root_schema_for::<SchemaTypes>();
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
