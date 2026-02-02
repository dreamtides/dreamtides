# Appendix A: English Localization Guide

## Overview

English is typically the source language for Phraselet projects. This appendix covers English-specific patterns and how they map to the DSL.

## Article Selection (a/an)

### Automatic Detection

Phraselet automatically selects "a" or "an" based on the first letter of the following word:

```rust
phraselet! {
    noun card = "card" / "cards"           // "a card"
    noun ally = "ally" / "allies"          // "an ally"
    noun character = "character" / ...     // "a character"
    noun event = "event" / "events"        // "an event"
}
```

**Rule:** Vowel letters (a, e, i, o, u) → "an"; consonant letters → "a"

### Override for Pronunciation

Some words have pronunciation that differs from their spelling:

```rust
phraselet! {
    // Starts with vowel letter but consonant sound
    noun uniform = a "uniform" / "uniforms"    // "a uniform" (not "an")
    noun unicorn = a "unicorn" / "unicorns"
    noun useful = a "useful item" / ...

    // Starts with consonant letter but vowel sound
    noun hour = an "hour" / "hours"            // "an hour" (not "a")
    noun honor = an "honor" / "honors"
    noun heir = an "heir" / "heirs"
}
```

### Dynamic Context

When the article depends on a variable value:

```rust
phraselet! {
    // The target could be "ally" (an) or "character" (a)
    dissolve_target(target: Predicate) = "Dissolve {target.a}."

    // .a automatically picks the right article based on what target expands to
}
```

### Handling Adjectives

Articles go before adjectives, so Phraselet looks at the adjective, not the noun:

```rust
phraselet! {
    // "a fast character" not "an fast character"
    // "an allied character" not "a allied character"

    predicate fast_character = "fast {character}"        // "a fast character"
    predicate allied_character = "allied {character}"    // "an allied character"

    // When using {predicate.a}, the article matches the first word
    target_fast = "Target {fast_character.a}."    // "Target a fast character."
    target_ally = "Target {allied_character.a}."  // "Target an allied character."
}
```

---

## Pluralization

### Standard Rules

English has two plural categories: singular (exactly 1) and plural (everything else):

```rust
phraselet! {
    // Automatic: add "s"
    noun card = "card" / "cards"

    // Automatic: add "es" after s, x, z, ch, sh
    noun match = "match" / "matches"

    // Explicit irregular plural
    noun ally = "ally" / "allies"
    noun enemy = "enemy" / "enemies"
    noun child = "child" / "children"
}
```

### Count-Sensitive Messages

```rust
phraselet! {
    // Option 1: Use .count() for automatic selection
    draw_cards(n: Int) = "Draw {n} {card.count(n)}."
    // n=1: "Draw 1 card."
    // n=5: "Draw 5 cards."

    // Option 2: Use match for different sentence structures
    draw_cards(n: Int) = match n {
        1 => "Draw a card."
        _ => "Draw {n} cards."
    }
}
```

### Zero Handling

In English, zero takes the plural form:

```rust
phraselet! {
    remaining(n: Int) = match n {
        0 => "No cards remaining."
        1 => "1 card remaining."
        _ => "{n} cards remaining."
    }

    // Or using .count() - zero gets plural form automatically
    simple_remaining(n: Int) = "{n} {card.count(n)} remaining."
    // n=0: "0 cards remaining."
}
```

---

## Ownership Transforms

Game text often needs to express ownership differently:

```rust
phraselet! {
    // Your controlled cards use friendlier terms
    transform your {
        character => "ally" / "allies"
        card => "your card" / "your cards"
        event => "your event" / "your events"
    }

    // Enemy cards use hostile terms
    transform enemy {
        character => "enemy" / "enemies"
        card => "enemy card" / "enemy cards"
    }

    // Another (for self-referential cards)
    transform another {
        character => "ally" / "allies"  // Same as "your"
    }

    // Usage
    dissolve_yours = "Dissolve {your.character.a}."     // "Dissolve an ally."
    dissolve_enemy = "Dissolve {enemy.character.a}."    // "Dissolve an enemy."
    dissolve_another = "Dissolve {another.character.a}." // "Dissolve an ally."
}
```

---

## Predicate Patterns

Complex targeting uses predicates that combine nouns with constraints:

```rust
phraselet! {
    // Base predicates
    predicate any_character = "{character}"
    predicate any_card = "{card}"

    // With ownership
    predicate your_character = "{your.character}"    // "ally"
    predicate enemy_character = "{enemy.character}"  // "enemy"

    // With constraints
    predicate character_with_spark(s: Int, op: Operator) =
        "{character} with spark {s}{op}"
    // "character with spark 3 or more"

    predicate ally_with_cost(c: Int, op: Operator) =
        "{your.character} with cost {c}{op}"
    // "ally with cost 2 or less"

    // Nested constraints
    predicate fast_character = "fast {character}"
    predicate character_not_subtype(subtype: Subtype) =
        "{character} that is not {subtype.a}"
    // "character that is not a Warrior"
}
```

---

## Effect Patterns

### Simple Effects

```rust
phraselet! {
    // Direct effects
    draw(n: Int) = "Draw {n} {card.count(n)}."
    discard(n: Int) = "Discard {n} {card.count(n)}."

    // With targets
    dissolve(target: Predicate) = "{Dissolve} {target.a}."
    return_to_hand(target: Predicate) = "Return {target.a} to hand."

    // Gains
    gain_spark(target: Predicate, s: Int) = "{target} gains +{s} spark."
    gain_energy(n: Int) = "Gain {n} {energy}."
}
```

### Compound Effects

```rust
phraselet! {
    // Sequential
    draw_then_discard(draw: Int, discard: Int) =
        "Draw {draw} {card.count(draw)}, then discard {discard} {card.count(discard)}."

    // Optional
    may_draw(n: Int) = "You may draw {n} {card.count(n)}."

    // Conditional
    draw_if(n: Int, condition: Condition) =
        "{condition} draw {n} {card.count(n)}."
}
```

### Collection Expressions

```rust
phraselet! {
    dissolve_collection(target: Predicate, collection: Collection) = match collection {
        exactly(1)    => "{Dissolve} {target.a}."
        exactly(n)    => "{Dissolve} {n} {target.plural}."
        up_to(n)      => "{Dissolve} up to {n} {target.plural}."
        all           => "{Dissolve} all {target.plural}."
        any_number    => "{Dissolve} any number of {target.plural}."
        all_but_one   => "{Dissolve} all but one {target.singular}."
        each_other    => "{Dissolve} each other {target.singular}."
    }
}
```

---

## Keywords and Formatting

### Defining Keywords

```rust
phraselet! {
    // Simple keywords (formatted)
    keyword dissolve = "<k>dissolve</k>"
    keyword Dissolve = "<k>Dissolve</k>"
    keyword reclaim = "<k>reclaim</k>"
    keyword Reclaim = "<k>Reclaim</k>"

    // Keywords with parameters
    keyword foresee(n: Int) = "<k>Foresee</k> {n}"
    keyword kindle(k: Int) = "<k>Kindle</k> {k}"

    // Keywords with costs
    keyword reclaim_for(cost: Int) = "<k>Reclaim</k> <e>{cost}</e>"
}
```

### Using Keywords

```rust
phraselet! {
    // Keywords auto-format when used
    effect_dissolve(target: Predicate) = "{Dissolve} {target.a}."
    // Produces: "<k>Dissolve</k> an ally."

    effect_foresee(n: Int) = "{foresee(n)}."
    // Produces: "<k>Foresee</k> 3."

    gains_reclaim(target: Predicate, cost: Option<Int>) = match cost {
        some(c) => "{target} gains {reclaim_for(c)}."
        none    => "{target} gains {reclaim} equal to its cost."
    }
}
```

---

## Capitalization

### Sentence Starts

The first letter of effect text should be capitalized:

```rust
phraselet! {
    // Use capitalized keyword variant at sentence start
    effect_dissolve = "{Dissolve} an enemy."

    // Use lowercase keyword mid-sentence
    may_dissolve = "You may {dissolve} an enemy."
}
```

### Keyword Capitalization

Some keywords are always capitalized (proper nouns), others follow sentence position:

```rust
phraselet! {
    // Position-sensitive
    keyword dissolve = "<k>dissolve</k>"   // mid-sentence
    keyword Dissolve = "<k>Dissolve</k>"   // sentence start

    // Always capitalized (game terms)
    keyword Judgment = "<k>Judgment</k>"   // proper noun
    keyword Aegis = "<k>Aegis</k>"
}
```

---

## Conditional Text

### Boolean Conditions

```rust
phraselet! {
    gains_reclaim(target: Predicate, this_turn: Bool) =
        "{target} gains {reclaim}{if this_turn: this turn}."
    // this_turn=true:  "It gains reclaim this turn."
    // this_turn=false: "It gains reclaim."
}
```

### With Else Clause

```rust
phraselet! {
    targeting(optional: Bool) =
        "{if optional: You may target | Target} a character."
    // optional=true:  "You may target a character."
    // optional=false: "Target a character."
}
```

---

## Complete Card Example

```rust
phraselet! {
    // Card: "Arcane Scholar"
    // Text: "Materialized: Draw 2 cards. Judgment: If you have 5 or more cards
    //        in your void, draw a card."

    arcane_scholar_materialized = "{Materialized}: Draw 2 cards."

    arcane_scholar_judgment = """
        {Judgment}: If you have 5 or more cards in your void, draw a card.
        """

    // More complex example:
    // "Dissolved: Return an ally with cost 2 or less from your void to your hand."

    noun ally_in_void(cost: Int, op: Operator) =
        "{your.character} with cost {cost}{op} in your void"

    return_from_void(target: Predicate) =
        "Return {target.a} to your hand."

    complex_dissolved = "{Dissolved}: {return_from_void(ally_in_void(2, or_less))}."
}
```
