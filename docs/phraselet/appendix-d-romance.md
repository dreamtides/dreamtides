# Appendix D: Spanish & Portuguese Localization Guide

## Overview

Spanish and Portuguese share many grammatical features as Romance languages:
- **2 grammatical genders** (masculine, feminine)
- **Adjective agreement** with noun gender and number
- **No case system** (unlike Russian)
- **Article system** (definite and indefinite)

This guide covers both languages with notes on their differences.

---

## Grammatical Gender

### Defining Gendered Nouns

```rust
// Spanish (es.phr.rs)
phraselet! {
    // Feminine nouns (usually end in -a)
    noun carta(fem) = "carta" / "cartas"
    noun energía(fem) = "energía" / "energías"

    // Masculine nouns (usually end in -o or consonant)
    noun personaje(masc) = "personaje" / "personajes"
    noun jugador(masc) = "jugador" / "jugadores"
    noun evento(masc) = "evento" / "eventos"
}

// Portuguese (pt_br.phr.rs)
phraselet! {
    noun carta(fem) = "carta" / "cartas"
    noun energia(fem) = "energia" / "energias"  // no accent in PT
    noun personagem(masc) = "personagem" / "personagens"
    noun jogador(masc) = "jogador" / "jogadores"
    noun evento(masc) = "evento" / "eventos"
}
```

### Article Agreement

Articles must match noun gender:

```rust
// Spanish
phraselet! {
    // Indefinite articles: un (m), una (f)
    // Definite articles: el (m), la (f)

    article un(masc) = "un" / "unos"
    article una(fem) = "una" / "unas"
    article el(masc) = "el" / "los"
    article la(fem) = "la" / "las"

    // Usage - article matches noun
    sacar_carta = "Roba {una.agree(carta)} {carta}."
    // "Roba una carta."

    el_personaje = "{el.agree(personaje)} {personaje}"
    // "el personaje"
}

// Portuguese
phraselet! {
    article um(masc) = "um" / "uns"
    article uma(fem) = "uma" / "umas"
    article o(masc) = "o" / "os"
    article a(fem) = "a" / "as"
}
```

### Adjective Agreement

Adjectives change form to match noun gender AND number:

```rust
// Spanish
phraselet! {
    // Adjective with 4 forms: masc.sg, fem.sg, masc.pl, fem.pl
    adj destruido = {
        masc: "destruido" / "destruidos",
        fem: "destruida" / "destruidas",
    }

    adj aliado = {
        masc: "aliado" / "aliados",
        fem: "aliada" / "aliadas",
    }

    adj enemigo = {
        masc: "enemigo" / "enemigos",
        fem: "enemiga" / "enemigas",
    }

    // Usage - adjective agrees with noun
    carta_destruida = "{la.agree(carta)} {carta} fue {destruido.agree(carta)}."
    // "La carta fue destruida."

    personaje_destruido = "{el.agree(personaje)} {personaje} fue {destruido.agree(personaje)}."
    // "El personaje fue destruido."
}
```

---

## Portuguese Contractions

Brazilian Portuguese has mandatory contractions when certain prepositions meet articles:

| Preposition | + o (m.sg) | + a (f.sg) | + os (m.pl) | + as (f.pl) |
|-------------|------------|------------|-------------|-------------|
| de (of/from) | do | da | dos | das |
| em (in) | no | na | nos | nas |
| a (to) | ao | à | aos | às |
| por (by/through) | pelo | pela | pelos | pelas |

### Implementing Contractions

```rust
// pt_br.phr.rs
phraselet! {
    // Define contractions
    contraction de + o = "do"
    contraction de + a = "da"
    contraction de + os = "dos"
    contraction de + as = "das"

    contraction em + o = "no"
    contraction em + a = "na"
    contraction em + os = "nos"
    contraction em + as = "nas"

    contraction a + o = "ao"
    contraction a + a = "à"
    contraction a + os = "aos"
    contraction a + as = "às"

    // Usage - contractions applied automatically
    na_mão = "{em.contract(a.agree(mão))} {mão}"
    // "na mão" (in the hand) - em + a → na

    do_baralho = "{de.contract(o.agree(baralho))} {baralho}"
    // "do baralho" (from the deck) - de + o → do

    no_vazio = "{em.contract(o.agree(vazio))} {vazio}"
    // "no vazio" (in the void) - em + o → no
}
```

---

## Ownership Transforms

```rust
// Spanish
phraselet! {
    transform tu {
        personaje => noun aliado(masc) = "aliado" / "aliados"
        carta => "tu carta" / "tus cartas"
        evento => "tu evento" / "tus eventos"
    }

    transform enemigo {
        personaje => noun enemigo(masc) = "enemigo" / "enemigos"
        carta => "carta enemiga" / "cartas enemigas"
    }

    // Usage
    disuelve_aliado = "Disuelve {un.agree(aliado)} {aliado}."
    // "Disuelve un aliado."

    disuelve_enemigo = "Disuelve {un.agree(enemigo)} {enemigo}."
    // "Disuelve un enemigo."
}

// Portuguese
phraselet! {
    transform seu {
        personagem => noun aliado(masc) = "aliado" / "aliados"
        carta => "sua carta" / "suas cartas"
    }

    transform inimigo {
        personagem => noun inimigo(masc) = "inimigo" / "inimigos"
    }
}
```

---

## Pluralization

Spanish and Portuguese have the same plural rules as English (2 categories: one, other):

```rust
phraselet! {
    robar_cartas(count: Int) = match count {
        1 => "Roba 1 carta."
        _ => "Roba {count} cartas."
    }

    // Or using automatic selection
    robar(count: Int) = "Roba {count} {carta.count(count)}."
}
```

---

## Keywords

```rust
// Spanish
phraselet! {
    keyword disolver = "<k>disolver</k>"
    keyword Disuelve = "<k>Disuelve</k>"
    keyword desterrar = "<k>desterrar</k>"
    keyword Destierra = "<k>Destierra</k>"
    keyword prever(n: Int) = "<k>Prever</k> {n}"
    keyword avivar(k: Int) = "<k>Avivar</k> {k}"
    keyword recuperar = "<k>recuperar</k>"
    keyword Recupera = "<k>Recupera</k>"
}

// Portuguese
phraselet! {
    keyword dissolver = "<k>dissolver</k>"
    keyword Dissolva = "<k>Dissolva</k>"
    keyword banir = "<k>banir</k>"
    keyword Bana = "<k>Bana</k>"
    keyword prever(n: Int) = "<k>Prever</k> {n}"
    keyword inflamar(k: Int) = "<k>Inflamar</k> {k}"
    keyword recuperar = "<k>recuperar</k>"
    keyword Recupere = "<k>Recupere</k>"
}
```

---

## Regional Variants

### Spanish: Spain vs Latin America

```rust
// es.phr.rs (base Spanish - Latin America)
phraselet! {
    // "You" forms - Latin America uses "ustedes" (formal plural)
    ustedes_roban = "Ustedes roban una carta."
}

// es_es.phr.rs (Spain variant)
phraselet! {
    // Spain uses "vosotros" (informal plural)
    vosotros_robáis = "Vosotros robáis una carta."
}

// Or handle with a variant flag:
phraselet! {
    robar_ustedes(region: Region) = match region {
        LatinAmerica => "Ustedes roban una carta."
        Spain => "Vosotros robáis una carta."
    }
}
```

### Vocabulary Differences

| Concept | Spain | Latin America |
|---------|-------|---------------|
| computer | ordenador | computadora |
| to take | coger | tomar |
| to grab | agarrar | agarrar |

For card games, most terminology is shared, but always verify with regional players.

---

## Complete Examples

### Draw Cards

```rust
// Spanish
phraselet! {
    robar_cartas(count: Int) = match count {
        1 => "Roba 1 carta."
        _ => "Roba {count} cartas."
    }

    // With condition
    robar_si(count: Int, condition: Condition) =
        "{condition}, roba {count} {carta.count(count)}."
}

// Portuguese
phraselet! {
    comprar_cartas(count: Int) = match count {
        1 => "Compre 1 carta."
        _ => "Compre {count} cartas."
    }
}
```

### Dissolve Effect

```rust
// Spanish
phraselet! {
    disolver_objetivo(target: Predicate) =
        "{Disuelve} {un.agree(target)} {target}."

    // "Disuelve un aliado."
    // "Disuelve una carta."

    // With gender tracking
    disolver_con_chispa(target: Predicate, s: Int, op: Operator) =
        "{Disuelve} {un.agree(target)} {target} con chispa {s}{op}."
}

// Portuguese
phraselet! {
    dissolver_alvo(target: Predicate) =
        "{Dissolva} {um.agree(target)} {target}."
}
```

### Gain Spark

```rust
// Spanish
phraselet! {
    noun chispa(fem) = "chispa" / "chispas"

    ganar_chispa(target: Predicate, amount: Int) =
        "{target} gana +{amount} {chispa.count(amount)}."
    // "El aliado gana +3 chispas."
}

// Portuguese
phraselet! {
    noun faísca(fem) = "faísca" / "faíscas"

    ganhar_faísca(target: Predicate, amount: Int) =
        "{target} ganha +{amount} {faísca.count(amount)}."
}
```

### Cards in Void

```rust
// Spanish
phraselet! {
    noun vacío(masc) = "vacío" / "vacíos"

    carta_en_vacío = "{una.agree(carta)} {carta} en tu {vacío}"
    // "una carta en tu vacío"

    devolver_del_vacío(target: Predicate) =
        "Devuelve {un.agree(target)} {target} de tu {vacío} a tu mano."
}

// Portuguese - note the mandatory contractions
phraselet! {
    noun vazio(masc) = "vazio" / "vazios"

    carta_no_vazio = "{uma.agree(carta)} {carta} {em.contract(o.agree(vazio))} seu {vazio}"
    // "uma carta no seu vazio" (no = em + o)

    devolver_do_vazio(target: Predicate) =
        "Devolva {um.agree(target)} {target} {de.contract(o.agree(vazio))} seu {vazio} para sua mão."
    // "do seu vazio" (do = de + o)
}
```

### Triggered Abilities

```rust
// Spanish
phraselet! {
    materializado = "▸ <b>Materializado:</b>"
    disuelto = "▸ <b>Disuelto:</b>"
    juicio = "▸ <b>Juicio:</b>"

    habilidad_materializado(effect: String) =
        "{materializado} {effect}"
}

// Portuguese
phraselet! {
    materializado = "▸ <b>Materializado:</b>"
    dissolvido = "▸ <b>Dissolvido:</b>"
    julgamento = "▸ <b>Julgamento:</b>"
}
```

---

## Pronoun Reference

```rust
// Spanish - pronouns agree with antecedent
phraselet! {
    pronoun él(masc) = "él" / "ellos"
    pronoun ella(fem) = "ella" / "ellas"

    // Object pronouns
    object_pronoun lo(masc) = "lo" / "los"
    object_pronoun la(fem) = "la" / "las"

    // "it gains" - pronoun agrees with target
    gana_recuperar(target: Predicate) =
        "{object_pronoun.agree(target)} gana {recuperar}."
    // For masculine target: "lo gana recuperar"
    // For feminine target: "la gana recuperar"
}

// Portuguese
phraselet! {
    pronoun ele(masc) = "ele" / "eles"
    pronoun ela(fem) = "ela" / "elas"

    object_pronoun o(masc) = "o" / "os"
    object_pronoun a(fem) = "a" / "as"
}
```

---

## Quick Reference: Gender Assignment

### Common Endings

| Ending | Gender | Examples |
|--------|--------|----------|
| -a | Feminine | carta, chispa, energía |
| -o | Masculine | personaje, evento, vacío |
| -ción/-sión | Feminine | acción, decisión |
| -dad/-tad | Feminine | habilidad, libertad |
| -or | Masculine | jugador, valor |
| -aje | Masculine | personaje, mensaje |

### Exceptions

Some words don't follow the pattern:
- el día (masc) - "the day" ends in -a but is masculine
- la mano (fem) - "the hand" ends in -o but is feminine
- el problema (masc) - "the problem" ends in -a but is masculine

---

## Differences Between Spanish and Portuguese

| Feature | Spanish | Portuguese (BR) |
|---------|---------|-----------------|
| "You" (formal sg) | usted | você |
| "You" (plural) | ustedes | vocês |
| "You" (Spain, informal pl) | vosotros | — |
| Contractions | rare | mandatory (de+o=do) |
| Object pronoun placement | before verb | varies |
| Present progressive | está + -ando/-iendo | está + -ando/-endo |

### Translation Tips

1. **Portuguese requires contractions** - never write "de o", always "do"
2. **Different vocabulary** - verify game terms with native speakers
3. **Object pronoun placement** - Portuguese has more flexibility
4. **Accents matter** - energía (ES) vs energia (PT)
