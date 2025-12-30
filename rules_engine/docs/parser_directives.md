# Parser Directives Reference

This document provides a complete reference for directives used in Dreamtides
rules text, based on `rules_text_sorted.json`.

---

## Trigger Keywords

| Directive | Example |
|-----------|---------|
| `{Judgment}` | `{Judgment} Draw {cards}.` |
| `{Materialized}` | `{Materialized} Gain {e}.` |
| `{Dissolved}` | `{Dissolved} Draw {cards}.` |
| `{MaterializedJudgment}` | `{MaterializedJudgment} Gain {e}.` |
| `{MaterializedDissolved}` | `{MaterializedDissolved} Draw {cards}.` |

---

## Action Verbs (Capitalized)

| Directive | Example |
|-----------|---------|
| `{Dissolve}` | `{Dissolve} an enemy.` |
| `{Banish}` | `{Banish} an enemy with cost {e} or less.` |
| `{Discover}` | `{Discover} {a-subtype}.` |
| `{Prevent}` | `{Prevent} a card.` |
| `{Foresee}` | `{Foresee}.` |
| `{Materialize}` | `{Materialize} it.` |
| `{Reclaim}` | `{Reclaim} this character.` |
| `{Kindle}` | `{Kindle}.` |

---

## Action Verbs (Lowercase - in context)

| Directive | Example |
|-----------|---------|
| `{dissolve}` | `when you {dissolve} an ally` |
| `{banish}` | `{banish} {up-to-n-allies}` |
| `{materialize}` | `when you {materialize} a character` |
| `{kindle}` | `once per turn, {kindle}` |
| `{reclaim}` | `it gains {reclaim}` |
| `{foresee}` | `When you play an event, {foresee}.` |

---

## Numeric Variables

| Directive | Purpose | Example |
|-----------|---------|---------|
| `{e}` | Energy amount | `Gain {e}.` with `e: 3` |
| `{cards}` | Card count | `Draw {cards}.` with `cards: 2` |
| `{discards}` | Discard count | `Discard {discards}.` |
| `{points}` | Points | `Gain {points}.` |
| `{s}` | Spark | `+{s} spark` |
| `{count}` | Generic count | `with {count} or more cards` |

---

## Subtype Variables

| Directive | Example |
|-----------|---------|
| `{subtype}` | `allied {subtype}` with `subtype: Warrior` |
| `{plural-subtype}` | `allied {plural-subtype}` |
| `{a-subtype}` | `{Discover} {a-subtype}.` |

---

## Count Expressions

| Directive | Example |
|-----------|---------|
| `{count-allies}` | `Abandon {count-allies}:` |
| `{count-allied-subtype}` | `With {count-allied-subtype},` |
| `{cards-numeral}` | `play {cards-numeral} in a turn` |
| `{top-n-cards}` | `the {top-n-cards} of your deck` |

---

## Collection Expressions

| Directive | Example |
|-----------|---------|
| `{up-to-n-allies}` | `{banish} {up-to-n-allies}` |
| `{up-to-n-events}` | `Return {up-to-n-events} from your void` |
| `{n-random-characters}` | `{Materialize} {n-random-characters}` |
| `{n-figments}` | `{Materialize} {n-figments}.` |
| `{a-figment}` | `{materialize} {a-figment}` |

---

## Named Abilities

| Directive | Example |
|-----------|---------|
| `{ReclaimForCost}` | `\n\n{ReclaimForCost}` |
| `{Fast}` | `{Fast} -- cost: effect` |
| `{reclaim-for-cost}` | `gains {reclaim-for-cost}` |

---

## Modal Markers

| Directive | Example |
|-----------|---------|
| `{ChooseOne}` | `{ChooseOne}\n{bullet}...` |
| `{bullet}` | `{bullet} {e}: effect` |

---

## Other Directives

| Directive | Purpose |
|-----------|---------|
| `{fast}` | Card with fast ability |
| `{energy-symbol}` | Energy symbol reference |
| `{spark}` | Spark value reference |
| `{it-or-them}` | Pronoun for singular/plural |
| `{this-turn-times}` | Turn count reference |
| `{JudgmentPhaseName}` | Phase name reference |
| `{maximum-energy}` | Maximum energy reference |

---

## Common Patterns in Rules Text

Study these patterns from `rules_text_sorted.json` to inform parser design.

### Simple Effects

```
Draw a card.
Gain {e}.
Draw {cards}.
```

### Triggered Abilities with Keywords

```
{Judgment} Gain {e}.
{Materialized} Draw {cards}.
{MaterializedJudgment} {Kindle}.
```

### Effects with Targets

```
{Dissolve} an enemy.
{Dissolve} an enemy with spark {s} or less.
{Banish} an enemy with cost {e} or less.
```

### Compound Effects

```
Gain {e}. Draw {cards}.
Draw {cards}. Discard {discards}.
Discard {discards}. Draw {cards}.
{Dissolve} an enemy. You lose {points}.
```

### Triggers with Conditions

```
When you discard a card, gain {points}.
When you play an event, gain {e}.
When you {materialize} an ally, gain {e}.
Once per turn, when you {materialize} a character, gain {e}.
```

### Activated Abilities

```
{e}: Draw {cards}.
Abandon an ally: Gain {e}.
{e}, Discard {discards}: {kindle}.
```

### Static Abilities

```
Events cost you {e} less.
Allied {plural-subtype} have +{s} spark.
This character's spark is equal to the number of cards in your void.
```

### Complex Patterns

```
{Judgment} You may discard {discards} to draw {cards} and gain {points}.
{Materialize} {n-random-characters} with cost {e} or less from your deck.
{ChooseOne}
{bullet} {e}: Return an enemy to hand.
{bullet} {e-}: Draw {cards}.
```

---

## Parser Architecture Overview

For the complete parser architecture, see `parser_v2_design.md`. Summary:

### Processing Flow

```
INPUT (rules_text + variables)
    ↓
LEXER → Vec<Token> (Word, Directive, Punctuation, Newline)
    ↓
VARIABLE RESOLVER → Vec<ResolvedToken>
    ↓
CHUMSKY PARSER → ParsedAbility with spans
    ↓
ABILITY BUILDER → Ability + SpannedAbility
```

### Token System

```rust
pub enum Token {
    Word(String),      // English word or symbol
    Directive(String), // Braced directive: {Judgment}, {cards}
    Period,            // Sentence terminator
    Comma,             // Clause separator
    Colon,             // Cost/effect separator
    Newline,           // Ability separator
}

pub enum ResolvedToken {
    Token(Token),                              // Non-variable token
    Integer { directive: String, value: u32 }, // Resolved number
    Subtype { directive: String, subtype: CardSubtype },
    FigmentCount { count: u32, figment_type: FigmentType },
    FigmentSingle { figment_type: FigmentType },
}
```

### Crate Structure

```
rules_engine/src/parser_v2/
├── src/
│   ├── lexer/           # Tokenization
│   ├── variables/       # Variable resolution
│   ├── parser/          # Chumsky parsers
│   │   └── effect/      # Effect parsers by category
│   ├── builder/         # Ability construction + spans
│   ├── serializer/      # Ability → String
│   └── error/           # Error handling + suggestions

rules_engine/tests/parser_v2_tests/
├── src/test_helpers.rs  # Test utilities
└── tests/               # Test files
```
