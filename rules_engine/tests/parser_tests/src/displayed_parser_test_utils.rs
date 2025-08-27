use ability_data::ability::DisplayedAbility;
use parser::{ability_parser, displayed_ability_parser};

pub fn parse_displayed(text: &str) -> Vec<DisplayedAbility> {
    let input = text.to_lowercase();
    let abilities = ability_parser::parse(&input).expect("ability parse failed");
    displayed_ability_parser::parse_with(&abilities, &input).expect("displayed parse failed")
}
