use ability_data::ability::{
    Ability, DisplayedAbility, DisplayedAbilityEffect, DisplayedEventAbility,
    DisplayedModalEffectChoice,
};
use core_data::initialization_error::{ErrorCode, InitializationError};

pub fn parse_with(
    abilities: &[Ability],
    input: &str,
) -> Result<Vec<DisplayedAbility>, Vec<InitializationError>> {
    let input = input.to_lowercase();
    let blocks = split_blocks(&input);
    if abilities.len() != blocks.len() {
        return Err(vec![InitializationError::with_details(
            ErrorCode::AbilityParsingError,
            "Displayed ability parser mismatch",
            format!(
                "abilities count {} does not match text blocks count {}",
                abilities.len(),
                blocks.len()
            ),
        )]);
    }

    let mut out = Vec::with_capacity(abilities.len());
    let mut errs = Vec::new();

    for (ability, block) in abilities.iter().zip(blocks.iter()) {
        match ability {
            Ability::Static(_) => out.push(DisplayedAbility::Static { text: (*block).to_string() }),
            Ability::Named(_) => {
                out.push(DisplayedAbility::Named { name: block.trim_end_matches('.').to_string() });
            }
            Ability::Triggered(_) => {
                out.push(DisplayedAbility::Triggered { text: (*block).to_string() });
            }
            Ability::Event(_) => match to_effect(extract_effect_tail(block)) {
                Ok(effect) => out.push(DisplayedAbility::Event(DisplayedEventAbility { effect })),
                Err(e) => errs.push(e),
            },
            Ability::Activated(_) => {
                let cost = extract_activated_cost(block).unwrap_or_default();
                match to_effect(extract_effect_tail(block)) {
                    Ok(effect) => out.push(DisplayedAbility::Activated { cost, effect }),
                    Err(e) => errs.push(e),
                }
            }
        }
    }

    if errs.is_empty() { Ok(out) } else { Err(errs) }
}

fn split_blocks(text: &str) -> Vec<&str> {
    text.split("\n\n").map(|b| b.trim()).filter(|b| !b.is_empty()).collect()
}

/// Finds a : character that is not inside parentheses or braces.
fn find_top_level_colon(text: &str) -> Option<usize> {
    let mut paren = 0_i32;
    let mut brace = 0_i32;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' => paren += 1,
            ')' => paren -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            ':' if paren == 0 && brace == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn extract_activated_cost(block: &str) -> Option<String> {
    let colon = find_top_level_colon(block)?;
    let head = block[..colon].trim();
    let head = head
        .strip_prefix("{fa} ")
        .or_else(|| head.strip_prefix("{fma} "))
        .or_else(|| head.strip_prefix("{ma} "))
        .or_else(|| head.strip_prefix("{a} "))
        .unwrap_or(head);
    Some(head.trim().to_string())
}

fn extract_effect_tail(block: &str) -> &str {
    if block.contains("{choose-one}") {
        return block.trim();
    }
    if let Some(idx) = find_top_level_colon(block) {
        return block[idx + 1..].trim();
    }
    block.trim()
}

#[expect(clippy::result_large_err)]
fn to_effect(effect_text: &str) -> Result<DisplayedAbilityEffect, InitializationError> {
    if is_modal(effect_text) {
        let choices = parse_modal_choices(effect_text)
            .into_iter()
            .map(|line| split_modal_cost_and_effect(&line))
            .collect::<Vec<_>>();
        if choices.is_empty() {
            return Err(InitializationError::with_details(
                ErrorCode::AbilityParsingError,
                "Failed to parse modal choices",
                format!("no modal choices found in block: {effect_text}"),
            ));
        }
        return Ok(DisplayedAbilityEffect::Modal(choices));
    }

    if effect_text.is_empty() {
        return Err(InitializationError::with_details(
            ErrorCode::AbilityParsingError,
            "Empty effect text",
            format!("no effect text found in block: {effect_text}"),
        ));
    }
    Ok(DisplayedAbilityEffect::Effect(effect_text.to_string()))
}

fn is_modal(block: &str) -> bool {
    block.contains("{choose-one}") && block.contains("{bullet}")
}

fn parse_modal_choices(block: &str) -> Vec<String> {
    let mut results = Vec::new();
    for line in block.lines() {
        if let Some(rest) = line.trim().strip_prefix("{bullet}") {
            results.push(rest.trim().to_string());
        }
    }
    if results.is_empty() {
        let mut remaining = block;
        while let Some(pos) = remaining.find("{bullet}") {
            let after = &remaining[pos + 8..];
            if let Some(next_pos) = after.find("{bullet}") {
                results.push(after[..next_pos].trim().to_string());
                remaining = &after[next_pos..];
            } else {
                results.push(after.trim().to_string());
                break;
            }
        }
    }
    results
}

fn split_modal_cost_and_effect(line: &str) -> DisplayedModalEffectChoice {
    if let Some(idx) = find_top_level_colon(line) {
        let cost = line[..idx].trim().to_string();
        let effect = line[idx + 1..].trim().to_string();
        return DisplayedModalEffectChoice { cost, effect };
    }
    DisplayedModalEffectChoice { cost: String::new(), effect: line.trim().to_string() }
}
