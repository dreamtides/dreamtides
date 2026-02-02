# Appendix B: Russian Localization Guide

## Overview

Russian has complex grammar that significantly affects localization:
- **6 grammatical cases** that change noun/adjective endings
- **3 genders** (masculine, feminine, neuter)
- **Animacy** affects accusative case for masculine nouns
- **Complex plural rules** with 3 categories based on number

## Grammatical Cases

### The Six Cases

| Case | Usage | Question |
|------|-------|----------|
| Nominative (им.) | Subject | Кто? Что? |
| Genitive (род.) | Possession, "of", after quantities 5+ | Кого? Чего? |
| Dative (дат.) | Indirect object, "to" | Кому? Чему? |
| Accusative (вин.) | Direct object | Кого? Что? |
| Instrumental (тв.) | "with", means | Кем? Чем? |
| Prepositional (пр.) | Location, "about" | О ком? О чём? |

### Defining Nouns with Declension

```rust
phraselet! {
    // Feminine noun: карта (card)
    noun карта(fem) = {
        nom: "карта" / "карты",
        gen: "карты" / "карт",
        dat: "карте" / "картам",
        acc: "карту" / "карты",
        ins: "картой" / "картами",
        pre: "карте" / "картах",
    }

    // Masculine inanimate: персонаж (character)
    noun персонаж(masc) = {
        nom: "персонаж" / "персонажи",
        gen: "персонажа" / "персонажей",
        dat: "персонажу" / "персонажам",
        acc: "персонаж" / "персонажи",    // inanimate = nominative
        ins: "персонажем" / "персонажами",
        pre: "персонаже" / "персонажах",
    }

    // Masculine animate: союзник (ally)
    noun союзник(masc.animate) = {
        nom: "союзник" / "союзники",
        gen: "союзника" / "союзников",
        dat: "союзнику" / "союзникам",
        acc: "союзника" / "союзников",    // animate = genitive!
        ins: "союзником" / "союзниками",
        pre: "союзнике" / "союзниках",
    }

    // Neuter noun: событие (event)
    noun событие(neut) = {
        nom: "событие" / "события",
        gen: "события" / "событий",
        dat: "событию" / "событиям",
        acc: "событие" / "события",       // neuter = nominative
        ins: "событием" / "событиями",
        pre: "событии" / "событиях",
    }

    // Feminine noun: искра (spark)
    noun искра(fem) = {
        nom: "искра" / "искры",
        gen: "искры" / "искр",
        dat: "искре" / "искрам",
        acc: "искру" / "искры",
        ins: "искрой" / "искрами",
        pre: "искре" / "искрах",
    }
}
```

### Using Cases in Messages

```rust
phraselet! {
    // Direct object takes accusative
    возьмите_карту = "Возьмите {карта.acc.sg}."  // "Возьмите карту."

    // "With" takes instrumental
    с_картой = "с {карта.ins.sg}"               // "с картой"

    // "In" takes prepositional
    в_колоде = "в {колода.pre.sg}"              // "в колоде"

    // "From" takes genitive
    из_руки = "из {рука.gen.sg}"                // "из руки"
}
```

---

## Plural Categories

### Russian Plural Rules

Russian has three plural categories based on the last digits:

| Category | Numbers | Example |
|----------|---------|---------|
| one | 1, 21, 31, 41... (except 11) | 1 карта |
| few | 2-4, 22-24, 32-34... (except 12-14) | 2 карты |
| many | 0, 5-20, 25-30, 35-40... and 11-14 | 5 карт |

**Special case:** 11-14 always use "many" form, not "one" or "few".

### Implementation

```rust
phraselet! {
    // Cards with numeric agreement
    взять_карты(count: Int) = match count {
        one  => "Возьмите {count} карту."      // 1, 21, 31...
        few  => "Возьмите {count} карты."      // 2, 3, 4, 22...
        many => "Возьмите {count} карт."       // 0, 5-20, 25...
    }

    // The case changes too! After numbers:
    // 1 → nominative singular
    // 2-4 → genitive singular
    // 5+ → genitive plural

    карты_с_числом(count: Int) = match count {
        one  => "{count} {карта.nom.sg}"       // "1 карта"
        few  => "{count} {карта.gen.sg}"       // "2 карты"
        many => "{count} {карта.gen.pl}"       // "5 карт"
    }
}
```

### Numbers 11-14 Exception

```rust
phraselet! {
    // These all use "many" form despite ending in 1, 2, 3, 4:
    // 11 карт (not "11 карта")
    // 12 карт (not "12 карты")
    // 13 карт
    // 14 карт

    // Phraselet handles this automatically with CLDR rules
    test_numbers(n: Int) = "{n} {карта.count(n)}"
    // n=1:  "1 карта"
    // n=11: "11 карт"   (many, not one!)
    // n=21: "21 карта"  (one)
    // n=12: "12 карт"   (many, not few!)
    // n=22: "22 карты"  (few)
}
```

---

## Gender Agreement

### Adjectives Must Match Noun Gender

```rust
phraselet! {
    // Adjective: целевой (target/targeted)
    adj целевой = {
        masc: {
            nom: "целевой" / "целевые",
            gen: "целевого" / "целевых",
            acc.inan: "целевой" / "целевые",
            acc.anim: "целевого" / "целевых",
            // ...
        },
        fem: {
            nom: "целевая" / "целевые",
            gen: "целевой" / "целевых",
            acc: "целевую" / "целевые",
            // ...
        },
        neut: {
            nom: "целевое" / "целевые",
            gen: "целевого" / "целевых",
            acc: "целевое" / "целевые",
            // ...
        },
    }

    // Usage - adjective agrees with noun
    целевой_союзник = "{целевой.agree(союзник).acc} {союзник.acc.sg}"
    // "целевого союзника" (masc.anim.acc = gen)

    целевая_карта = "{целевой.agree(карта).acc} {карта.acc.sg}"
    // "целевую карту" (fem.acc)
}
```

### Verb Agreement

Past tense verbs agree with subject gender:

```rust
phraselet! {
    // Verb: получить (to receive)
    verb получить = {
        past: {
            masc: "получил" / "получили",
            fem: "получила" / "получили",
            neut: "получило" / "получили",
        },
        imperative: "получите",  // formal/plural "you"
    }

    // Subject-verb agreement
    персонаж_получил(target: Predicate, amount: Int) =
        "{target.nom} {получить.past.agree(target)} +{amount} {искра.gen.count(amount)}."

    // "Союзник получил +3 искры." (masc)
    // "Карта получила +3 искры." (fem)  -- if card were the subject
}
```

---

## Ownership and Transforms

```rust
phraselet! {
    // Your characters → союзники (allies)
    transform ваш {
        персонаж => noun союзник(masc.animate) = { /* declension */ }
        карта => noun ваша_карта(fem) = { /* with "ваша" prefix */ }
    }

    // Enemy characters → враги (enemies)
    transform вражеский {
        персонаж => noun враг(masc.animate) = {
            nom: "враг" / "враги",
            gen: "врага" / "врагов",
            acc: "врага" / "врагов",
            // ...
        }
    }

    // Usage
    уничтожить_союзника = "Уничтожьте {ваш.персонаж.acc.sg}."
    // "Уничтожьте союзника."

    уничтожить_врага = "Уничтожьте {вражеский.персонаж.acc.sg}."
    // "Уничтожьте врага."
}
```

---

## Keywords

```rust
phraselet! {
    // Keywords with Russian translations
    keyword растворить = "<k>растворить</k>"
    keyword Растворите = "<k>Растворите</k>"  // Imperative form
    keyword предвидение(n: Int) = "<k>Предвидение</k> {n}"
    keyword разжечь(k: Int) = "<k>Разжечь</k> {k}"

    // Reclaim with cost
    keyword возврат_за(cost: Int) = "<k>Возврат</k> <e>{cost}</e>"
}
```

---

## Complete Examples

### Draw Cards Effect

```rust
phraselet! {
    // "Draw 3 cards" in Russian
    // Different structure based on count

    возьмите_карты(count: Int) = match count {
        one  => "Возьмите {count} карту."
        few  => "Возьмите {count} карты."
        many => "Возьмите {count} карт."
    }

    // Or using automatic agreement:
    возьмите_карты_auto(count: Int) =
        "Возьмите {count} {карта.case_for_number(count)}."
}
```

### Dissolve Effect

```rust
phraselet! {
    // "Dissolve an ally"
    // союзник is animate masculine, so accusative = genitive

    растворить_союзника = "{Растворите} {союзник.acc.sg}."
    // "Растворите союзника."

    // "Dissolve an enemy"
    растворить_врага = "{Растворите} {враг.acc.sg}."
    // "Растворите врага."

    // With constraint
    растворить_с_искрой(s: Int, op: Operator) =
        "{Растворите} {союзник.acc.sg} с {искра.ins.sg} {s}{op}."
    // "Растворите союзника с искрой 3 или больше."
}
```

### Gain Effect

```rust
phraselet! {
    // "This character gains +3 spark"
    // Verb agrees with subject, amount uses case rules

    получает_искру(target: Predicate, amount: Int) = match amount {
        one  => "{target.nom} получает +{amount} {искра.acc.sg}."
        few  => "{target.nom} получает +{amount} {искра.gen.sg}."
        many => "{target.nom} получает +{amount} {искра.gen.pl}."
    }

    // "Союзник получает +1 искру."
    // "Союзник получает +3 искры."
    // "Союзник получает +5 искр."
}
```

### Condition Text

```rust
phraselet! {
    // "If you control 3 or more allies"

    если_союзников(count: Int, op: Operator) = match count {
        one  => "если вы контролируете {count} {союзник.acc.sg}{op},"
        few  => "если вы контролируете {count} {союзник.gen.sg}{op},"
        many => "если вы контролируете {count} {союзник.gen.pl}{op},"
    }

    // "если вы контролируете 1 союзника или больше,"
    // "если вы контролируете 3 союзника или больше,"
    // "если вы контролируете 5 союзников или больше,"
}
```

### Card in Void

```rust
phraselet! {
    // "a card in your void" (prepositional case for location)

    noun пустота(fem) = {
        nom: "пустота",
        gen: "пустоты",
        pre: "пустоте",
        // ...
    }

    карта_в_пустоте = "{карта.nom.sg} в вашей {пустота.pre.sg}"
    // "карта в вашей пустоте"

    // "Return a card from your void" (genitive for "from")
    вернуть_из_пустоты =
        "Верните {карта.acc.sg} из вашей {пустота.gen.sg} в руку."
    // "Верните карту из вашей пустоты в руку."
}
```

---

## Pronoun Reference

```rust
phraselet! {
    // Pronouns must agree in gender with their antecedent

    pronoun он = {
        nom: "он" / "они",
        gen: "его" / "их",
        acc: "его" / "их",
        // ...
    }

    pronoun она = {
        nom: "она" / "они",
        gen: "её" / "их",
        acc: "её" / "их",
        // ...
    }

    pronoun оно = {
        nom: "оно" / "они",
        gen: "его" / "их",
        acc: "его" / "их",
        // ...
    }

    // "it gains" - pronoun agrees with the noun it references
    получает_возврат(target: Predicate) =
        "{pronoun.agree(target).nom} получает {возврат}."

    // If target was союзник (masc): "он получает возврат"
    // If target was карта (fem): "она получает возврат"
}
```

---

## Quick Reference: Case Usage

| Situation | Case | Example |
|-----------|------|---------|
| Subject | Nominative | Союзник получает... |
| Direct object | Accusative | Уничтожьте союзника |
| "of", possession | Genitive | искра союзника |
| After 5+ numbers | Genitive plural | 5 карт |
| After 2-4 numbers | Genitive singular | 3 карты |
| "to", recipient | Dative | дать союзнику |
| "with", using | Instrumental | с картой |
| "in", "at", "about" | Prepositional | в пустоте |

## Quick Reference: Number Agreement

| Number | Category | Noun form |
|--------|----------|-----------|
| 1, 21, 31... | one | nom.sg (карта) |
| 2, 3, 4, 22... | few | gen.sg (карты) |
| 0, 5-20, 25-30... | many | gen.pl (карт) |
| 11, 12, 13, 14 | many (!) | gen.pl (карт) |
