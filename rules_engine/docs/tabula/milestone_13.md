# Milestone 13: Tabula Struct & Loading

## Objective

Implement the main `Tabula` struct that holds all loaded card data.

## Tasks

1. Create `Tabula` struct in `tabula_struct.rs`
2. Implement `load()` method that orchestrates all loading
3. Build lookup maps for fast card access
4. Handle loading errors gracefully (skip bad cards, log errors)
5. Cache parsed abilities to avoid re-parsing

## Tabula Struct

```rust
use std::collections::HashMap;

pub struct Tabula {
    pub cards: Vec<CardDefinition>,
    pub dreamwell_cards: Vec<DreamwellCardDefinition>,
    pub card_effects: Vec<CardEffectRow>,
    pub card_lists: Vec<CardListRow>,
    pub strings: FluentStrings,

    // Lookup maps
    card_by_id: HashMap<BaseCardId, usize>,
    dreamwell_by_id: HashMap<BaseCardId, usize>,
}

impl Tabula {
    pub fn get_card(&self, id: BaseCardId) -> Option<&CardDefinition> {
        self.card_by_id.get(&id).map(|&i| &self.cards[i])
    }

    pub fn get_dreamwell(&self, id: BaseCardId) -> Option<&DreamwellCardDefinition> {
        self.dreamwell_by_id.get(&id).map(|&i| &self.dreamwell_cards[i])
    }
}
```

## Loading Orchestration

```rust
impl Tabula {
    pub fn load(source: TabulaSource, base_path: &Path) -> Result<Self, TabulaError> {
        // 1. Load Fluent strings
        let strings = FluentStrings::load(&base_path.join("strings.ftl"))?;
        let rules_ftl = FluentStrings::load(&base_path.join("strings.ftl"))?;

        // 2. Create parser (once for all cards)
        let parser = AbilityParser::new();
        let builder = CardDefinitionBuilder::new(&parser, &rules_ftl);

        // 3. Load and build cards
        let cards_file = source.cards_filename();
        let raw_cards = toml_loader::load_cards(&base_path.join(cards_file))?;
        let cards = raw_cards
            .into_iter()
            .filter_map(|raw| match builder.build(raw) {
                Ok(card) => Some(card),
                Err(e) => { eprintln!("Warning: {e}"); None }
            })
            .collect();

        // 4. Load dreamwell cards similarly
        // 5. Load card effects and lists
        // 6. Build lookup maps

        Ok(Self { cards, /* ... */ })
    }
}
```

## Error Handling Strategy

Skip cards that fail to build rather than failing entire load:

```rust
let cards: Vec<CardDefinition> = raw_cards
    .into_iter()
    .filter_map(|raw| {
        match builder.build(raw.clone()) {
            Ok(card) => Some(card),
            Err(e) => {
                log::warn!(
                    "Skipping card {:?}: {}",
                    raw.name.as_deref().unwrap_or("unknown"),
                    e
                );
                None
            }
        }
    })
    .collect();
```

## Testing

```rust
#[test]
fn test_tabula_load_production() {
    let tabula = Tabula::load(TabulaSource::Production, tabula_path()).unwrap();
    assert!(tabula.cards.len() > 50);
}

#[test]
fn test_card_lookup_by_id() {
    let tabula = Tabula::load(TabulaSource::Test, test_path()).unwrap();
    let card = tabula.get_card(test_card::TEST_DRAW_TWO);
    assert!(card.is_some());
}
```

## Verification

- `cargo test -p tabula_data_v2` passes
- All production cards load successfully
- Lookup by ID works correctly

## Context Files

1. `src/tabula_data/src/tabula.rs` - V1 Tabula struct
2. `src/state_provider/src/state_provider.rs` - Loading patterns
3. `docs/tabula/tabula_v2_design_document.md` - Architecture overview
