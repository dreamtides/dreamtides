# Appendix E: German Localization Guide

## Overview

German has complex grammar that combines several features:
- **4 grammatical cases** (nominative, accusative, dative, genitive)
- **3 genders** (masculine, feminine, neuter)
- **Definite/indefinite articles** that change by case AND gender
- **Adjective endings** that depend on article type, case, gender, and number
- **Compound nouns** - words combine into single long words
- **Verb position** - verbs move in subordinate clauses

## The Case + Gender Matrix

German articles form a matrix of case × gender:

### Definite Articles (der/die/das)

| Case | Masculine | Feminine | Neuter | Plural |
|------|-----------|----------|--------|--------|
| Nominative | der | die | das | die |
| Accusative | den | die | das | die |
| Dative | dem | der | dem | den |
| Genitive | des | der | des | der |

### Indefinite Articles (ein/eine)

| Case | Masculine | Feminine | Neuter |
|------|-----------|----------|--------|
| Nominative | ein | eine | ein |
| Accusative | einen | eine | ein |
| Dative | einem | einer | einem |
| Genitive | eines | einer | eines |

### Defining Articles in Phraselet

```rust
phraselet! {
    // Definite articles
    article der = {
        masc: { nom: "der", acc: "den", dat: "dem", gen: "des" },
        fem:  { nom: "die", acc: "die", dat: "der", gen: "der" },
        neut: { nom: "das", acc: "das", dat: "dem", gen: "des" },
        pl:   { nom: "die", acc: "die", dat: "den", gen: "der" },
    }

    // Indefinite articles
    article ein = {
        masc: { nom: "ein", acc: "einen", dat: "einem", gen: "eines" },
        fem:  { nom: "eine", acc: "eine", dat: "einer", gen: "einer" },
        neut: { nom: "ein", acc: "ein", dat: "einem", gen: "eines" },
        // No plural for "ein"
    }
}
```

---

## Noun Declension

German nouns change minimally (mainly adding -(e)n in dative plural and -s/-es in genitive for some masculines/neuters):

```rust
phraselet! {
    // Feminine noun: die Karte (card)
    noun Karte(fem) = {
        nom: "Karte" / "Karten",
        acc: "Karte" / "Karten",
        dat: "Karte" / "Karten",
        gen: "Karte" / "Karten",
    }

    // Masculine noun: der Charakter (character)
    noun Charakter(masc) = {
        nom: "Charakter" / "Charaktere",
        acc: "Charakter" / "Charaktere",
        dat: "Charakter" / "Charakteren",  // -n in dative plural
        gen: "Charakters" / "Charaktere",  // -s in genitive singular
    }

    // Neuter noun: das Ereignis (event)
    noun Ereignis(neut) = {
        nom: "Ereignis" / "Ereignisse",
        acc: "Ereignis" / "Ereignisse",
        dat: "Ereignis" / "Ereignissen",
        gen: "Ereignisses" / "Ereignisse",
    }

    // Masculine noun: der Verbündete (ally) - adjectival noun!
    noun Verbündete(masc) = {
        nom: "Verbündete" / "Verbündeten",   // weak declension
        acc: "Verbündeten" / "Verbündeten",
        dat: "Verbündeten" / "Verbündeten",
        gen: "Verbündeten" / "Verbündeten",
    }
}
```

---

## Adjective Endings

German adjectives have THREE declension patterns based on what precedes them:

### 1. Strong (no article)

| Case | Masc | Fem | Neut | Plural |
|------|------|-----|------|--------|
| Nom | -er | -e | -es | -e |
| Acc | -en | -e | -es | -e |
| Dat | -em | -er | -em | -en |
| Gen | -en | -er | -en | -er |

### 2. Mixed (after ein/eine/kein)

| Case | Masc | Fem | Neut | Plural |
|------|------|-----|------|--------|
| Nom | -er | -e | -es | -en |
| Acc | -en | -e | -es | -en |
| Dat | -en | -en | -en | -en |
| Gen | -en | -en | -en | -en |

### 3. Weak (after der/die/das)

| Case | Masc | Fem | Neut | Plural |
|------|------|-----|------|--------|
| Nom | -e | -e | -e | -en |
| Acc | -en | -e | -e | -en |
| Dat | -en | -en | -en | -en |
| Gen | -en | -en | -en | -en |

### Implementing Adjectives

```rust
phraselet! {
    // Adjective: feindlich (enemy/hostile)
    adj feindlich = {
        strong: {
            masc: { nom: "feindlicher", acc: "feindlichen", ... },
            fem:  { nom: "feindliche", acc: "feindliche", ... },
            neut: { nom: "feindliches", acc: "feindliches", ... },
            pl:   { nom: "feindliche", acc: "feindliche", ... },
        },
        mixed: {
            masc: { nom: "feindlicher", acc: "feindlichen", ... },
            fem:  { nom: "feindliche", acc: "feindliche", ... },
            neut: { nom: "feindliches", acc: "feindliches", ... },
            pl:   { nom: "feindlichen", acc: "feindlichen", ... },
        },
        weak: {
            masc: { nom: "feindliche", acc: "feindlichen", ... },
            fem:  { nom: "feindliche", acc: "feindliche", ... },
            neut: { nom: "feindliche", acc: "feindliche", ... },
            pl:   { nom: "feindlichen", acc: "feindlichen", ... },
        },
    }

    // Usage with article type detection
    der_feindliche = "{der.acc} {feindlich.weak(Charakter).acc} {Charakter.acc}"
    // "den feindlichen Charakter"

    ein_feindlicher = "{ein.acc} {feindlich.mixed(Charakter).acc} {Charakter.acc}"
    // "einen feindlichen Charakter"
}
```

---

## Case Usage

| Case | Usage | Example |
|------|-------|---------|
| Nominative | Subject | Der Charakter gewinnt... |
| Accusative | Direct object, movement | Zerstöre den Charakter |
| Dative | Indirect object, location | mit dem Charakter |
| Genitive | Possession | des Charakters Stärke |

### Common Prepositions and Their Cases

```rust
phraselet! {
    // Accusative prepositions
    // für (for), durch (through), gegen (against), ohne (without)

    // Dative prepositions
    // mit (with), bei (at), nach (after), von (from), zu (to), aus (out of)

    // Two-way prepositions (accusative for motion, dative for location)
    // in, an, auf, unter, über, vor, hinter, neben, zwischen

    // Examples
    mit_dem_Charakter = "mit {der.dat} {Charakter.dat}"  // "mit dem Charakter"
    in_die_Hand = "in {der.acc.fem} Hand"                // "in die Hand" (motion)
    in_der_Hand = "in {der.dat.fem} Hand"                // "in der Hand" (location)
}
```

---

## Compound Nouns

German famously creates compound nouns:

```rust
phraselet! {
    // Compound nouns - the last component determines gender
    noun Kartenspiel(neut) = "Kartenspiel" / "Kartenspiele"
    // Karte (card) + Spiel (game) = Kartenspiel (card game)

    noun Handkarte(fem) = "Handkarte" / "Handkarten"
    // Hand (hand) + Karte (card) = Handkarte (hand card)

    noun Friedhof(masc) = "Friedhof" / "Friedhöfe"
    // "Void" might be translated as Friedhof (graveyard) or Leere (void)

    noun Ablagestapel(masc) = "Ablagestapel" / "Ablagestapel"
    // "Discard pile"
}
```

---

## Verb Position

In German main clauses, the verb is in second position. In subordinate clauses, it moves to the end:

```rust
phraselet! {
    // Main clause: verb in position 2
    ziehe_karten(count: Int) = "Ziehe {count} {Karte.count(count)}."
    // "Ziehe 3 Karten." (Draw 3 cards.)

    // Subordinate clause: verb at end
    wenn_du_ziehst = "wenn du eine Karte ziehst"
    // "wenn du eine Karte ziehst" (when you draw a card)
    // Note: "ziehst" moves to the end

    // Conditional with verb-final
    wenn_bedingung(condition: String, effect: String) =
        "{condition}, {effect}."
    // The effect clause keeps verb in position 2
}
```

---

## Ownership Transforms

```rust
phraselet! {
    transform dein {
        Charakter => noun Verbündeter(masc) = {
            nom: "Verbündeter" / "Verbündete",
            acc: "Verbündeten" / "Verbündete",
            // ...
        }
        Karte => "deine Karte" / "deine Karten"
        Ereignis => "dein Ereignis" / "deine Ereignisse"
    }

    transform feindlich {
        Charakter => noun Feind(masc) = {
            nom: "Feind" / "Feinde",
            acc: "Feind" / "Feinde",
            // ...
        }
    }

    // Usage
    zerstöre_verbündeten = "Zerstöre {ein.acc} {dein.Charakter.acc}."
    // "Zerstöre einen Verbündeten."
}
```

---

## Keywords

```rust
phraselet! {
    keyword zerstören = "<k>zerstören</k>"
    keyword Zerstöre = "<k>Zerstöre</k>"
    keyword verbannen = "<k>verbannen</k>"
    keyword Verbanne = "<k>Verbanne</k>"
    keyword Vorausschau(n: Int) = "<k>Vorausschau</k> {n}"
    keyword Entfachen(k: Int) = "<k>Entfachen</k> {k}"
    keyword Rückholung = "<k>Rückholung</k>"
}
```

---

## Complete Examples

### Draw Cards

```rust
phraselet! {
    Karten_ziehen(count: Int) = match count {
        1 => "Ziehe 1 Karte."
        _ => "Ziehe {count} Karten."
    }
}
```

### Dissolve Effect

```rust
phraselet! {
    // "Destroy an enemy" - accusative case
    Feind_zerstören = "{Zerstöre} {ein.acc.masc} {Feind.acc}."
    // "Zerstöre einen Feind."

    // With constraint - article + adjective agreement
    Charakter_zerstören_mit_funke(s: Int, op: Operator) =
        "{Zerstöre} {ein.acc} {Charakter.acc} mit Funke {s}{op}."
    // "Zerstöre einen Charakter mit Funke 3 oder mehr."
}
```

### Gain Effect

```rust
phraselet! {
    noun Funke(masc) = {
        nom: "Funke" / "Funken",
        acc: "Funken" / "Funken",
        dat: "Funken" / "Funken",
        gen: "Funkens" / "Funken",
    }

    Funke_erhalten(target: Predicate, amount: Int) = match amount {
        1 => "{target.nom} erhält +1 {Funke.acc.sg}."
        _ => "{target.nom} erhält +{amount} {Funke.acc.pl}."
    }
    // "Dieser Charakter erhält +3 Funken."
}
```

### Cards in Void

```rust
phraselet! {
    noun Friedhof(masc) = {
        nom: "Friedhof",
        dat: "Friedhof",
        gen: "Friedhofs",
        // ...
    }

    // "in your void" - dative for location
    in_deinem_Friedhof = "in deinem {Friedhof.dat}"
    // "in deinem Friedhof"

    // "from your void" - dative with "aus"
    aus_deinem_Friedhof = "aus deinem {Friedhof.dat}"
    // "aus deinem Friedhof"

    // "a card from your void"
    Karte_aus_Friedhof = "{ein.nom.fem} {Karte.nom} aus deinem {Friedhof.dat}"
    // "eine Karte aus deinem Friedhof"
}
```

### Triggered Abilities

```rust
phraselet! {
    Materialisiert = "▸ <b>Materialisiert:</b>"
    Aufgelöst = "▸ <b>Aufgelöst:</b>"
    Urteil = "▸ <b>Urteil:</b>"
}
```

---

## Quick Reference

### Case Usage Summary

| Situation | Case |
|-----------|------|
| Subject | Nominative |
| Direct object | Accusative |
| Indirect object | Dative |
| After mit, bei, von, zu | Dative |
| After für, durch, gegen | Accusative |
| Possession | Genitive |
| Motion into | Accusative |
| Location at | Dative |

### Article Selection Summary

1. Identify noun gender
2. Identify required case
3. Choose article type (der/ein/kein)
4. Look up in matrix

### Adjective Ending Summary

1. Determine article type → strong/mixed/weak
2. Determine noun gender
3. Determine case
4. Apply ending

---

## Common Pitfalls

1. **Adjective endings after articles** - Use weak endings after der/die/das
2. **Dative plural -n** - Most nouns add -n in dative plural
3. **Compound noun gender** - Always determined by the LAST component
4. **Verb position** - Remember to move verb to end in subordinate clauses
5. **Accusative vs Dative** - Two-way prepositions require careful attention
