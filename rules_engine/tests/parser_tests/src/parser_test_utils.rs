use ability_data::ability::Ability;
use parser::ability_parser;

#[expect(clippy::print_stderr)]
pub fn parse(text: &str) -> Vec<Ability> {
    let input = text.to_lowercase();
    let result = ability_parser::parse(&input);
    match result {
        Ok(output) => output,
        Err(errs) => {
            for e in &errs {
                eprintln!("{e:?}");
            }
            panic!("Error parsing input! {errs:?}");
        }
    }
}
