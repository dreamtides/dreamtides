# Implementation Notes for Russian Phraselet System

## Parser Requirements

### Token Types

```rust
enum Token {
    // Literal text
    Text(String),

    // {variable}
    Variable(String),

    // {количество|noun:case}
    NumericAgreement {
        count_var: String,
        noun_key: String,
        case: Option<Case>,
    },

    // {chain:case|adj:key|noun:key}
    AgreementChain {
        case: Case,
        adjectives: Vec<String>,
        noun: String,
    },

    // {verb:key:form} or {verb:key:tense|subject}
    Verb {
        verb_key: String,
        form: VerbForm,
        subject: Option<String>,
    },

    // {@phraselet_id params...}
    PhraseletRef {
        id: String,
        params: HashMap<String, String>,
    },

    // {if:condition|true|false}
    Conditional {
        condition: Condition,
        true_branch: Box<Token>,
        false_branch: Box<Token>,
    },
}
```

### Agreement Resolution Algorithm

```rust
fn resolve_chain(case: Case, adjectives: &[&Adjective], noun: &Noun) -> String {
    let gender = noun.gender;
    let animacy = noun.animacy;
    let number = Number::Singular; // or from context

    let mut result = String::new();

    for adj in adjectives {
        let form = adj.get_form(case, gender, number, animacy);
        result.push_str(&form);
        result.push(' ');
    }

    let noun_form = noun.get_form(case, number);
    result.push_str(&noun_form);

    result
}
```

### Plural Rule Implementation

```rust
fn get_plural_form(count: i64) -> PluralForm {
    let mod100 = (count % 100).abs();
    let mod10 = (count % 10).abs();

    // Special case: 11-14 always genitive plural
    if mod100 >= 11 && mod100 <= 14 {
        return PluralForm::GenitivePlural;
    }

    match mod10 {
        1 => PluralForm::NominativeSingular,
        2 | 3 | 4 => PluralForm::GenitiveSingular,
        _ => PluralForm::GenitivePlural,
    }
}

fn format_numeric(count: i64, noun: &Noun, case: Case) -> String {
    let plural_form = get_plural_form(count);

    // Map plural form to actual case+number
    let (actual_case, number) = match plural_form {
        PluralForm::NominativeSingular => (Case::Nominative, Number::Singular),
        PluralForm::GenitiveSingular => (Case::Genitive, Number::Singular),
        PluralForm::GenitivePlural => (Case::Genitive, Number::Plural),
    };

    // If explicit case override, use it but keep the number
    let final_case = case.unwrap_or(actual_case);

    let noun_form = noun.get_form(final_case, number);
    format!("{} {}", count, noun_form)
}
```

## Data Structure Design

### Noun Storage

```rust
struct Noun {
    key: String,
    gender: Gender,
    animacy: Animacy,
    forms: HashMap<(Case, Number), String>,
}

impl Noun {
    fn get_form(&self, case: Case, number: Number) -> &str {
        self.forms.get(&(case, number))
            .expect(&format!("Missing form {}_{} for noun {}", case, number, self.key))
    }

    // For accusative, handle animacy
    fn get_accusative(&self, number: Number) -> &str {
        if self.animacy == Animacy::Animate &&
           (self.gender == Gender::Masculine || number == Number::Plural) {
            // Animate accusative = genitive
            self.get_form(Case::Genitive, number)
        } else {
            self.get_form(Case::Accusative, number)
        }
    }
}
```

### Adjective Storage

```rust
struct Adjective {
    key: String,
    // Forms indexed by case, gender, number, and animacy (for accusative)
    forms: HashMap<AdjForm, String>,
}

#[derive(Hash, Eq, PartialEq)]
struct AdjForm {
    case: Case,
    gender: Gender,
    number: Number,
    animacy: Option<Animacy>, // Only relevant for accusative
}

impl Adjective {
    fn get_form(&self, case: Case, gender: Gender, number: Number, animacy: Animacy) -> &str {
        // For plural, gender doesn't matter
        let form_gender = if number == Number::Plural { Gender::Masculine } else { gender };

        // Animacy only matters for accusative
        let form_animacy = if case == Case::Accusative {
            Some(animacy)
        } else {
            None
        };

        let key = AdjForm { case, gender: form_gender, number, animacy: form_animacy };
        self.forms.get(&key)
            .expect(&format!("Missing adjective form {:?}", key))
    }
}
```

## Validation

The system should validate at load time:

1. **All noun forms present**: Each noun must have all 12 forms (6 cases x 2 numbers)
2. **All adjective forms present**: Each adjective must have all required forms
3. **Verb conjugations complete**: Based on aspect, verify required forms exist
4. **Phraselet references valid**: All `{@id}` references must exist
5. **Parameter types match**: Count variables must resolve to numbers

```rust
fn validate_declensions(nouns: &[Noun]) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    let required_forms = [
        (Case::Nominative, Number::Singular),
        (Case::Genitive, Number::Singular),
        (Case::Dative, Number::Singular),
        (Case::Accusative, Number::Singular),
        (Case::Instrumental, Number::Singular),
        (Case::Prepositional, Number::Singular),
        (Case::Nominative, Number::Plural),
        (Case::Genitive, Number::Plural),
        (Case::Dative, Number::Plural),
        (Case::Accusative, Number::Plural),
        (Case::Instrumental, Number::Plural),
        (Case::Prepositional, Number::Plural),
    ];

    for noun in nouns {
        for (case, number) in &required_forms {
            if !noun.forms.contains_key(&(*case, *number)) {
                errors.push(ValidationError::MissingNounForm {
                    noun: noun.key.clone(),
                    case: *case,
                    number: *number,
                });
            }
        }
    }

    errors
}
```

## Performance Considerations

1. **Pre-compile phraselets**: Parse all templates at load time, not runtime
2. **Cache agreement lookups**: Gender/animacy checks are frequent
3. **Intern strings**: Noun forms are reused constantly
4. **Lazy phraselet expansion**: Only expand nested references when needed

## Testing Strategy

```rust
#[test]
fn test_plural_rules() {
    // карта (f) - card
    assert_eq!(format_numeric(1, &карта, None), "1 карта");
    assert_eq!(format_numeric(2, &карта, None), "2 карты");
    assert_eq!(format_numeric(5, &карта, None), "5 карт");
    assert_eq!(format_numeric(11, &карта, None), "11 карт");
    assert_eq!(format_numeric(21, &карта, None), "21 карта");
    assert_eq!(format_numeric(22, &карта, None), "22 карты");
    assert_eq!(format_numeric(25, &карта, None), "25 карт");
}

#[test]
fn test_agreement_chain() {
    // "этот персонаж" in various cases
    assert_eq!(
        resolve_chain(Case::Nominative, &[&этот], &персонаж),
        "этот персонаж"
    );
    assert_eq!(
        resolve_chain(Case::Accusative, &[&этот], &персонаж),
        "этого персонажа"  // animate masculine
    );
    assert_eq!(
        resolve_chain(Case::Dative, &[&этот], &персонаж),
        "этому персонажу"
    );
}

#[test]
fn test_animate_accusative() {
    // союзник (m, anim) - ally
    // accusative = genitive for animate masculine
    assert_eq!(союзник.get_accusative(Number::Singular), "союзника");

    // карта (f, inanim) - card
    // accusative is distinct form
    assert_eq!(карта.get_accusative(Number::Singular), "карту");
}
```

## Translator Workflow

1. **Define new nouns**: Add to `declensions.toml` with all 12 forms
2. **Define new adjectives**: Add with all gender/case/number forms
3. **Create phraselets**: Write templates using the syntax
4. **Test output**: Run validation and visual review
5. **Iterate**: Adjust forms based on native speaker feedback

The system is designed so that Russian-speaking translators can work primarily
with the TOML files without needing to understand the Rust implementation.
