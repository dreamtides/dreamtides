# Ability Serializer Localization Strategy

## Overview

This document describes a comprehensive localization strategy for the Dreamtides card ability serializer system using [Mozilla Fluent](https://projectfluent.org). The serializer generates complex, dynamic rules text that must be translated into multiple languages while preserving grammatical correctness across different morphosyntactic systems.

## Current Architecture Analysis

The serializer (`src/parser/src/serializer/`) generates English rules text through composable functions:

```rust
// Current English-only approach
pub fn serialize_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::Your(card_predicate) => format!("an {}", serialize_your_predicate(card_predicate)),
        // ...
    }
}
```

**Key string categories identified:**

| Category | Examples | Localization Challenge |
|----------|----------|----------------------|
| Predicates | "a character", "an enemy", "this character" | Gender, case, articles |
| Quantities | "Draw {cards}", "Discard {discards}" | Number agreement, cardinal forms |
| Keywords | "{Dissolve}", "{Banish}", "{Kindle}" | Verb conjugation in context |
| Subtypes | "warrior", "ancient", "explorer" | Gender, number inflection |
| Triggers | "when you play", "at end of turn" | Verb tense, word order |
| Effects | "gain +{s} spark for each allied {subtype}" | Complex agreement chains |
| Conditions | "with {count} or more cards" | Comparatives, number agreement |

## Fluent-Based Architecture

### Core Design Principles

1. **Separate Files Per Language**: Each language has its own `.ftl` files with the SAME message IDs
2. **Rich Term Attributes**: Encode grammatical metadata (gender, animacy, class) as term attributes within each language
3. **Parameterized Terms**: Pass grammatical context (case, number, definiteness) to terms
4. **Selector Cascades**: Use nested selectors for complex agreement patterns
5. **Compositional Messages**: Build sentences from reusable, context-aware components

### Idiomatic Fluent File Structure

Each language has separate `.ftl` files. The Fluent runtime loads the appropriate file based on locale. All files use the **same message IDs**.

---

## Morphosyntactic Categories

### 1. Grammatical Gender

**Challenge**: Nouns have inherent gender affecting articles, adjectives, and past participles.

**Languages affected**: French, Spanish, Italian, German, Portuguese

#### Example: "ally" across languages

**locales/en/predicates.ftl:**
```fluent
-ally = ally
    .plural = allies

predicate-ally = an ally
predicate-allies = { $count } allies
```

**locales/fr/predicates.ftl:**
```fluent
# French - allié (m) / alliée (f) depending on referent
-ally = { $gender ->
    [masculine] allié
    [feminine] alliée
   *[other] allié
}
    .plural = { $gender ->
        [masculine] alliés
        [feminine] alliées
       *[other] alliés
    }
    .gender = variable

# Article must agree with gender
predicate-ally = { $gender ->
    [masculine] un allié
    [feminine] une alliée
   *[other] un allié
}
```

**locales/de/predicates.ftl:**
```fluent
# German - der Verbündete (m), die Verbündete (f), with case declension
-ally = { $case ->
    [nominative] { $gender ->
        [masculine] Verbündeter
        [feminine] Verbündete
       *[neuter] Verbündetes
    }
    [accusative] { $gender ->
        [masculine] Verbündeten
        [feminine] Verbündete
       *[neuter] Verbündetes
    }
    [dative] { $gender ->
        [masculine] Verbündetem
        [feminine] Verbündeter
       *[neuter] Verbündetem
    }
    [genitive] { $gender ->
        [masculine] Verbündeten
        [feminine] Verbündeter
       *[neuter] Verbündeten
    }
   *[other] Verbündete
}

# Article declension for indefinite
-article-indefinite = { $case ->
    [nominative] { $gender ->
        [masculine] ein
        [feminine] eine
       *[neuter] ein
    }
    [accusative] { $gender ->
        [masculine] einen
        [feminine] eine
       *[neuter] ein
    }
    [dative] { $gender ->
        [masculine] einem
        [feminine] einer
       *[neuter] einem
    }
   *[other] ein
}

predicate-ally = { -article-indefinite(case: $case, gender: "masculine") } { -ally(case: $case, gender: "masculine") }
```

**locales/es/predicates.ftl:**
```fluent
-ally = { $gender ->
    [masculine] aliado
    [feminine] aliada
   *[other] aliado
}
    .plural = { $gender ->
        [masculine] aliados
        [feminine] aliadas
       *[other] aliados
    }

predicate-ally = { $gender ->
    [masculine] un aliado
    [feminine] una aliada
   *[other] un aliado
}
```

**locales/it/predicates.ftl:**
```fluent
-ally = { $gender ->
    [masculine] alleato
    [feminine] alleata
   *[other] alleato
}
    .plural = { $gender ->
        [masculine] alleati
        [feminine] alleate
       *[other] alleati
    }

predicate-ally = { $gender ->
    [masculine] un alleato
    [feminine] una alleata
   *[other] un alleato
}
```

**locales/pt/predicates.ftl:**
```fluent
-ally = { $gender ->
    [masculine] aliado
    [feminine] aliada
   *[other] aliado
}
    .plural = { $gender ->
        [masculine] aliados
        [feminine] aliadas
       *[other] aliados
    }

predicate-ally = { $gender ->
    [masculine] um aliado
    [feminine] uma aliada
   *[other] um aliado
}
```

---

### 2. Grammatical Case

**Challenge**: German requires noun/article declension based on syntactic role.

**Languages affected**: German (4 cases), with remnants in some others

**locales/de/effects.ftl:**
```fluent
# German case system for "enemy character"
-enemy = { $case ->
    [nominative] feindlicher Charakter
    [accusative] feindlichen Charakter
    [dative] feindlichem Charakter
    [genitive] feindlichen Charakters
   *[other] feindlicher Charakter
}

# Usage in different grammatical contexts:

# Subject (nominative): "An enemy is dissolved"
effect-enemy-dissolved-passive =
    { -article-indefinite(case: "nominative", gender: "masculine") }
    { -enemy(case: "nominative") } wird aufgelöst.

# Direct object (accusative): "Dissolve an enemy"
effect-dissolve-enemy =
    Löse { -article-indefinite(case: "accusative", gender: "masculine") }
    { -enemy(case: "accusative") } auf.

# Indirect object (dative): "Give +1 spark to an enemy"
effect-give-spark-to-enemy =
    Gib { -article-indefinite(case: "dative", gender: "masculine") }
    { -enemy(case: "dative") } +{ $spark } Funken.
```

**locales/en/effects.ftl:**
```fluent
# English has no case system - simple forms
-enemy = enemy

effect-enemy-dissolved-passive = An enemy is dissolved.

effect-dissolve-enemy = Dissolve an enemy.

effect-give-spark-to-enemy = Give +{ $spark } spark to an enemy.
```

---

### 3. Plurality and Number Agreement

**Challenge**: Number affects verbs, articles, adjectives, and noun forms. Languages have different plural categories.

#### CLDR Plural Rules Integration

**locales/en/effects.ftl:**
```fluent
# English: singular (one) vs plural (other)
effect-draw-cards = { NUMBER($count) ->
    [one] Draw a card.
   *[other] Draw { $count } cards.
}

effect-discard-cards = { NUMBER($count) ->
    [one] Discard a card.
   *[other] Discard { $count } cards.
}
```

**locales/fr/effects.ftl:**
```fluent
# French requires article + noun + past participle agreement
effect-draw-cards = { NUMBER($count) ->
    [one] Piochez une carte.
   *[other] Piochez { $count } cartes.
}

# Past participle agreement in passive
cards-discarded = { NUMBER($count) ->
    [one] { $gender ->
        [feminine] une carte défaussée
       *[masculine] un carte défaussé
    }
   *[other] { $count } cartes défaussées
}
```

**locales/de/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Ziehe eine Karte.
   *[other] Ziehe { $count } Karten.
}

# With accusative case
effect-discard-cards = { NUMBER($count) ->
    [one] Wirf eine Karte ab.
   *[other] Wirf { $count } Karten ab.
}
```

**locales/ja/effects.ftl:**
```fluent
# Japanese uses counter words, not plural forms
effect-draw-cards = カードを{ $count }枚引く。

effect-discard-cards = カードを{ $count }枚捨てる。

# Counter classifier for cards (枚 is for flat objects)
-counter-cards = 枚
```

**locales/ko/effects.ftl:**
```fluent
effect-draw-cards = 카드를 { $count }장 뽑습니다.

effect-discard-cards = 카드를 { $count }장 버립니다.

# Native Korean number for small counts (optional)
-korean-counter = { NUMBER($count) ->
    [1] 한
    [2] 두
    [3] 세
    [4] 네
    [5] 다섯
   *[other] { $count }
}
```

**locales/zh/effects.ftl:**
```fluent
# Chinese uses classifiers (量词)
effect-draw-cards = 抽{ $count }张牌。

effect-discard-cards = 弃{ $count }张牌。
```

---

### 4. Articles and Definiteness

**Challenge**: Languages differ in article usage, definiteness marking, and when articles are required.

**locales/en/predicates.ftl:**
```fluent
# English: a/an based on phonology
-article-indef = { $next-sound ->
    [vowel] an
   *[consonant] a
}

predicate-event = { -article-indef(next-sound: "vowel") } event
predicate-character = { -article-indef(next-sound: "consonant") } character
```

**locales/fr/predicates.ftl:**
```fluent
# French: un/une based on gender
-article-indef = { $gender ->
    [masculine] un
    [feminine] une
   *[other] un
}

predicate-event = { -article-indef(gender: "masculine") } événement
predicate-card = { -article-indef(gender: "feminine") } carte

# Definite articles: le/la/l'/les
-article-def = { $number ->
    [plural] les
   *[singular] { $gender ->
        [masculine] { $elision ->
            [yes] l'
           *[no] le
        }
        [feminine] { $elision ->
            [yes] l'
           *[no] la
        }
       *[other] le
    }
}
```

**locales/de/predicates.ftl:**
```fluent
# German: der/die/das with case
-article-def = { $case ->
    [nominative] { $number ->
        [plural] die
       *[singular] { $gender ->
            [masculine] der
            [feminine] die
           *[neuter] das
        }
    }
    [accusative] { $number ->
        [plural] die
       *[singular] { $gender ->
            [masculine] den
            [feminine] die
           *[neuter] das
        }
    }
    [dative] { $number ->
        [plural] den
       *[singular] { $gender ->
            [masculine] dem
            [feminine] der
           *[neuter] dem
        }
    }
    [genitive] { $number ->
        [plural] der
       *[singular] { $gender ->
            [masculine] des
            [feminine] der
           *[neuter] des
        }
    }
   *[other] der
}
```

**locales/es/predicates.ftl:**
```fluent
# Spanish: el/la/los/las
-article-def = { $number ->
    [plural] { $gender ->
        [masculine] los
        [feminine] las
       *[other] los
    }
   *[singular] { $gender ->
        [masculine] el
        [feminine] la
       *[other] el
    }
}
```

**locales/ja/predicates.ftl:**
```fluent
# Japanese: no articles, context-dependent
predicate-character = キャラクター
```

**locales/ko/predicates.ftl:**
```fluent
# Korean: no articles
predicate-character = 캐릭터
```

**locales/zh/predicates.ftl:**
```fluent
# Chinese: no articles, classifier optional for definiteness
predicate-character = 角色
```

---

### 5. Adjective Agreement

**Challenge**: Adjectives must agree with nouns in gender, number, and sometimes case.

#### Example: "allied character"

**locales/en/predicates.ftl:**
```fluent
# English (no agreement)
-allied = allied

predicate-allied-character = an allied character
```

**locales/fr/predicates.ftl:**
```fluent
# French (gender + number agreement)
-allied = { $gender ->
    [masculine] { $number ->
        [plural] alliés
       *[singular] allié
    }
    [feminine] { $number ->
        [plural] alliées
       *[singular] alliée
    }
   *[other] allié
}

predicate-allied-character =
    { -article-indef(gender: "masculine") } personnage { -allied(gender: "masculine", number: "singular") }

predicate-allied-characters =
    des personnages { -allied(gender: "masculine", number: "plural") }
```

**locales/de/predicates.ftl:**
```fluent
# German (gender + number + case agreement)
-allied = { $case ->
    [nominative] { $gender ->
        [masculine] { $number ->
            [plural] verbündete
           *[singular] verbündeter
        }
        [feminine] { $number ->
            [plural] verbündete
           *[singular] verbündete
        }
       *[neuter] { $number ->
            [plural] verbündete
           *[singular] verbündetes
        }
    }
    [accusative] { $gender ->
        [masculine] { $number ->
            [plural] verbündete
           *[singular] verbündeten
        }
        [feminine] { $number ->
            [plural] verbündete
           *[singular] verbündete
        }
       *[neuter] { $number ->
            [plural] verbündete
           *[singular] verbündetes
        }
    }
   *[other] verbündete
}

predicate-allied-character =
    { -article-indefinite(case: $case, gender: "masculine") }
    { -allied(case: $case, gender: "masculine", number: "singular") }
    Charakter
```

**locales/es/predicates.ftl:**
```fluent
# Spanish (gender + number)
-allied = { $gender ->
    [masculine] { $number ->
        [plural] aliados
       *[singular] aliado
    }
    [feminine] { $number ->
        [plural] aliadas
       *[singular] aliada
    }
   *[other] aliado
}

predicate-allied-character = un personaje aliado
```

**locales/it/predicates.ftl:**
```fluent
# Italian (gender + number)
-allied = { $gender ->
    [masculine] { $number ->
        [plural] alleati
       *[singular] alleato
    }
    [feminine] { $number ->
        [plural] alleate
       *[singular] alleata
    }
   *[other] alleato
}
```

**locales/pt/predicates.ftl:**
```fluent
# Portuguese (gender + number)
-allied = { $gender ->
    [masculine] { $number ->
        [plural] aliados
       *[singular] aliado
    }
    [feminine] { $number ->
        [plural] aliadas
       *[singular] aliada
    }
   *[other] aliado
}
```

**locales/ja/predicates.ftl:**
```fluent
# Japanese (no agreement needed, adjective precedes noun)
predicate-allied-character = 味方キャラクター
```

**locales/ko/predicates.ftl:**
```fluent
# Korean (no agreement)
predicate-allied-character = 아군 캐릭터
```

**locales/zh/predicates.ftl:**
```fluent
# Chinese (no agreement)
predicate-allied-character = 友方角色
```

---

### 6. Adpositions (Prepositions/Postpositions)

**Challenge**: Languages use different strategies for spatial/relational concepts. Some require case changes.

#### Example: "from your void"

**locales/en/effects.ftl:**
```fluent
# English: preposition
location-from-void = from your void
```

**locales/fr/effects.ftl:**
```fluent
# French: preposition with article contraction
location-from-void = de votre néant
```

**locales/de/effects.ftl:**
```fluent
# German: preposition + dative case
location-from-void = aus deinem Nichts
```

**locales/es/effects.ftl:**
```fluent
# Spanish: preposition
location-from-void = de tu vacío
```

**locales/it/effects.ftl:**
```fluent
# Italian: preposition with article
location-from-void = dal tuo vuoto
```

**locales/pt/effects.ftl:**
```fluent
# Portuguese: preposition
location-from-void = do seu vazio
```

**locales/ja/effects.ftl:**
```fluent
# Japanese: postposition (particle)
location-from-void = あなたの虚無から
```

**locales/ko/effects.ftl:**
```fluent
# Korean: postposition (particle)
location-from-void = 당신의 공허에서
```

**locales/zh/effects.ftl:**
```fluent
# Chinese: preposition-like coverb
location-from-void = 从你的虚无
```

#### Locative Expressions

**locales/en/effects.ftl:**
```fluent
location-into-hand = into your hand
location-to-hand = to your hand
location-in-hand = in your hand
```

**locales/fr/effects.ftl:**
```fluent
# French (uses "dans" for all, or specific constructions)
location-into-hand = dans votre main
location-in-hand = dans votre main
```

**locales/de/effects.ftl:**
```fluent
# German (accusative for motion, dative for location)
location-into-hand = in deine Hand     # accusative - motion towards
location-in-hand = in deiner Hand      # dative - static location
```

**locales/ja/effects.ftl:**
```fluent
# Japanese (different particles)
location-into-hand = 手札に             # に indicates direction
location-in-hand = 手札で              # で indicates location
location-from-hand = 手札から          # から indicates source
```

---

### 7. Verb Agreement and Conjugation

**Challenge**: Verbs agree with subjects in person/number, and sometimes gender. Imperative vs. indicative forms differ.

#### Example: "you may" - 2nd person optative/permissive

**locales/en/effects.ftl:**
```fluent
phrase-you-may = you may
```

**locales/fr/effects.ftl:**
```fluent
# French (vous/tu formality distinction)
phrase-you-may = { $formality ->
    [formal] vous pouvez
   *[informal] tu peux
}
```

**locales/de/effects.ftl:**
```fluent
phrase-you-may = { $formality ->
    [formal] Sie dürfen
   *[informal] du darfst
}
```

**locales/es/effects.ftl:**
```fluent
phrase-you-may = { $formality ->
    [formal] usted puede
   *[informal] puedes
}
```

**locales/it/effects.ftl:**
```fluent
phrase-you-may = { $formality ->
    [formal] Lei può
   *[informal] puoi
}
```

**locales/pt/effects.ftl:**
```fluent
phrase-you-may = { $formality ->
    [formal] você pode
   *[informal] podes
}
```

**locales/ja/effects.ftl:**
```fluent
# Japanese (politeness levels)
phrase-you-may = { $politeness ->
    [polite] ～てもよいです
   *[plain] ～てもよい
}
```

**locales/ko/effects.ftl:**
```fluent
# Korean (politeness levels)
phrase-you-may = { $politeness ->
    [formal] ～해도 됩니다
   *[informal] ～해도 돼
}
```

#### Imperative Forms

**locales/en/effects.ftl:**
```fluent
# English (bare verb)
verb-draw-imperative = Draw
```

**locales/fr/effects.ftl:**
```fluent
# French (vous form imperative)
verb-draw-imperative = Piochez
```

**locales/de/effects.ftl:**
```fluent
# German (Sie form imperative)
verb-draw-imperative = Ziehen Sie
```

**locales/es/effects.ftl:**
```fluent
# Spanish (tú/usted)
verb-draw-imperative = { $formality ->
    [formal] Robe
   *[informal] Roba
}
```

**locales/it/effects.ftl:**
```fluent
verb-draw-imperative = Pesca
```

**locales/pt/effects.ftl:**
```fluent
verb-draw-imperative = { $formality ->
    [formal] Compre
   *[informal] Compra
}
```

**locales/ja/effects.ftl:**
```fluent
# Japanese (command form varies by politeness)
verb-draw-imperative = { $politeness ->
    [polite] 引いてください
   *[plain] 引け
}
```

**locales/ko/effects.ftl:**
```fluent
verb-draw-imperative = { $politeness ->
    [formal] 뽑으세요
   *[informal] 뽑아
}
```

---

### 8. Countability and Mass Nouns

**Challenge**: Some nouns are countable in one language but mass nouns in another.

#### Example: "energy"

**locales/en/effects.ftl:**
```fluent
# English: mass noun, no plural
effect-gain-energy = Gain { $amount }●.

# But with explicit unit
effect-gain-energy-verbose = Gain { $amount } energy { NUMBER($amount) ->
    [one] point
   *[other] points
}.
```

**locales/fr/effects.ftl:**
```fluent
# French: can use countable construction
effect-gain-energy = Gagnez { $amount } { NUMBER($amount) ->
    [one] point d'énergie
   *[other] points d'énergie
}.
```

**locales/de/effects.ftl:**
```fluent
# German: Energie (feminine, usually mass)
effect-gain-energy = Erhalte { $amount } Energie.
```

**locales/ja/effects.ftl:**
```fluent
# Japanese: counter classifier for abstract quantities
effect-gain-energy = { $amount }エネルギーを得る。
```

---

### 9. Comparatives and Superlatives

**Challenge**: "X or more", "X or less", "greater than", etc. require different syntactic structures.

#### Example: "with cost 3 or less"

**locales/en/predicates.ftl:**
```fluent
predicate-cost-or-less = with cost { $cost }● or less
predicate-cost-or-more = with cost { $cost }● or more
predicate-cost-less-than = with cost less than { $cost }●
predicate-cost-greater-than = with cost greater than { $cost }●
```

**locales/fr/predicates.ftl:**
```fluent
predicate-cost-or-less = avec un coût de { $cost }● ou moins
predicate-cost-or-more = avec un coût de { $cost }● ou plus
predicate-cost-less-than = avec un coût inférieur à { $cost }●
```

**locales/de/predicates.ftl:**
```fluent
# German (uses genitive or specific constructions)
predicate-cost-or-less = mit Kosten von { $cost }● oder weniger
predicate-cost-or-more = mit Kosten von { $cost }● oder mehr
```

**locales/es/predicates.ftl:**
```fluent
predicate-cost-or-less = con coste de { $cost }● o menos
predicate-cost-or-more = con coste de { $cost }● o más
```

**locales/it/predicates.ftl:**
```fluent
predicate-cost-or-less = con costo { $cost }● o meno
predicate-cost-or-more = con costo { $cost }● o più
```

**locales/pt/predicates.ftl:**
```fluent
predicate-cost-or-less = com custo { $cost }● ou menos
predicate-cost-or-more = com custo { $cost }● ou mais
```

**locales/ja/predicates.ftl:**
```fluent
predicate-cost-or-less = コスト{ $cost }●以下の
predicate-cost-or-more = コスト{ $cost }●以上の
predicate-cost-less-than = コスト{ $cost }●未満の
```

**locales/ko/predicates.ftl:**
```fluent
predicate-cost-or-less = 비용 { $cost }● 이하의
predicate-cost-or-more = 비용 { $cost }● 이상의
```

**locales/zh/predicates.ftl:**
```fluent
predicate-cost-or-less = 费用为{ $cost }●或更少的
predicate-cost-or-more = 费用为{ $cost }●或更多的
```

---

### 10. Coordination (And/Or)

**Challenge**: Conjunctions may require different forms, and lists have language-specific punctuation.

#### Example: "draw a card and gain 1⍟"

**locales/en/effects.ftl:**
```fluent
conjunction-and = and
conjunction-or = or

effect-combined-and = { $effect1 } and { $effect2 }
effect-combined-or = { $effect1 } or { $effect2 }
```

**locales/fr/effects.ftl:**
```fluent
conjunction-and = et
conjunction-or = ou

effect-combined-and = { $effect1 } et { $effect2 }
effect-combined-or = { $effect1 } ou { $effect2 }
```

**locales/de/effects.ftl:**
```fluent
conjunction-and = und
conjunction-or = oder

effect-combined-and = { $effect1 } und { $effect2 }
effect-combined-or = { $effect1 } oder { $effect2 }
```

**locales/es/effects.ftl:**
```fluent
conjunction-and = y
conjunction-or = o

effect-combined-and = { $effect1 } y { $effect2 }
effect-combined-or = { $effect1 } o { $effect2 }

# Note: "y" becomes "e" before words starting with "i-" or "hi-"
# "o" becomes "u" before words starting with "o-" or "ho-"
# This may need runtime handling
```

**locales/it/effects.ftl:**
```fluent
conjunction-and = e
conjunction-or = o

effect-combined-and = { $effect1 } e { $effect2 }
effect-combined-or = { $effect1 } o { $effect2 }

# Note: "e" becomes "ed" before vowels (euphonic d)
# This may need runtime handling
```

**locales/pt/effects.ftl:**
```fluent
conjunction-and = e
conjunction-or = ou

effect-combined-and = { $effect1 } e { $effect2 }
effect-combined-or = { $effect1 } ou { $effect2 }
```

**locales/ja/effects.ftl:**
```fluent
# Japanese (multiple conjunction types)
effect-combined-and = { $effect1 }、そして{ $effect2 }
effect-combined-then = { $effect1 }、次に{ $effect2 }
```

**locales/ko/effects.ftl:**
```fluent
effect-combined-and = { $effect1 }, 그리고 { $effect2 }
```

**locales/zh/effects.ftl:**
```fluent
effect-combined-and = { $effect1 }，然后{ $effect2 }
```

#### List Serialization

**locales/en/effects.ftl:**
```fluent
# Three-item list: "A, B, and C" (Oxford comma)
list-three = { $item1 }, { $item2 }, and { $item3 }
```

**locales/fr/effects.ftl:**
```fluent
# French (no Oxford comma)
list-three = { $item1 }, { $item2 } et { $item3 }
```

**locales/de/effects.ftl:**
```fluent
list-three = { $item1 }, { $item2 } und { $item3 }
```

**locales/ja/effects.ftl:**
```fluent
# Japanese (different punctuation)
list-three = { $item1 }、{ $item2 }、{ $item3 }
```

---

## Complete Ability Examples

These examples show how the **same message ID** is used across all language files, with appropriate translations.

### Example 1: Simple Draw Effect

**English**: "Draw a card."

**locales/en/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Draw a card.
   *[other] Draw { $count } cards.
}
```

**locales/fr/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Piochez une carte.
   *[other] Piochez { $count } cartes.
}
```

**locales/de/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Ziehe eine Karte.
   *[other] Ziehe { $count } Karten.
}
```

**locales/es/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Roba una carta.
   *[other] Roba { $count } cartas.
}
```

**locales/it/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Pesca una carta.
   *[other] Pesca { $count } carte.
}
```

**locales/pt/effects.ftl:**
```fluent
effect-draw-cards = { NUMBER($count) ->
    [one] Compre uma carta.
   *[other] Compre { $count } cartas.
}
```

**locales/ja/effects.ftl:**
```fluent
effect-draw-cards = カードを{ $count }枚引く。
```

**locales/ko/effects.ftl:**
```fluent
effect-draw-cards = 카드를 { $count }장 뽑습니다.
```

**locales/zh/effects.ftl:**
```fluent
effect-draw-cards = 抽{ $count }张牌。
```

---

### Example 2: Dissolve with Target

**English**: "Dissolve an enemy." / "Dissolve an enemy with spark 1 or less."

**locales/en/keywords.ftl:**
```fluent
-keyword-dissolve = Dissolve
    .verb = dissolve

-enemy = enemy
    .plural = enemies

effect-dissolve-enemy = { -keyword-dissolve } an enemy.

effect-dissolve-enemy-spark-limit = { -keyword-dissolve } an enemy with spark { $spark } or less.
```

**locales/fr/keywords.ftl:**
```fluent
-keyword-dissolve = Dissoudre
    .verb = dissoudre
    .past-participle-m-s = dissous
    .past-participle-f-s = dissoute
    .past-participle-m-p = dissous
    .past-participle-f-p = dissoutes

-enemy = ennemi
    .plural = ennemis
    .gender = masculine

effect-dissolve-enemy = Dissolvez un ennemi.

effect-dissolve-enemy-spark-limit = Dissolvez un ennemi avec { $spark } d'étincelle ou moins.
```

**locales/de/keywords.ftl:**
```fluent
-keyword-dissolve = Auflösen
    .verb = auflösen

-enemy = Feind
    .plural = Feinde
    .gender = masculine

effect-dissolve-enemy = Löse einen Feind auf.

effect-dissolve-enemy-spark-limit = Löse einen Feind mit { $spark } Funken oder weniger auf.
```

**locales/es/keywords.ftl:**
```fluent
-keyword-dissolve = Disolver
    .verb = disolver

-enemy = enemigo
    .plural = enemigos
    .gender = masculine

effect-dissolve-enemy = Disuelve un enemigo.

effect-dissolve-enemy-spark-limit = Disuelve un enemigo con chispa { $spark } o menos.
```

**locales/it/keywords.ftl:**
```fluent
-keyword-dissolve = Dissolvere
    .verb = dissolvere

-enemy = nemico
    .plural = nemici
    .gender = masculine

effect-dissolve-enemy = Dissolvi un nemico.

effect-dissolve-enemy-spark-limit = Dissolvi un nemico con scintilla { $spark } o meno.
```

**locales/pt/keywords.ftl:**
```fluent
-keyword-dissolve = Dissolver
    .verb = dissolver

-enemy = inimigo
    .plural = inimigos
    .gender = masculine

effect-dissolve-enemy = Dissolva um inimigo.

effect-dissolve-enemy-spark-limit = Dissolva um inimigo com faísca { $spark } ou menos.
```

**locales/ja/keywords.ftl:**
```fluent
-keyword-dissolve = 溶解
    .verb = 溶解する

effect-dissolve-enemy = 敵1体を溶解する。

effect-dissolve-enemy-spark-limit = 閃き{ $spark }以下の敵1体を溶解する。
```

**locales/ko/keywords.ftl:**
```fluent
-keyword-dissolve = 용해
    .verb = 용해하다

effect-dissolve-enemy = 적 1명을 용해합니다.

effect-dissolve-enemy-spark-limit = 점화 { $spark } 이하인 적 1명을 용해합니다.
```

**locales/zh/keywords.ftl:**
```fluent
-keyword-dissolve = 溶解
    .verb = 溶解

effect-dissolve-enemy = 溶解1名敌人。

effect-dissolve-enemy-spark-limit = 溶解1名闪光{ $spark }或更少的敌人。
```

---

### Example 3: Complex Triggered Ability

**English**: "When you materialize an allied warrior, this character gains +1 spark."

**locales/en/triggers.ftl:**
```fluent
-subtype-warrior = warrior
    .plural = warriors

trigger-materialize-allied-subtype = When you materialize an allied { $subtype },

effect-this-gains-spark = this character gains +{ $spark } spark.
```

**locales/fr/triggers.ftl:**
```fluent
-subtype-warrior = guerrier
    .plural = guerriers
    .gender = masculine

trigger-materialize-allied-subtype = Quand vous matérialisez un { $subtype } allié,

effect-this-gains-spark = ce personnage gagne +{ $spark } d'étincelle.
```

**locales/de/triggers.ftl:**
```fluent
-subtype-warrior = Krieger
    .plural = Krieger
    .gender = masculine

trigger-materialize-allied-subtype = Wenn du einen verbündeten { $subtype } materialisierst,

effect-this-gains-spark = erhält dieser Charakter +{ $spark } Funken.
```

**locales/es/triggers.ftl:**
```fluent
-subtype-warrior = guerrero
    .plural = guerreros
    .gender = masculine

trigger-materialize-allied-subtype = Cuando materializas un { $subtype } aliado,

effect-this-gains-spark = este personaje gana +{ $spark } de chispa.
```

**locales/it/triggers.ftl:**
```fluent
-subtype-warrior = guerriero
    .plural = guerrieri
    .gender = masculine

trigger-materialize-allied-subtype = Quando materializzi un { $subtype } alleato,

effect-this-gains-spark = questo personaggio ottiene +{ $spark } scintilla.
```

**locales/pt/triggers.ftl:**
```fluent
-subtype-warrior = guerreiro
    .plural = guerreiros
    .gender = masculine

trigger-materialize-allied-subtype = Quando você materializa um { $subtype } aliado,

effect-this-gains-spark = esta personagem ganha +{ $spark } de faísca.
```

**locales/ja/triggers.ftl:**
```fluent
-subtype-warrior = 戦士

trigger-materialize-allied-subtype = あなたが味方の{ $subtype }を実体化したとき、

effect-this-gains-spark = このキャラクターは+{ $spark }閃きを得る。
```

**locales/ko/triggers.ftl:**
```fluent
-subtype-warrior = 전사

trigger-materialize-allied-subtype = 당신이 아군 { $subtype }를 구현할 때,

effect-this-gains-spark = 이 캐릭터는 +{ $spark } 점화를 얻습니다.
```

**locales/zh/triggers.ftl:**
```fluent
-subtype-warrior = 战士

trigger-materialize-allied-subtype = 当你实体化一名友方{ $subtype }时，

effect-this-gains-spark = 此角色获得+{ $spark }闪光。
```

---

### Example 4: For-Each Quantified Effect

**English**: "Gain +1 spark for each allied warrior."

**locales/en/effects.ftl:**
```fluent
effect-gain-spark-for-each = Gain +{ $spark } spark for each { $target }.
```

**locales/fr/effects.ftl:**
```fluent
effect-gain-spark-for-each = Gagnez +{ $spark } d'étincelle pour chaque { $target }.
```

**locales/de/effects.ftl:**
```fluent
effect-gain-spark-for-each = Erhalte +{ $spark } Funken für jeden { $target }.
```

**locales/es/effects.ftl:**
```fluent
effect-gain-spark-for-each = Gana +{ $spark } de chispa por cada { $target }.
```

**locales/it/effects.ftl:**
```fluent
effect-gain-spark-for-each = Ottieni +{ $spark } scintilla per ogni { $target }.
```

**locales/pt/effects.ftl:**
```fluent
effect-gain-spark-for-each = Ganhe +{ $spark } de faísca para cada { $target }.
```

**locales/ja/effects.ftl:**
```fluent
effect-gain-spark-for-each = { $target }1体につき+{ $spark }閃きを得る。
```

**locales/ko/effects.ftl:**
```fluent
effect-gain-spark-for-each = { $target } 1명당 +{ $spark } 점화를 얻습니다.
```

**locales/zh/effects.ftl:**
```fluent
effect-gain-spark-for-each = 每有一名{ $target }，获得+{ $spark }闪光。
```

---

### Example 5: Optional Effect with Cost

**English**: "You may discard a card to draw a card and gain 1⍟."

**locales/en/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    You may { NUMBER($discard-count) ->
        [one] discard a card
       *[other] discard { $discard-count } cards
    } to { NUMBER($draw-count) ->
        [one] draw a card
       *[other] draw { $draw-count } cards
    } and gain { $points }⍟.
```

**locales/fr/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    Vous pouvez défausser { NUMBER($discard-count) ->
        [one] une carte
       *[other] { $discard-count } cartes
    } pour piocher { NUMBER($draw-count) ->
        [one] une carte
       *[other] { $draw-count } cartes
    } et gagner { $points }⍟.
```

**locales/de/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    Du darfst { NUMBER($discard-count) ->
        [one] eine Karte
       *[other] { $discard-count } Karten
    } abwerfen, um { NUMBER($draw-count) ->
        [one] eine Karte
       *[other] { $draw-count } Karten
    } zu ziehen und { $points }⍟ zu erhalten.
```

**locales/es/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    Puedes descartar { NUMBER($discard-count) ->
        [one] una carta
       *[other] { $discard-count } cartas
    } para robar { NUMBER($draw-count) ->
        [one] una carta
       *[other] { $draw-count } cartas
    } y ganar { $points }⍟.
```

**locales/it/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    Puoi scartare { NUMBER($discard-count) ->
        [one] una carta
       *[other] { $discard-count } carte
    } per pescare { NUMBER($draw-count) ->
        [one] una carta
       *[other] { $draw-count } carte
    } e guadagnare { $points }⍟.
```

**locales/pt/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    Você pode descartar { NUMBER($discard-count) ->
        [one] uma carta
       *[other] { $discard-count } cartas
    } para comprar { NUMBER($draw-count) ->
        [one] uma carta
       *[other] { $draw-count } cartas
    } e ganhar { $points }⍟.
```

**locales/ja/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    カードを{ $discard-count }枚捨ててもよい。そうしたなら、カードを{ $draw-count }枚引き、{ $points }⍟を得る。
```

**locales/ko/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    카드 { $discard-count }장을 버릴 수 있습니다. 그렇게 하면, 카드 { $draw-count }장을 뽑고 { $points }⍟를 얻습니다.
```

**locales/zh/effects.ftl:**
```fluent
effect-optional-discard-to-draw-and-gain =
    你可以弃{ $discard-count }张牌来抽{ $draw-count }张牌并获得{ $points }⍟。
```

---

## Implementation Architecture

### Rust Integration Strategy

#### 1. Define Grammatical Context Struct

```rust
/// Grammatical context passed to Fluent for proper agreement
#[derive(Clone, Debug, Default)]
pub struct GrammaticalContext {
    /// Grammatical gender of the subject/object
    pub gender: Option<Gender>,
    /// Number (singular/plural)
    pub number: Option<Number>,
    /// Grammatical case (for German, etc.)
    pub case: Option<Case>,
    /// Whether this is definite or indefinite
    pub definiteness: Option<Definiteness>,
    /// Politeness/formality level
    pub formality: Option<Formality>,
}

#[derive(Clone, Copy, Debug)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    Common,
}

#[derive(Clone, Copy, Debug)]
pub enum Number {
    Singular,
    Plural,
}

#[derive(Clone, Copy, Debug)]
pub enum Case {
    Nominative,
    Accusative,
    Dative,
    Genitive,
}

#[derive(Clone, Copy, Debug)]
pub enum Definiteness {
    Definite,
    Indefinite,
}

#[derive(Clone, Copy, Debug)]
pub enum Formality {
    Formal,
    Informal,
}
```

#### 2. Create Localized String Builder

```rust
use fluent::{FluentBundle, FluentResource, FluentArgs, FluentValue};

pub struct LocalizedStringBuilder<'a> {
    bundle: &'a FluentBundle<FluentResource>,
    args: FluentArgs<'a>,
    context: GrammaticalContext,
}

impl<'a> LocalizedStringBuilder<'a> {
    pub fn new(bundle: &'a FluentBundle<FluentResource>) -> Self {
        Self {
            bundle,
            args: FluentArgs::new(),
            context: GrammaticalContext::default(),
        }
    }

    pub fn with_count(mut self, name: &'a str, count: i64) -> Self {
        self.args.set(name, FluentValue::from(count));
        self
    }

    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.context.gender = Some(gender);
        self.args.set("gender", FluentValue::from(gender.to_fluent_string()));
        self
    }

    pub fn with_case(mut self, case: Case) -> Self {
        self.context.case = Some(case);
        self.args.set("case", FluentValue::from(case.to_fluent_string()));
        self
    }

    pub fn format(self, message_id: &str) -> String {
        let msg = self.bundle.get_message(message_id)
            .expect("Message not found");
        let pattern = msg.value()
            .expect("Message has no value");
        let mut errors = vec![];
        self.bundle.format_pattern(pattern, Some(&self.args), &mut errors)
            .to_string()
    }
}
```

#### 3. Modify Serializer Functions

```rust
/// Serialize an effect with localization support
pub fn serialize_effect_localized(
    effect: &StandardEffect,
    bundle: &FluentBundle<FluentResource>,
) -> String {
    match effect {
        StandardEffect::DrawCards { count } => {
            LocalizedStringBuilder::new(bundle)
                .with_count("count", *count as i64)
                .format("effect-draw-cards")
        }
        StandardEffect::DissolveCharacter { target } => {
            let (target_msg, target_context) = serialize_predicate_localized(target, bundle);
            LocalizedStringBuilder::new(bundle)
                .with_gender(target_context.gender.unwrap_or(Gender::Common))
                .format("effect-dissolve")
                + " "
                + &target_msg
                + "."
        }
        // ...
    }
}

/// Serialize a predicate, returning both the string and grammatical metadata
pub fn serialize_predicate_localized(
    predicate: &Predicate,
    bundle: &FluentBundle<FluentResource>,
) -> (String, GrammaticalContext) {
    match predicate {
        Predicate::This => {
            let s = LocalizedStringBuilder::new(bundle).format("predicate-this");
            // "this character" inherits character's gender
            let ctx = GrammaticalContext {
                gender: Some(Gender::Common), // or lookup from term
                number: Some(Number::Singular),
                ..Default::default()
            };
            (s, ctx)
        }
        Predicate::Enemy(card_predicate) => {
            // Get base predicate info
            let (base, ctx) = serialize_card_predicate_localized(card_predicate, bundle);
            let s = LocalizedStringBuilder::new(bundle)
                .with_gender(ctx.gender.unwrap_or(Gender::Common))
                .format("predicate-enemy")
                .replace("{target}", &base);
            (s, ctx)
        }
        // ...
    }
}
```

#### 4. Term Metadata Registry

```rust
/// Registry for looking up grammatical properties of game terms
pub struct TermRegistry {
    terms: HashMap<String, TermMetadata>,
}

pub struct TermMetadata {
    pub gender: HashMap<Locale, Gender>,
    pub noun_class: Option<NounClass>,
    pub countable: bool,
}

impl TermRegistry {
    pub fn get_gender(&self, term: &str, locale: &Locale) -> Option<Gender> {
        self.terms.get(term)?.gender.get(locale).copied()
    }
}

// Initialize from term definitions in Fluent files
// using term attributes like .gender = masculine
```

---

## File Organization

```
src/
├── localization/
│   ├── mod.rs
│   ├── builder.rs          # LocalizedStringBuilder
│   ├── context.rs          # GrammaticalContext types
│   ├── registry.rs         # TermRegistry
│   └── bundles.rs          # Bundle loading/management
│
├── parser/src/serializer/
│   ├── mod.rs
│   ├── ability_serializer.rs    # Updated with _localized variants
│   ├── effect_serializer.rs
│   ├── predicate_serializer.rs
│   └── ...
│
locales/
├── en/
│   ├── main.ftl           # Core messages
│   ├── keywords.ftl       # Game keyword terms
│   ├── effects.ftl        # Effect templates
│   ├── triggers.ftl       # Trigger templates
│   └── predicates.ftl     # Predicate templates
├── fr/
│   ├── main.ftl           # Same message IDs, French translations
│   ├── keywords.ftl
│   ├── effects.ftl
│   ├── triggers.ftl
│   └── predicates.ftl
├── de/
│   └── ... (same structure, German translations)
├── es/
│   └── ... (same structure, Spanish translations)
├── it/
│   └── ... (same structure, Italian translations)
├── pt/
│   └── ... (same structure, Portuguese translations)
├── ja/
│   └── ... (same structure, Japanese translations)
├── ko/
│   └── ... (same structure, Korean translations)
└── zh/
    └── ... (same structure, Chinese translations)
```

---

## Testing Strategy

### 1. Grammatical Correctness Tests

```rust
#[test]
fn test_french_article_agreement() {
    let bundle = load_bundle("fr");

    // Feminine noun (carte)
    let result = LocalizedStringBuilder::new(&bundle)
        .with_count("count", 1)
        .format("effect-draw-cards");
    assert_eq!(result, "Piochez une carte.");

    // Masculine noun (personnage)
    let result = LocalizedStringBuilder::new(&bundle)
        .format("predicate-character");
    assert_eq!(result, "un personnage");
}

#[test]
fn test_german_case_declension() {
    let bundle = load_bundle("de");

    // Accusative case for direct object
    let result = LocalizedStringBuilder::new(&bundle)
        .with_case(Case::Accusative)
        .format("effect-dissolve-enemy");
    assert!(result.contains("einen Feind"));

    // Dative case for indirect object
    let result = LocalizedStringBuilder::new(&bundle)
        .with_case(Case::Dative)
        .format("effect-give-to-enemy");
    assert!(result.contains("einem Feind"));
}
```

### 2. Plural Category Tests

```rust
#[test]
fn test_plural_categories() {
    let bundle = load_bundle("en");

    // Singular
    assert_eq!(
        format_draw_cards(&bundle, 1),
        "Draw a card."
    );

    // Plural
    assert_eq!(
        format_draw_cards(&bundle, 3),
        "Draw 3 cards."
    );
}

#[test]
fn test_japanese_counters() {
    let bundle = load_bundle("ja");

    // Should use 枚 counter for cards
    let result = format_draw_cards(&bundle, 2);
    assert!(result.contains("2枚"));
}
```

### 3. Message ID Consistency Tests

```rust
#[test]
fn test_all_languages_have_same_message_ids() {
    let languages = vec!["en", "fr", "de", "es", "it", "pt", "ja", "ko", "zh"];

    let en_bundle = load_bundle("en");
    let en_message_ids: HashSet<_> = en_bundle
        .get_message_ids()
        .collect();

    for lang in languages.iter().skip(1) {
        let bundle = load_bundle(lang);
        let message_ids: HashSet<_> = bundle
            .get_message_ids()
            .collect();

        // All languages should have the same message IDs
        assert_eq!(
            en_message_ids, message_ids,
            "Language {} has different message IDs than English", lang
        );
    }
}
```

### 4. Round-Trip Tests

```rust
#[test]
fn test_localized_matches_english_semantics() {
    let en_bundle = load_bundle("en");
    let fr_bundle = load_bundle("fr");

    // Same ability should produce grammatically valid output in both
    let ability = Ability::Triggered(TriggeredAbility {
        trigger: TriggerEvent::Materialize(Predicate::Any(CardPredicate::Character)),
        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
        // ...
    });

    let en_text = serialize_ability_localized(&ability, &en_bundle);
    let fr_text = serialize_ability_localized(&ability, &fr_bundle);

    // Both should be non-empty and well-formed
    assert!(!en_text.is_empty());
    assert!(!fr_text.is_empty());

    // French should have proper accents
    assert!(fr_text.chars().any(|c| "éèêë".contains(c)));
}
```

---

## Migration Path

### Phase 1: Foundation (Weeks 1-2)
1. Create `localization/` module structure
2. Define `GrammaticalContext` and related types
3. Set up Fluent bundle loading infrastructure
4. Create English `.ftl` files as baseline with all message IDs

### Phase 2: Core Serializers (Weeks 3-4)
1. Add `_localized` variants to all serializer functions
2. Implement `LocalizedStringBuilder`
3. Create `TermRegistry` for gender/class lookups
4. Write comprehensive tests for English

### Phase 3: First Target Language (Weeks 5-6)
1. Choose French as first non-English language (rich morphology)
2. Create French `.ftl` files with same message IDs, full term attributes
3. Validate all grammatical agreement patterns
4. Fix edge cases discovered during testing

### Phase 4: Remaining Languages (Weeks 7-10)
1. German (case system validation)
2. Spanish/Italian/Portuguese (Romance language patterns)
3. Japanese/Korean/Chinese (counter classifiers, no articles)
4. Each language uses the same message IDs

### Phase 5: Integration (Weeks 11-12)
1. Wire localized serializers into main codebase
2. Add locale selection to client
3. Performance optimization
4. Final validation across all supported languages
5. Add CI tests to ensure all languages have matching message IDs

---

## Open Questions

1. **Formality level**: Should we use formal (vous/Sie/usted) or informal (tu/du/tú) address? Most card games use informal.

2. **Locale variants**: Do we need separate locales for regional variants (e.g., pt-BR vs pt-PT, zh-CN vs zh-TW)?

3. **Dynamic subtypes**: How do we handle user-created card subtypes? Pre-translate all possible subtypes or use a fallback?

4. **Symbol rendering**: Should ⍟ and ● be localized or kept as universal symbols?

5. **Text length**: Some languages (German, Japanese) produce significantly longer text. How does this affect card layout?

---

## Appendix: CLDR Plural Rules Reference

| Language | Plural Categories |
|----------|-------------------|
| English | one, other |
| French | one, many, other |
| German | one, other |
| Spanish | one, many, other |
| Italian | one, many, other |
| Portuguese | one, many, other |
| Japanese | other (no grammatical plural) |
| Korean | other (no grammatical plural) |
| Chinese | other (no grammatical plural) |

---

## Appendix: Game Term Gender Reference

| Term (EN) | French | German | Spanish | Italian | Portuguese |
|-----------|--------|--------|---------|---------|------------|
| character | m | m | m | m | f |
| event | m | n | m | m | m |
| card | f | f | f | f | f |
| enemy | m | m | m | m | m |
| ally | m | m | m | m | m |
| spark | f | m | f | f | f |
| energy | f | f | f | f | f |
| void | m | n | m | m | m |
| hand | f | f | f | f | f |
| deck | m | n | m | m | m |
