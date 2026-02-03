# Appendix: RLT Standard Library

This appendix documents the standard transforms and metadata tags provided by RLT for the world's most widely-spoken languages.

## Overview

RLT provides three categories of transforms:

1. **Universal transforms**: Work on any text in any language
2. **Language-family transforms**: Shared across related languages
3. **Language-specific transforms**: Unique to individual languages

---

## Universal Transforms

These transforms work identically in all languages:

| Transform | Effect | Example |
|-----------|--------|---------|
| `@cap` | Capitalize first letter | "card" → "Card" |
| `@upper` | All uppercase | "card" → "CARD" |
| `@lower` | All lowercase | "Card" → "card" |

---

## Language Reference

### English

**Grammatical features**: No gender, no case, simple plural (one/other)

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:a` | Use "a" as indefinite article (required for `@a`) |
| `:an` | Use "an" as indefinite article (required for `@a`) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@a` | `:a`, `:an` (required) | Prepend indefinite article; errors if tag missing |
| `@the` | - | Prepend "the" |

**Plural categories**: `one`, `other`

```rust
// en.rlt.rs
rlt! {
    card = "card" :a;
    event = "event" :an;
    ally = "ally" :an;
    hour = "hour" :an;       // silent h
    uniform = "uniform" :a;   // /juː/ sound

    card = { one: "card", other: "cards" };

    draw_one = "Draw {@a card}.";      // → "Draw a card."
    play_one = "Play {@a event}.";     // → "Play an event."
    the_card = "{@the card}";          // → "the card"
}
```

---

### Mandarin Chinese (简体中文)

**Grammatical features**: No plural, no gender, no case, measure words required

**Metadata tags**:
| Tag | Purpose | Measure word |
|-----|---------|--------------|
| `:zhang` | Flat objects (cards, paper) | 张 |
| `:ge` | General classifier | 个 |
| `:ming` | People (formal) | 名 |
| `:wei` | People (respectful) | 位 |
| `:tiao` | Long thin objects | 条 |
| `:ben` | Books, volumes | 本 |
| `:zhi` | Animals, hands | 只 |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@count` | measure word tags | Insert number + measure word |

**Plural categories**: `other` (no plural distinction)

```rust
// zh_cn.rlt.rs
rlt! {
    pai = "牌" :zhang;
    jue_se = "角色" :ge;
    wan_jia = "玩家" :ming;

    // @count inserts number + measure word
    draw(n) = "抽{@count n pai}";       // n=3 → "抽3张牌"
    summon(n) = "召唤{@count n jue_se}"; // n=2 → "召唤2个角色"
}
```

---

### Hindi (हिन्दी)

**Grammatical features**: Two genders (masc/fem), two numbers, three cases (direct/oblique/vocative)

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |

**Plural categories**: `one`, `other`

```rust
// hi.rlt.rs
rlt! {
    card = "कार्ड" :masc;
    event = "घटना" :fem;

    card = {
        dir.one: "कार्ड",
        dir.other: "कार्ड",
        obl.one: "कार्ड",
        obl.other: "कार्डों",
    };
}
```

---

### Spanish (Español)

**Grammatical features**: Two genders (masc/fem), definite and indefinite articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@el` | `:masc`, `:fem` | Definite article (el/la/los/las) |
| `@un` | `:masc`, `:fem` | Indefinite article (un/una/unos/unas) |

**Plural categories**: `one`, `other`

```rust
// es.rlt.rs
rlt! {
    card = "carta" :fem;
    enemy = "enemigo" :masc;

    card = { one: "carta", other: "cartas" };
    enemy = { one: "enemigo", other: "enemigos" };

    destroyed = { masc: "destruido", fem: "destruida" };

    draw_one = "Roba {@un card}.";           // → "Roba una carta."
    the_enemy = "{@el enemy}";               // → "el enemigo"
    destroy(x) = "{x} fue {destroyed:x}.";   // → "carta fue destruida."
}
```

---

### French (Français)

**Grammatical features**: Two genders, articles, contractions, elision

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |
| `:vowel` | Starts with vowel sound (for elision) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@le` | `:masc`, `:fem`, `:vowel` | Definite article (le/la/l'/les) |
| `@un` | `:masc`, `:fem` | Indefinite article (un/une) |
| `@de` | `:masc`, `:fem`, `:vowel` | "de" + article (du/de la/de l'/des) |
| `@a` | `:masc`, `:fem`, `:vowel` | "à" + article (au/à la/à l'/aux) |

**Plural categories**: `one`, `other`

```rust
// fr.rlt.rs
rlt! {
    card = "carte" :fem;
    enemy = "ennemi" :masc :vowel;
    friend = "ami" :masc :vowel;
    void = "vide" :masc;
    hand = "main" :fem;

    the_card = "{@le card}";      // → "la carte"
    the_enemy = "{@le enemy}";    // → "l'ennemi" (elision)
    from_void = "{@de void}";     // → "du vide"
    to_hand = "{@a hand}";        // → "à la main"
}
```

---

### Arabic (العربية)

**Grammatical features**: Two genders, three numbers (singular/dual/plural), definite article, complex agreement

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |
| `:sun` | Sun letter (assimilates ال) |
| `:moon` | Moon letter (no assimilation) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@al` | `:sun`, `:moon` | Definite article with assimilation |

**Plural categories**: `zero`, `one`, `two`, `few`, `many`, `other`

```rust
// ar.rlt.rs
rlt! {
    card = "بطاقة" :fem :moon;

    card = {
        one: "بطاقة",
        two: "بطاقتان",
        few: "بطاقات",
        many: "بطاقة",
        other: "بطاقات",
    };
}
```

---

### Bengali (বাংলা)

**Grammatical features**: No gender, classifiers for counting

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:ta` | General classifier (টা) |
| `:ti` | Formal classifier (টি) |
| `:khana` | For flat objects (খানা) |
| `:jon` | For people (জন) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@count` | classifier tags | Number + classifier |

**Plural categories**: `one`, `other`

---

### Portuguese (Português)

**Grammatical features**: Two genders, articles, contractions

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@o` | `:masc`, `:fem` | Definite article (o/a/os/as) |
| `@um` | `:masc`, `:fem` | Indefinite article (um/uma) |
| `@de` | `:masc`, `:fem` | "de" + article (do/da/dos/das) |
| `@em` | `:masc`, `:fem` | "em" + article (no/na/nos/nas) |

**Plural categories**: `one`, `other`

```rust
// pt_br.rlt.rs
rlt! {
    card = "carta" :fem;
    enemy = "inimigo" :masc;
    void = "vazio" :masc;
    hand = "mão" :fem;

    the_card = "{@o card}";      // → "a carta"
    from_void = "{@de void}";    // → "do vazio"
    in_hand = "{@em hand}";      // → "na mão"
}
```

---

### Russian (Русский)

**Grammatical features**: Three genders, six cases, complex plural

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine gender |
| `:fem` | Feminine gender |
| `:neut` | Neuter gender |
| `:anim` | Animate (affects accusative) |
| `:inan` | Inanimate |

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `prep`

```rust
// ru.rlt.rs
rlt! {
    card = "карта" :fem :inan;
    ally = "союзник" :masc :anim;

    card = {
        nom.one: "карта",
        nom.few: "карты",
        nom.many: "карт",
        acc.one: "карту",
        acc.few: "карты",
        acc.many: "карт",
        gen.one: "карты",
        gen.few: "карт",
        gen.many: "карт",
        // ... dat, ins, prep
    };
}
```

---

### Japanese (日本語)

**Grammatical features**: No plural, no gender, counters (similar to Chinese), particles

**Metadata tags**:
| Tag | Purpose | Counter |
|-----|---------|---------|
| `:mai` | Flat objects | 枚 |
| `:nin` | People | 人 |
| `:hiki` | Small animals | 匹 |
| `:hon` | Long objects | 本 |
| `:ko` | General small objects | 個 |
| `:satsu` | Books | 冊 |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@count` | counter tags | Number + counter |

**Plural categories**: `other` (no plural distinction)

```rust
// ja.rlt.rs
rlt! {
    card = "カード" :mai;
    character = "キャラクター" :nin;

    draw(n) = "{@count n card}を引く";  // n=3 → "3枚のカードを引く"
}
```

---

### German (Deutsch)

**Grammatical features**: Three genders, four cases, definite/indefinite articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine (der) |
| `:fem` | Feminine (die) |
| `:neut` | Neuter (das) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@der` | `:masc`, `:fem`, `:neut` + case | Definite article (der/die/das/den/dem/des) |
| `@ein` | `:masc`, `:fem`, `:neut` + case | Indefinite article (ein/eine/einen/einem/einer/eines) |

**Plural categories**: `one`, `other`

**Case variants**: `nom`, `acc`, `dat`, `gen`

```rust
// de.rlt.rs
rlt! {
    karte = "Karte" :fem;
    charakter = "Charakter" :masc;
    ereignis = "Ereignis" :neut;

    karte = {
        nom.one: "Karte",
        nom.other: "Karten",
        acc.one: "Karte",
        acc.other: "Karten",
        dat.one: "Karte",
        dat.other: "Karten",
        gen.one: "Karte",
        gen.other: "Karten",
    };

    the_card = "{@der:nom karte}";  // → "die Karte"
    a_char = "{@ein:acc charakter}"; // → "einen Charakter"
}
```

---

### Korean (한국어)

**Grammatical features**: No gender, counters, particles, honorifics

**Metadata tags**:
| Tag | Purpose | Counter |
|-----|---------|---------|
| `:jang` | Flat objects | 장 |
| `:myeong` | People (formal) | 명 |
| `:mari` | Animals | 마리 |
| `:gae` | General objects | 개 |
| `:gwon` | Books | 권 |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@count` | counter tags | Number + counter (Korean or Sino-Korean) |

**Plural categories**: `other` (no plural distinction)

```rust
// ko.rlt.rs
rlt! {
    card = "카드" :jang;
    character = "캐릭터" :myeong;

    draw(n) = "{@count n card}를 뽑는다";  // n=3 → "카드 3장을 뽑는다"
}
```

---

### Vietnamese (Tiếng Việt)

**Grammatical features**: No inflection, classifiers

**Metadata tags**:
| Tag | Purpose | Classifier |
|-----|---------|------------|
| `:cai` | General objects | cái |
| `:con` | Animals, some objects | con |
| `:nguoi` | People | người |
| `:chiec` | Vehicles, single items | chiếc |
| `:to` | Flat paper items | tờ |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@count` | classifier tags | Number + classifier |

**Plural categories**: `other` (no plural distinction)

---

### Turkish (Türkçe)

**Grammatical features**: Vowel harmony, agglutinative, no gender, six cases

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:front` | Front vowels (e, i, ö, ü) |
| `:back` | Back vowels (a, ı, o, u) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@acc` | `:front`, `:back` | Accusative suffix (-i/-ı/-u/-ü) |
| `@dat` | `:front`, `:back` | Dative suffix (-e/-a) |
| `@loc` | `:front`, `:back` | Locative suffix (-de/-da/-te/-ta) |
| `@abl` | `:front`, `:back` | Ablative suffix (-den/-dan/-ten/-tan) |

**Plural categories**: `one`, `other`

```rust
// tr.rlt.rs
rlt! {
    card = "kart" :back;
    hand = "el" :front;
    void = "boşluk" :back;

    to_hand = "{@dat hand}";     // → "ele"
    from_void = "{@abl void}";   // → "boşluktan"
}
```

---

### Italian (Italiano)

**Grammatical features**: Two genders, articles, contractions, elision

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine |
| `:fem` | Feminine |
| `:vowel` | Starts with vowel |
| `:s_imp` | Starts with s+consonant, z, gn, ps, x |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@il` | gender + sound tags | Definite article (il/lo/la/l'/i/gli/le) |
| `@un` | gender + sound tags | Indefinite article (un/uno/una/un') |
| `@di` | gender + sound tags | "di" + article (del/dello/della/dell'/dei/degli/delle) |
| `@a` | gender + sound tags | "a" + article (al/allo/alla/all'/ai/agli/alle) |

**Plural categories**: `one`, `other`

```rust
// it.rlt.rs
rlt! {
    card = "carta" :fem;
    student = "studente" :masc :s_imp;
    friend = "amico" :masc :vowel;

    the_card = "{@il card}";       // → "la carta"
    the_student = "{@il student}"; // → "lo studente"
    the_friend = "{@il friend}";   // → "l'amico"
}
```

---

### Polish (Polski)

**Grammatical features**: Three genders, seven cases, complex plural, animate distinction

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc_anim` | Masculine animate |
| `:masc_inan` | Masculine inanimate |
| `:fem` | Feminine |
| `:neut` | Neuter |

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc`

```rust
// pl.rlt.rs
rlt! {
    card = "karta" :fem;
    enemy = "wróg" :masc_anim;

    card = {
        nom.one: "karta",
        nom.few: "karty",
        nom.many: "kart",
        acc.one: "kartę",
        acc.few: "karty",
        acc.many: "kart",
        // ... etc
    };
}
```

---

### Ukrainian (Українська)

**Grammatical features**: Three genders, seven cases, complex plural (same as Russian/Polish family)

**Metadata tags**: Same as Russian (`:masc`, `:fem`, `:neut`, `:anim`, `:inan`)

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc`

---

### Dutch (Nederlands)

**Grammatical features**: Two genders (common/neuter for articles), definite/indefinite articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:de` | Common gender (de-words) |
| `:het` | Neuter gender (het-words) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@de` | `:de`, `:het` | Definite article (de/het) |
| `@een` | - | Indefinite article (een) |

**Plural categories**: `one`, `other`

```rust
// nl.rlt.rs
rlt! {
    card = "kaart" :de;
    character = "karakter" :het;

    the_card = "{@de card}";        // → "de kaart"
    the_char = "{@de character}";   // → "het karakter"
}
```

---

### Thai (ภาษาไทย)

**Grammatical features**: No inflection, classifiers

**Metadata tags**:
| Tag | Purpose | Classifier |
|-----|---------|------------|
| `:bai` | Flat objects, cards | ใบ |
| `:tua` | Animals, letters, characters | ตัว |
| `:khon` | People | คน |
| `:an` | General small objects | อัน |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@count` | classifier tags | Number + classifier |

**Plural categories**: `other` (no plural distinction)

---

### Indonesian (Bahasa Indonesia)

**Grammatical features**: No inflection, no gender, reduplication for plural

**Metadata tags**: None required

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@plural` | - | Reduplication (kartu → kartu-kartu) |

**Plural categories**: `other` (context-dependent)

```rust
// id.rlt.rs
rlt! {
    card = "kartu";

    all_cards = "semua {@plural card}";  // → "semua kartu-kartu"
}
```

---

### Persian (فارسی)

**Grammatical features**: No gender, ezafe construction, simple plural

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:vowel` | Ends in vowel (affects ezafe) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@ezafe` | `:vowel` | Ezafe connector (-e/-ye) |

**Plural categories**: `one`, `other`

```rust
// fa.rlt.rs
rlt! {
    card = "کارت";
    hand = "دست" :vowel;

    card_of_player = "{@ezafe card} بازیکن";  // → "کارت‌ِ بازیکن"
}
```

---

### Romanian (Română)

**Grammatical features**: Three genders, postposed definite article, two cases

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine |
| `:fem` | Feminine |
| `:neut` | Neuter (masc singular, fem plural) |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@def` | gender | Postposed definite article |

**Plural categories**: `one`, `few`, `other`

```rust
// ro.rlt.rs
rlt! {
    card = "carte" :fem;

    the_card = "{@def card}";  // → "cartea"
}
```

---

### Greek (Ελληνικά)

**Grammatical features**: Three genders, four cases, articles

**Metadata tags**:
| Tag | Purpose |
|-----|---------|
| `:masc` | Masculine |
| `:fem` | Feminine |
| `:neut` | Neuter |

**Transforms**:
| Transform | Reads | Effect |
|-----------|-------|--------|
| `@o` | gender + case | Definite article (ο/η/το/τον/την/του/της/οι/τα...) |
| `@enas` | gender + case | Indefinite article |

**Plural categories**: `one`, `other`

---

### Czech (Čeština)

**Grammatical features**: Three genders, seven cases, animate distinction

**Metadata tags**: Same as Polish

**Plural categories**: `one`, `few`, `many`, `other`

**Case variants**: `nom`, `acc`, `gen`, `dat`, `ins`, `loc`, `voc`

---

## Summary Table

| Language | Gender | Cases | Plural Forms | Key Transforms |
|----------|--------|-------|--------------|----------------|
| English | - | - | 2 | `@a`, `@the` |
| Chinese | - | - | 1 | `@count` |
| Hindi | 2 | 3 | 2 | - |
| Spanish | 2 | - | 2 | `@el`, `@un` |
| French | 2 | - | 2 | `@le`, `@un`, `@de`, `@a` |
| Arabic | 2 | 3 | 6 | `@al` |
| Bengali | - | - | 2 | `@count` |
| Portuguese | 2 | - | 2 | `@o`, `@um`, `@de`, `@em` |
| Russian | 3 | 6 | 4 | - |
| Japanese | - | - | 1 | `@count` |
| German | 3 | 4 | 2 | `@der`, `@ein` |
| Korean | - | - | 1 | `@count` |
| Vietnamese | - | - | 1 | `@count` |
| Turkish | - | 6 | 2 | `@acc`, `@dat`, `@loc`, `@abl` |
| Italian | 2 | - | 2 | `@il`, `@un`, `@di`, `@a` |
| Polish | 3 | 7 | 4 | - |
| Ukrainian | 3 | 7 | 4 | - |
| Dutch | 2 | - | 2 | `@de`, `@een` |
| Thai | - | - | 1 | `@count` |
| Indonesian | - | - | 1 | `@plural` |
| Persian | - | - | 2 | `@ezafe` |
| Romanian | 3 | 2 | 3 | `@def` |
| Greek | 3 | 4 | 2 | `@o`, `@enas` |
| Czech | 3 | 7 | 4 | - |

---

## Design Notes

### Required Metadata Tags

Metadata-driven transforms require their expected tags to be present. Using `@a`
on a phrase without `:a` or `:an` produces a runtime error, not a guess based on
phonetics. This prevents silent incorrect output (e.g., "a uniform" is correct
but heuristics would suggest "an uniform"; "an hour" is correct but heuristics
would suggest "a hour").

Similarly, `@the` in German requires `:masc`/`:fem`/`:neut`, `@count` in Chinese
requires a measure word tag like `:zhang`/`:ge`, etc. Always define phrases with
the tags required by the transforms that will be applied to them.

### Languages Without Special Transforms

Some languages (Russian, Polish, Ukrainian, Czech) have complex case systems but don't need special transforms—variant selection handles all the complexity. The Rust code selects the appropriate case+number variant.

### Classifier/Counter Languages

Chinese, Japanese, Korean, Vietnamese, Thai, and Bengali all use classifiers. The `@count` transform is shared across these languages but reads language-specific tags.

### Contraction Languages

French, Italian, and Portuguese all have preposition+article contractions. Each has its own transforms (`@de`, `@a`, `@em`, etc.) that handle the contraction rules.

### Vowel Harmony Languages

Turkish and other Turkic languages require vowel harmony in suffixes. Tags like `:front`/`:back` let transforms select the correct suffix form.
