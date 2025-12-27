use crate::variables::parser_substitutions;

static PARSER_WORDS: &[&str] = &[
    "abandon",
    "a",
    "allied",
    "and",
    "an",
    "another",
    "any",
    "are",
    "at",
    "battlefield",
    "becomes",
    "card",
    "cards",
    "character",
    "characters",
    "control",
    "cost",
    "deck",
    "disable",
    "discard",
    "draw",
    "end",
    "enemy",
    "event",
    "events",
    "from",
    "gain",
    "gains",
    "hand",
    "has",
    "have",
    "in",
    "it",
    "less",
    "materialized",
    "may",
    "of",
    "once",
    "or",
    "per",
    "play",
    "spark",
    "that",
    "the",
    "them",
    "this",
    "to",
    "top",
    "turn",
    "void",
    "when",
    "whenever",
    "with",
    "you",
    "your",
];

pub fn suggest_directive(name: &str) -> Option<Vec<String>> {
    find_suggestions(name, parser_substitutions::directive_names().collect())
}

pub fn suggest_variable(name: &str) -> Option<Vec<String>> {
    find_suggestions(name, parser_substitutions::variable_names().collect())
}

pub fn suggest_word(word: &str) -> Option<Vec<String>> {
    find_suggestions(word, PARSER_WORDS.to_vec())
}

fn find_suggestions(input: &str, candidates: Vec<&str>) -> Option<Vec<String>> {
    let mut matches: Vec<(usize, &str)> = candidates
        .iter()
        .filter_map(|candidate| {
            let distance = levenshtein(input, candidate);
            if distance <= 3 {
                Some((distance, *candidate))
            } else {
                None
            }
        })
        .collect();

    if matches.is_empty() {
        return None;
    }

    matches.sort_by_key(|(distance, _)| *distance);

    let best_distance = matches[0].0;
    let suggestions: Vec<String> = matches
        .iter()
        .take_while(|(distance, _)| *distance == best_distance)
        .take(5)
        .map(|(_, candidate)| (*candidate).to_string())
        .collect();

    if suggestions.is_empty() {
        None
    } else {
        Some(suggestions)
    }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut previous_row: Vec<usize> = (0..=b_len).collect();
    let mut current_row = vec![0; b_len + 1];

    for i in 1..=a_len {
        current_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            current_row[j] =
                (previous_row[j] + 1).min(current_row[j - 1] + 1).min(previous_row[j - 1] + cost);
        }
        std::mem::swap(&mut previous_row, &mut current_row);
    }

    previous_row[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("test", "test"), 0);
    }

    #[test]
    fn test_levenshtein_one_char_diff() {
        assert_eq!(levenshtein("test", "tent"), 1);
    }

    #[test]
    fn test_levenshtein_empty() {
        assert_eq!(levenshtein("", "test"), 4);
        assert_eq!(levenshtein("test", ""), 4);
    }

    #[test]
    fn test_suggest_directive_close_match() {
        let suggestions = suggest_directive("Judgement");
        assert!(suggestions.is_some());
        let suggestions = suggestions.unwrap();
        assert!(suggestions.contains(&"Judgment".to_string()));
    }

    #[test]
    fn test_suggest_directive_no_match() {
        let suggestions = suggest_directive("CompletelyInvalidDirectiveName");
        assert!(suggestions.is_none());
    }

    #[test]
    fn test_suggest_variable_close_match() {
        let suggestions = suggest_variable("card");
        assert!(suggestions.is_some());
        let suggestions = suggestions.unwrap();
        assert!(suggestions.contains(&"cards".to_string()));
    }

    #[test]
    fn test_suggest_word_close_match() {
        let suggestions = suggest_word("drew");
        assert!(suggestions.is_some());
        let suggestions = suggestions.unwrap();
        assert!(suggestions.contains(&"draw".to_string()));
    }

    #[test]
    fn test_suggest_word_exact_match() {
        let suggestions = suggest_word("draw");
        assert!(suggestions.is_some());
        let suggestions = suggestions.unwrap();
        assert_eq!(suggestions, vec!["draw"]);
    }

    #[test]
    fn test_suggest_word_no_match() {
        let suggestions = suggest_word("xyzabc");
        assert!(suggestions.is_none());
    }
}
