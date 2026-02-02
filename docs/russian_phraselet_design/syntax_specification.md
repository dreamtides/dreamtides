# Russian Phraselet Syntax Specification

## Overview

This document specifies the phraselet syntax for Russian localization of Dreamtides.
The syntax handles Russian's complex grammatical requirements:
- 6 grammatical cases
- 3 genders with animacy distinctions
- Complex numeric plural rules
- Agreement chains across adjectives, nouns, and verbs

## File Structure

```
localization/
  ru/
    russian_grammar.toml      # Grammar rules and plural patterns
    declensions.toml          # Word form tables
    phraselets_ru.toml        # Template definitions
```

## Syntax Elements

### 1. Simple Variable Substitution

```
{variable_name}
```

Inserts the value directly without modification.

**Example:**
```toml
template = "Урон: {значение}"
# Output: "Урон: 5"
```

### 2. Numeric Agreement with Nouns

```
{count_var|noun_key}
{count_var|noun_key:case}
```

Automatically selects the correct noun form based on Russian plural rules:
- 1 (except 11): nominative singular
- 2-4 (except 12-14): genitive singular
- 0, 5-20, 11-14: genitive plural

**Examples:**
```toml
{количество|карта:acc}

# количество=1  → "1 карту"
# количество=2  → "2 карты"
# количество=5  → "5 карт"
# количество=11 → "11 карт"
# количество=21 → "21 карту"
# количество=22 → "22 карты"
```

### 3. Agreement Chains

```
{chain:case|adj:adj_key|noun:noun_key}
{chain:case|adj:adj1|adj:adj2|noun:noun_key}
```

Creates a phrase where all adjectives agree with the noun in case, gender, number, and animacy.

**Examples:**
```toml
{chain:nom|adj:этот|noun:персонаж}
# Output: "этот персонаж"

{chain:acc|adj:этот|noun:персонаж}
# Output: "этого персонажа" (accusative animate masculine)

{chain:dat|adj:ваш|adj:самый_левый|noun:персонаж}
# Output: "вашему самому левому персонажу"
```

### 4. Verb Agreement

```
{verb:verb_key:conjugation}
{verb:verb_key:tense|subject_noun}
```

Conjugates verbs. For past tense, agrees with subject in gender/number.

**Conjugation keys:**
- `imp_sg`, `imp_pl` - imperative
- `pres_1sg`, `pres_2sg`, `pres_3sg`, `pres_1pl`, `pres_2pl`, `pres_3pl`
- `past_m`, `past_f`, `past_n`, `past_pl`
- `fut_*` - future (same pattern as present for perfective)

**Examples:**
```toml
# Imperative (commands on cards)
{verb:взять:imp_pl}
# Output: "Возьмите"

# Past tense with agreement
{noun:персонаж:nom} {verb:получить:past|noun:персонаж}
# Output: "персонаж получил" (masculine agreement)

{noun:карта:nom} {verb:получить:past|noun:карта}
# Output: "карта получила" (feminine agreement)
```

### 5. Phraselet References

```
{@phraselet_id}
{@phraselet_id param1=value1 param2=value2}
```

Embeds another phraselet, optionally passing parameters.

**Example:**
```toml
[phraselet.when_materialized]
template = "Когда {chain:nom|adj:этот|noun:персонаж} материализован"

[phraselet.kindle_on_materialize]
template = "{@when_materialized}, {@kindle_description количество=2}"
# Output: "Когда этот персонаж материализован, добавьте 2 искры вашему самому левому персонажу."
```

### 6. Conditional Forms

```
{if:condition|true_value|false_value}
```

Selects text based on a condition.

**Conditions:**
- `var=value` - equality check
- `var>value`, `var<value`, `var>=value`, `var<=value` - comparisons
- `var` - truthy check (non-zero, non-empty)

**Example:**
```toml
{if:количество=1|карту|{количество|карта:acc}}
# количество=1 → "карту"
# количество=3 → "3 карты"
```

### 7. Explicit Case/Gender Override

```
{noun:noun_key:case:number}
{adj:adj_key:case:gender:number:animacy}
```

Forces specific grammatical forms when automatic agreement doesn't apply.

**Example:**
```toml
# Force genitive plural regardless of count
с {noun:карта:gen:pl}
# Output: "с карт"
```

## Declension Table Format

### Nouns

```toml
[noun.карта]
gender = "f"           # m/f/n
animacy = "inanim"     # anim/inanim
nom_sg = "карта"
gen_sg = "карты"
dat_sg = "карте"
acc_sg = "карту"
inst_sg = "картой"
prep_sg = "карте"
nom_pl = "карты"
gen_pl = "карт"
dat_pl = "картам"
acc_pl = "карты"
inst_pl = "картами"
prep_pl = "картах"
```

### Adjectives

Adjectives need forms for all gender/number/case/animacy combinations:

```toml
[adj.этот]
# Masculine singular
nom_m_sg = "этот"
gen_m_sg = "этого"
dat_m_sg = "этому"
acc_m_sg_anim = "этого"    # animate accusative = genitive
acc_m_sg_inanim = "этот"   # inanimate accusative = nominative
inst_m_sg = "этим"
prep_m_sg = "этом"

# Feminine singular
nom_f_sg = "эта"
gen_f_sg = "этой"
# ... etc

# Neuter singular
nom_n_sg = "это"
# ... etc

# Plural (all genders)
nom_pl = "эти"
gen_pl = "этих"
dat_pl = "этим"
acc_pl_anim = "этих"
acc_pl_inanim = "эти"
inst_pl = "этими"
prep_pl = "этих"
```

### Verbs

```toml
[verb.получить]
infinitive = "получить"
aspect = "perfective"

# Imperative
imp_sg = "получи"
imp_pl = "получите"

# Past (gender agreement)
past_m = "получил"
past_f = "получила"
past_n = "получило"
past_pl = "получили"

# Future (perfective has no present)
fut_1sg = "получу"
fut_2sg = "получишь"
fut_3sg = "получит"
fut_1pl = "получим"
fut_2pl = "получите"
fut_3pl = "получат"
```

## Russian Plural Rules

The system implements standard Russian numeric agreement:

| Number ending | Form used | Example |
|--------------|-----------|---------|
| 1 (not 11) | nom_sg | 1 карта, 21 карта, 101 карта |
| 2-4 (not 12-14) | gen_sg | 2 карты, 23 карты, 102 карты |
| 0, 5-20 | gen_pl | 5 карт, 11 карт, 20 карт |
| 11-14 | gen_pl | 11 карт, 12 карт, 14 карт |

Implementation in `russian_grammar.toml`:

```toml
[[grammar.plural_rules.patterns]]
range = { mod100 = [11, 12, 13, 14] }
form = "gen_pl"

[[grammar.plural_rules.patterns]]
range = { mod10 = [1] }
form = "nom_sg"

[[grammar.plural_rules.patterns]]
range = { mod10 = [2, 3, 4] }
form = "gen_sg"

[[grammar.plural_rules.patterns]]
range = { default = true }
form = "gen_pl"
```

## Complete Examples

### Example 1: "Draw a card"

```toml
[phraselet.draw_cards]
params = ["количество"]
template = "Возьмите {количество|карта:acc}."
```

| Input | Output |
|-------|--------|
| количество=1 | Возьмите 1 карту. |
| количество=2 | Возьмите 2 карты. |
| количество=5 | Возьмите 5 карт. |
| количество=21 | Возьмите 21 карту. |

### Example 2: "This character gains 2 spark"

```toml
[phraselet.gain_spark]
params = ["количество"]
template = "{chain:nom|adj:этот|noun:персонаж} получает {количество|искра:acc}."
```

Output: "Этот персонаж получает 2 искры."

### Example 3: "An ally with spark 3 or more"

```toml
[phraselet.ally_with_spark_acc]
params = ["минимум"]
template = "союзника с искрой {минимум} или больше"
```

Output: "союзника с искрой 3 или больше"

Note: "искрой" is instrumental case (с + instrumental = "with").

### Example 4: Complex triggered ability

```toml
[phraselet.example_trigger]
template = """
{@when_materialized}, развейте {chain:acc|adj:целевой|noun:персонаж}
с искрой меньше искры {@this_card:gen}.
"""
```

Output: "Когда этот персонаж материализован, развейте целевого персонажа с искрой меньше искры этой карты."

## Design Principles

1. **Cyrillic-native**: All keys and text use Cyrillic naturally
2. **Explicit over implicit**: Cases are marked explicitly to avoid ambiguity
3. **Composable**: Phraselets reference each other for complex sentences
4. **Complete declension**: All word forms are pre-defined, no runtime generation
5. **Translator-friendly**: Russian speakers can read and modify templates directly

## Error Handling

The system should provide clear errors for:
- Missing declension forms
- Invalid case/gender combinations
- Undefined phraselet references
- Type mismatches (e.g., non-numeric value for count)

Example error:
```
Error in phraselet 'gain_spark':
  Noun 'искра' missing form 'acc_sg'
  at template position: {количество|искра:acc}
```
