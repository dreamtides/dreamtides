# Japanese (日本語) i18n Review of Serializer RLF Migration Plan

**Reviewer:** ja-agent (Japanese localization specialist)
**Date:** 2026-02-08
**Status:** PASS with recommendations

---

## Executive Summary

The migration plan is **well-designed for Japanese support**. Japanese shares many CJK characteristics with Chinese (counters, no plural, no gender, no articles), and the Chinese review already validated the core CJK patterns. This review focuses on **Japanese-specific concerns not covered by the Chinese review**.

There are **2 medium-priority issues** and **3 minor recommendations**. None are blockers. The most significant finding is that the `@count:word` modifier proposed for Chinese (Section 9.8.4) needs slightly different behavior for Japanese, and the `@particle` transform documented in APPENDIX_STDLIB.md needs verification.

---

## Analysis by Concern Area

### 1. Counters (助数詞) — Different from Chinese Classifiers

**Verdict: SUPPORTED, with `@count:word` nuance**

Japanese counters (助数詞) are similar to Chinese classifiers but use different characters and have different counter-number mappings:

| Concept | Chinese | Japanese | Counter |
|---------|---------|----------|---------|
| Cards (flat) | 张 (zhāng) | 枚 (まい) | `:zhang` / `:mai` |
| Characters | 个 (gè) | 体 (たい) | `:ge` / `:ko` or custom |
| People | 名 (míng) | 人 (にん) | `:ming` / `:nin` |
| General | 个 (gè) | 個 (こ) | `:ge` / `:ko` |

The APPENDIX_STDLIB.md already defines Japanese counter tags (`:mai`, `:nin`, `:hiki`, `:hon`, `:ko`, `:satsu`). The `@count` transform reads these tags. **This works.**

**Key difference from Chinese — Counter for game characters:**

Japanese card games typically use 体 (たい) for non-human characters/creatures and 人 (にん) for human-like characters. The stdlib defines `:nin` for people but not `:tai` for creatures/bodies. Dreamtides "characters" may need a `:tai` tag.

**RECOMMENDATION J1:** Add `:tai` (体) to the Japanese counter tag set in APPENDIX_STDLIB.md. This is the standard counter for dolls, robots, statues, and game characters/creatures. Many TCGs use 体 for summoned creatures.

**`@count:word` for Japanese:** The Chinese review proposed `@count:word` to produce number words (一/两/三) instead of digits. Japanese needs the same feature but uses different number words:

| n | Chinese | Japanese |
|---|---------|----------|
| 1 | 一 | 一 |
| 2 | 两 (liǎng) | 二 |
| 3 | 三 | 三 |
| 4 | 四 | 四 |
| 5 | 五 | 五 |
| 10 | 十 | 十 |

**Critical difference:** Chinese uses 两 (liǎng) for counting "two of something", not 二 (èr). Japanese always uses 二 (ni) — there is no special counting form for two. The `@count:word` implementation in Section 9.8.4 correctly notes "~30 lines per CJK language" — each language gets its own mapping. Japanese mapping is simpler than Chinese (no 两/二 distinction).

**Japanese counter pronunciation irregularities:** Japanese counters have irregular sound changes (音便) that `@count` should handle:
- 一枚 (いちまい) — regular
- 三枚 (さんまい) — regular
- But: 一本 (い**っぽん**), 三本 (さん**ぼん**), 六本 (ろ**っぽん**) — rendaku/gemination

However, since RLF renders text (not audio), and written forms use the same kanji regardless of pronunciation, this is NOT a framework issue. "1本", "3本" display correctly. **No action needed.**

---

### 2. Particles (助詞) — Postpositions

**Verdict: SUPPORTED via `@particle` transform**

Japanese particles are postpositions attached after nouns: カードを (card-OBJ), カードが (card-SUBJ). The APPENDIX_STDLIB.md documents `@particle` for Japanese with contexts `:subj` (が), `:obj` (を), `:topic` (は), `:loc` (に), `:place` (で), `:dir` (へ), `:from` (から), `:until` (まで).

**How this works in practice:**
```
// ja.rlf
card = :mai "カード";
draw($n) = "{@count($n) card}を引く";  // "3枚カードを引く" — draw 3 cards
```

Unlike Korean `@particle` (which inspects final sound to choose particle form), Japanese particles are invariant — が is always が regardless of what precedes it. The `@particle` transform simply appends the particle. This is simpler than Korean's phonology-based selection.

**CONCERN J2 — Is `@particle` necessary at all?** Since Japanese particles don't change form based on the preceding word, translators could simply write the particle directly in the template string:

```
// These are equivalent:
draw($n) = "{@count($n) card}を引く";           // with @particle
draw($n) = "{@count($n) card}{@particle:obj card}を引く"; // verbose, redundant
```

Wait — looking at the stdlib example more carefully:
```
draw($n) = "{@count($n) card}を引く";  // n=3 → "3枚カードを引く"
```

The particle `を` is written directly in the template, not via `@particle`. The `@particle` transform seems designed for cases where the particle choice depends on the phrase structure, but in practice, Japanese card text uses fixed particles for each phrase. **The `@particle` transform is a convenience, not a necessity.** Translators can write particles directly. This is fine.

**CONCERN — Particle placement relative to `@count` output:** When `@count($n)` produces "3枚", the output is "3枚カード" (number-counter-noun). The particle を attaches after the noun: "3枚カードを引く". This is correct for the "Number + Counter + Noun + Particle + Verb" pattern. However, Japanese also allows "カードを3枚引く" (Noun + Particle + Number + Counter + Verb). Both orderings are natural. The phrase template controls this:

```
// Pattern A: Number-Counter-Noun-Particle-Verb
draw_a($n) = "{@count($n) card}を引く";     // "3枚カードを引く"

// Pattern B: Noun-Particle-Number-Counter-Verb
draw_b($n) = "カードを{@count($n) card}引く"; // "カードを3枚引く"
```

Wait — pattern B has a problem. `@count($n) card` produces "3枚カード" (with the noun), but we only want "3枚" (just the counter). The noun "カード" is already written separately. This would produce "カードを3枚カード引く" (double noun).

**RECOMMENDATION J2:** Verify that `@count` can produce just "number+counter" without the noun text. If `@count($n) card` always produces "{number}{counter}{noun}", then Pattern B isn't directly expressible. Translators would need to bypass `@count` and use `:match` for this ordering:

```
draw($n) = :match($n) {
    1: "カードを一枚引く",
    *other: "カードを{$n}枚引く",
};
```

This is a minor concern — Pattern A is perfectly natural for card text and most TCGs use it. But documenting this limitation for Japanese translators would be helpful.

---

### 3. SOV Word Order

**Verdict: FULLY SUPPORTED**

Japanese is strictly SOV (Subject-Object-Verb). The verb always comes last:
- "カードを3枚引く" (card-OBJ 3-counter draw) — Draw 3 cards
- "敵のキャラクターを消滅させる" (enemy-GEN character-OBJ dissolve) — Dissolve an enemy character

Each phrase template defines its own word order. The RLF framework doesn't impose English SVO order. **No issues.**

The top-level string concatenation (Section 2.3) concatenates trigger + effect. In Japanese:
- "カードをプレイした時、カードを1枚引く。"
- (card-OBJ play-PAST when, card-OBJ 1-counter draw.)

Trigger-before-effect ordering works in Japanese (same as Chinese). The trigger phrase includes its own connective particle "時、" (toki, "when"). **No issues.**

---

### 4. No Articles — `@a`/`@the` Irrelevance

**Verdict: FULLY SUPPORTED**

Japanese has no articles. The English `@a`/`@the` transforms are English-specific and are not invoked in Japanese translation files. The `:a`/`:an` tags on English source terms are ignored when running in the Japanese locale. **No issues.**

Where English says "Draw **a** card", Japanese says "カードを1枚引く" (draw 1 card) or just "カードを引く" (draw cards). The article concept doesn't exist.

---

### 5. Katakana for Game Keywords

**Verdict: FULLY SUPPORTED**

Japanese card games may use either:
- Katakana loan words: ディゾルブ (dissolve), マテリアライズ (materialize)
- Japanese equivalents: 消滅 (dissolve/destroy), 実体化 (materialize)
- Mixed: カタカナ keyword with Japanese explanation

The choice is a translation decision, not a framework concern. RLF keyword terms are simply redefined in the Japanese translation file:

```
// ja.rlf
dissolve = "<color=#AA00FF>消滅</color>";
// OR: dissolve = "<color=#AA00FF>ディゾルブ</color>";
materialize = "<color=#AA00FF>実体化</color>";
banish = "<color=#AA00FF>追放</color>";
reclaim = "<color=#AA00FF>回収</color>";
prevent = "<color=#AA00FF>打消</color>";
kindle($k) = "<color=#AA00FF>灯火</color> {$k}";
foresee($n) = "<color=#AA00FF>予見</color> {$n}";
```

The `@cap` transform is a no-op on CJK characters (Section 9.7 checklist item). Whether using katakana or kanji, capitalization has no effect. **No issues.**

---

### 6. Politeness/Formality (敬語)

**Verdict: NOT A FRAMEWORK ISSUE**

Japanese card text universally uses plain form (辞書形/命令形), not polite form (です/ます). This is a translation convention, not a framework concern:
- "カードを引く" (plain) — correct for card text
- "カードを引きます" (polite) — wrong for card text

No framework changes needed. Translators simply use plain form verbs.

---

### 7. Counter + Number Ordering

**Verdict: SUPPORTED (see Section 2 for Pattern B limitation)**

Japanese has two natural orderings for counted nouns:
- "3枚のカード" or "3枚カード" (3-counter card) — number first
- "カード3枚" (card 3-counter) — noun first

Both are common. The phrase template controls which ordering is used:
```
// Number first (more common in card text):
cards($n) = "{@count($n) card}";     // "3枚カード"

// Noun first (also natural):
cards_alt($n) = "カード{$n}枚";       // "カード3枚" — requires manual counter
```

As noted in Section 2, the second pattern may require bypassing `@count` since it embeds the noun. **Acceptable — Pattern A is standard in TCGs.**

---

### 8. `@count:word` for Japanese

**Verdict: SUPPORTED, Chinese proposal works for Japanese**

The Chinese review's `@count:word` proposal (Section 9.8.4) works for Japanese with these number words:

| n | Japanese |
|---|---------|
| 1 | 一 |
| 2 | 二 |
| 3 | 三 |
| 4 | 四 |
| 5 | 五 |
| 6 | 六 |
| 7 | 七 |
| 8 | 八 |
| 9 | 九 |
| 10 | 十 |
| >10 | digit form |

Note: Japanese has NO equivalent of Chinese 两/二 distinction. The number 二 is always used. The implementation is straightforward — ~30 lines mapping integers to Japanese kanji numbers.

**Usage in Japanese:**
```
// ja.rlf
cards($n) = "{@count:word($n) card}";
draw_cards_effect($c) = "{cards($c)}を引く。";
// draw_cards_effect(1) → "一枚カードを引く。"
// draw_cards_effect(3) → "三枚カードを引く。"
```

---

## Issue Summary

### Medium Priority

| ID | Issue | Section | Impact | Recommendation |
|----|-------|---------|--------|----------------|
| J1 | Add `:tai` (体) counter tag for creatures/characters | 1 | Missing counter for a common noun category in TCGs | Add to APPENDIX_STDLIB.md Japanese counter tags |
| J2 | `@count` noun-included output prevents Noun+Counter ordering | 2 | Limits Japanese word order options | Document limitation; verify `@count` behavior |

### Minor / Informational

| ID | Note | Section |
|----|------|---------|
| I1 | `@particle` is optional — translators can write particles directly in templates | 2 |
| I2 | Counter pronunciation irregularities (音便) are irrelevant for text display | 1 |
| I3 | Politeness level is a translation convention, not a framework concern | 6 |

---

## RLF Framework Changes for Japanese

### Items Already Covered by Chinese Review

The following items from the Chinese review apply equally to Japanese and need no additional discussion:

- **`@count:word` modifier (Section 9.8.4)** — Japanese needs this too, with its own mapping. Already proposed.
- **`:from` tag propagation** — Works the same way. Confirmed correct by Chinese review.
- **`@cap` no-op on CJK** — Japanese katakana/kanji have no case. Already verified.
- **Classifiers as tags, not first-class** — Japanese counters work the same way as Chinese classifiers under the tag + `@count` pattern.
- **Full-width punctuation** — Japanese uses 。(period), 、(comma), ：(colon). Handled via RLF phrase redefinition, same as Chinese.

### Recommendation 1: Add `:tai` Counter Tag

**Problem:** The APPENDIX_STDLIB.md Japanese counter tags include `:mai` (枚, flat objects), `:nin` (人, people), `:hiki` (匹, small animals), `:hon` (本, long objects), `:ko` (個, general), `:satsu` (冊, books). Missing is `:tai` (体, bodies/creatures/characters/robots), which is the standard counter for game characters and summoned creatures in Japanese TCGs.

**Proposed:** Add to the Japanese metadata tags table:

```
| `:tai` | Bodies, characters, robots | 体 |
```

**Implementation:** Add the tag to the Japanese `@count` transform's counter map. ~2 lines.

**Priority: MEDIUM.** Without this, Japanese translators must hardcode 体 in every character-counting phrase instead of using `@count`.

### Recommendation 2: Document `@count` Noun Inclusion Behavior

**Problem:** `@count($n) card` in the stdlib example produces "3枚カード" (number + counter + noun). Japanese sometimes prefers "カードを3枚" (noun + particle + number + counter) — but this requires the count WITHOUT the noun. It's unclear whether `@count` always includes the noun text or can produce just "3枚".

**Proposed:** Add a documentation note to APPENDIX_STDLIB.md:

> **Note for Japanese/CJK translators:** `@count($n) term` produces "{number}{counter}{noun}". To use the alternative "noun + counter" ordering (e.g., "カードを3枚"), bypass `@count` and write the counter directly in a `:match` block, or place `@count` output before the particle.

**Priority: LOW.** The "{number}{counter}{noun}" ordering is the dominant pattern in Japanese TCG card text. The alternative ordering is nice-to-have but not critical.

### Recommendation 3: Verify `@particle` Transform Implementation

**Problem:** The APPENDIX_STDLIB.md documents `@particle` for Japanese with 8 particle contexts (`:subj`, `:obj`, `:topic`, `:loc`, `:place`, `:dir`, `:from`, `:until`). This needs to be verified as implemented. Unlike Korean `@particle` (which must inspect final sounds), Japanese `@particle` is a simple context-to-string lookup.

**Proposed:** Verify the Japanese `@particle` transform exists in the RLF crate. If not implemented yet, it's ~15 lines:

```rust
fn japanese_particle(context: &str) -> &str {
    match context {
        "subj" => "が",
        "obj" => "を",
        "topic" => "は",
        "loc" => "に",
        "place" => "で",
        "dir" => "へ",
        "from" => "から",
        "until" => "まで",
        _ => "が",  // default to subject
    }
}
```

**Priority: LOW.** Japanese translators can write particles directly in templates. `@particle` is purely a convenience.

---

## Detailed Test Cases for Japanese

### "Draw 3 cards." → "カードを3枚引く。"

```
// ja.rlf
card = :mai "カード";
cards($n) = :match($n) {
    1: "カードを一枚",
    *other: "カードを{$n}枚",
};
draw_cards_effect($c) = "{cards($c)}引く。";
// draw_cards_effect(1) → "カードを一枚引く。" ✓
// draw_cards_effect(3) → "カードを3枚引く。" ✓
```

### "Dissolve an enemy Ancient." → "敵の＜エンシェント＞を消滅させる。"

```
// ja.rlf
ancient = :ko "エンシェント";
subtype($s) = :from($s) "<color=#2E7D32><b>{$s}</b></color>";
enemy_subtype($t) = :from($t) "敵の{$t}";

dissolve_target_effect($target) = "{$target}を消滅させる。";
// With enemy Ancient → "敵の<b>エンシェント</b>を消滅させる。" ✓
```

Note: No article, no counter needed for single-target dissolve. The object particle を marks the target.

### "When you play a card, draw a card." → "カードをプレイした時、カードを一枚引く。"

```
// ja.rlf
when_you_play_trigger($target) = "{$target}をプレイした時、";
draw_cards_effect($c) = "{cards($c)}引く。";
// "カードをプレイした時、" + "カードを一枚引く。"
// → "カードをプレイした時、カードを一枚引く。" ✓
```

### Activated ability: "Discard a card: Draw 2 cards." → "カードを一枚捨てる：カードを2枚引く。"

```
// ja.rlf
discard_cards_cost($d) = "{cards($d)}捨てる";
cost_effect_separator = "：";     // Full-width colon
draw_cards_effect($c) = "{cards($c)}引く。";
// "カードを一枚捨てる" + "：" + "カードを2枚引く。"
// → "カードを一枚捨てる：カードを2枚引く。" ✓
```

---

## Conclusion

The migration plan is **sound for Japanese**. The architecture handles Japanese's core requirements:

1. **Counters via `@count` + tags** — same pattern as Chinese, different counter characters
2. **No plural/gender/articles** — irrelevant features are no-ops or ignored
3. **SOV word order** — phrase templates control ordering freely
4. **Particles** — written directly in templates or via `@particle` convenience transform
5. **`@cap` no-op on CJK** — safe for katakana and kanji
6. **Full-width punctuation** — RLF phrase redefinition handles this
7. **`@count:word`** — Chinese proposal works for Japanese with its own number mapping

The Rust code after Phase 2 will be fully language-neutral for Japanese. No Rust code changes will be needed — only Japanese `.rlf` translation files.

The only net-new recommendation compared to the Chinese review is adding the `:tai` counter tag for game characters, which is a ~2 line addition to the RLF stdlib.

---

## Summary of Japanese-Specific RLF Recommendations

| # | Change | Type | Priority | Effort |
|---|--------|------|----------|--------|
| 1 | Add `:tai` (体) counter tag for characters/creatures | Feature | MEDIUM | ~2 lines |
| 2 | Document `@count` noun-inclusion behavior for CJK word order | Docs | LOW | ~5 lines |
| 3 | Verify `@particle` transform implementation | Verification | LOW | ~15 lines if missing |
