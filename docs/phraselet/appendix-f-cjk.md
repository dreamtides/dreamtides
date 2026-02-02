# Appendix F: CJK Languages Guide (Japanese & Korean)

## Overview

Japanese and Korean share some characteristics with Chinese but have their own unique features:

- **Counters/Measure words** - like Chinese
- **Particles** - mark grammatical roles (subject, object, topic)
- **Honorific levels** - formal vs informal speech
- **SOV word order** - Subject-Object-Verb (unlike English SVO)

---

# Part 1: Japanese (日本語)

## Counters (助数詞)

Japanese uses counters between numbers and nouns, similar to Chinese:

| Counter | Reading | Used for |
|---------|---------|----------|
| 枚 | mai | flat objects (cards) |
| 体 | tai | bodies (characters, creatures) |
| 個 | ko | generic objects |
| 点 | ten | points, scores |
| 回 | kai | occurrences, times |

### Defining Counters

```rust
phraselet! {
    // Associate counters with nouns
    counter 枚 for [カード, イベント]     // flat objects
    counter 体 for [キャラクター, 味方, 敵]  // characters
    counter 点 for [スパーク, エネルギー]   // points

    // Nouns
    noun カード = "カード"
    noun キャラクター = "キャラクター"
    noun 味方 = "味方"
    noun 敵 = "敵"
}
```

### Using Counters

```rust
phraselet! {
    // {count:noun} syntax inserts the counter
    カードを引く(count: Int) = "{count:カード}を引く。"
    // count=1: "1枚を引く。" or "カードを1枚引く。"
    // count=3: "3枚を引く。"

    // Full form with noun stated
    カードを引く_full(count: Int) = "カードを{count:カード}引く。"
    // "カードを3枚引く。"
}
```

### Counter Reading Variations

Some counters change pronunciation based on the number:

```rust
phraselet! {
    // 枚 (mai) is regular
    // But some counters have irregular readings:
    // 1本 = ippon, 2本 = nihon, 3本 = sanbon

    // Phraselet handles this automatically for common counters
    counter_reading 枚 = {
        1: "いちまい", 2: "にまい", 3: "さんまい",
        // ... regular pattern
    }

    counter_reading 体 = {
        1: "いったい", 2: "にたい", 3: "さんたい",
        // ...
    }
}
```

## Particles (助詞)

Particles mark the grammatical role of words:

| Particle | Function | Example |
|----------|----------|---------|
| は (wa) | Topic marker | カードは... (As for the card...) |
| が (ga) | Subject marker | 敵が... (The enemy [does]...) |
| を (wo/o) | Object marker | カードを引く (draw a card) |
| に (ni) | Direction/location | 手札に戻す (return to hand) |
| で (de) | Means/location | ... |
| の (no) | Possessive | 相手の手札 (opponent's hand) |

### Using Particles

```rust
phraselet! {
    // Object particle を
    カードを引く(count: Int) = "カードを{count:カード}引く。"
    // "カードを3枚引く。" (Draw 3 cards)

    // Direction particle に
    手札に戻す(target: Predicate) = "{target}を手札に戻す。"
    // "味方を手札に戻す。" (Return ally to hand)

    // Possessive particle の
    相手の墓地 = "相手の墓地"
    // "opponent's void"

    // Topic particle は
    このキャラクターは = "このキャラクターは"
}
```

## Keywords

```rust
phraselet! {
    keyword 消滅 = "<k>消滅</k>"
    keyword 消滅させる = "<k>消滅させる</k>"
    keyword 追放 = "<k>追放</k>"
    keyword 予見(n: Int) = "<k>予見</k>{n}"
    keyword 燃焼(k: Int) = "<k>燃焼</k>{k}"
    keyword 回収 = "<k>回収</k>"
}
```

## Honorific Levels

Japanese has different politeness levels:

```rust
phraselet! {
    // Casual (dictionary form)
    引く_casual = "引く"     // draw

    // Polite (ます form)
    引く_polite = "引きます"  // draw (polite)

    // For card games, typically use dictionary form or ます form consistently
    // Choose one style for the game

    // Dictionary form (more common in games)
    効果_引く(count: Int) = "カードを{count:カード}引く。"

    // Polite form
    効果_引く_polite(count: Int) = "カードを{count:カード}引きます。"
}
```

## Word Order (SOV)

Japanese is Subject-Object-Verb:

```rust
phraselet! {
    // English: "Draw 3 cards"
    // Japanese: "カードを3枚引く" (Cards-OBJ 3-counter draw)

    // English: "This character gains +3 spark"
    // Japanese: "このキャラクターは+3スパークを得る"
    //           (This character-TOP +3 spark-OBJ gain)

    スパークを得る(target: Predicate, amount: Int) =
        "{target}は+{amount:スパーク}を得る。"
}
```

## Complete Examples

```rust
phraselet! {
    // Draw cards
    カードを引く(count: Int) = "カードを{count:カード}引く。"
    // "カードを3枚引く。"

    // Dissolve
    消滅させる(target: Predicate) = "{target}を{消滅させる}。"
    // "敵を消滅させる。"

    // Gain spark
    スパーク獲得(target: Predicate, amount: Int) =
        "{target}は+{amount:スパーク}を得る。"
    // "この味方は+3点を得る。"

    // Return to hand
    手札に戻す(target: Predicate) = "{target}を手札に戻す。"

    // Triggered abilities
    具現化時 = "▸ <b>具現化:</b>"
    消滅時 = "▸ <b>消滅:</b>"
    審判 = "▸ <b>審判:</b>"
}
```

---

# Part 2: Korean (한국어)

## Counters (수분류사)

Korean also uses counters:

| Counter | Used for |
|---------|----------|
| 장 | flat objects (cards) |
| 명/분 | people |
| 개 | generic objects |
| 점 | points |
| 번 | occurrences |

### Defining Counters

```rust
phraselet! {
    counter 장 for [카드, 이벤트]
    counter 개 for [캐릭터, 아군, 적]
    counter 점 for [스파크, 에너지]

    noun 카드 = "카드"
    noun 캐릭터 = "캐릭터"
    noun 아군 = "아군"
    noun 적 = "적"
}
```

## Particles

Korean particles change based on whether the preceding word ends in a consonant (받침/batchim) or vowel:

| Function | After consonant | After vowel |
|----------|-----------------|-------------|
| Subject | 이 (i) | 가 (ga) |
| Object | 을 (eul) | 를 (reul) |
| Topic | 은 (eun) | 는 (neun) |

### Particle Selection

```rust
phraselet! {
    // Particles that depend on final consonant
    particle subject = {
        consonant: "이",
        vowel: "가",
    }

    particle object = {
        consonant: "을",
        vowel: "를",
    }

    particle topic = {
        consonant: "은",
        vowel: "는",
    }

    // Usage - particle auto-selects based on preceding word
    카드를_뽑다(count: Int) = "카드{object.for(카드)} {count}장 뽑습니다."
    // 카드 ends in consonant ㄷ → "카드를"
    // "카드를 3장 뽑습니다."

    아군이_얻다 = "{아군}{subject.for(아군)} 얻습니다."
    // 아군 ends in consonant ㄴ → "아군이"
}
```

### Batchim Detection

```rust
phraselet! {
    // Words are marked with their final sound
    noun 카드(ends_consonant) = "카드"      // ends in ㄷ
    noun 아군(ends_consonant) = "아군"      // ends in ㄴ
    noun 스파크(ends_consonant) = "스파크"  // ends in ㅋ (ㄱ sound)

    // Some words end in vowel
    noun 마나(ends_vowel) = "마나"          // ends in 아

    // Then particle selection is automatic
    get_object_particle(word: Noun) = "{object.for(word)}"
}
```

## Honorific Levels

Korean has elaborate honorific levels:

| Level | Ending | Usage |
|-------|--------|-------|
| Formal polite | -ㅂ니다/-습니다 | Most common for games |
| Informal polite | -아요/-어요 | Friendly |
| Casual | dictionary form | Very informal |

```rust
phraselet! {
    // Formal polite (합니다/합니다) - typical for games
    카드를_뽑다(count: Int) = "카드를 {count}장 뽑습니다."
    // "카드를 3장 뽑습니다." (Draw 3 cards - formal polite)

    // Casual/informal (for different game tone)
    카드를_뽑다_casual(count: Int) = "카드를 {count}장 뽑아."
}
```

## Keywords

```rust
phraselet! {
    keyword 소멸 = "<k>소멸</k>"
    keyword 추방 = "<k>추방</k>"
    keyword 예견(n: Int) = "<k>예견</k> {n}"
    keyword 점화(k: Int) = "<k>점화</k> {k}"
    keyword 회수 = "<k>회수</k>"
}
```

## Word Order (SOV)

Korean is also SOV like Japanese:

```rust
phraselet! {
    // English: "Draw 3 cards"
    // Korean: "카드를 3장 뽑습니다" (Card-OBJ 3-counter draw)

    카드_뽑기(count: Int) = "카드를 {count:카드} 뽑습니다."

    // English: "This character gains +3 spark"
    // Korean: "이 캐릭터는 스파크를 +3점 얻습니다"

    스파크_얻기(target: Predicate, amount: Int) =
        "{target}{topic.for(target)} 스파크를 +{amount:스파크} 얻습니다."
}
```

## Native vs Sino-Korean Numbers

Korean has two number systems:

| Number | Native | Sino-Korean |
|--------|--------|-------------|
| 1 | 하나 (하나) | 일 (il) |
| 2 | 둘 (dul) | 이 (i) |
| 3 | 셋 (set) | 삼 (sam) |
| 4 | 넷 (net) | 사 (sa) |
| 5 | 다섯 (daseot) | 오 (o) |

Native numbers are used with counters up to about 99; Sino-Korean for larger numbers and formal contexts.

```rust
phraselet! {
    // Native numbers with counters
    // Note: 하나, 둘, 셋, 넷 shorten before counters:
    // 하나 → 한, 둘 → 두, 셋 → 세, 넷 → 네

    number_with_counter(n: Int, counter: Counter) = match n {
        1 => "한{counter}"      // 한 장
        2 => "두{counter}"      // 두 장
        3 => "세{counter}"      // 세 장
        4 => "네{counter}"      // 네 장
        _ => "{n}{counter}"     // 5장, 6장...
    }
}
```

## Complete Examples

```rust
phraselet! {
    // Draw cards
    카드_뽑기(count: Int) = "카드를 {count:카드} 뽑습니다."
    // "카드를 3장 뽑습니다."

    // Dissolve
    소멸시키다(target: Predicate) = "{target}{object.for(target)} {소멸}시킵니다."
    // "적을 소멸시킵니다."

    // Gain spark
    스파크_획득(target: Predicate, amount: Int) =
        "{target}{topic.for(target)} +{amount:스파크} 스파크를 얻습니다."
    // "이 아군은 +3점 스파크를 얻습니다."

    // Return to hand
    손으로_되돌리기(target: Predicate) =
        "{target}{object.for(target)} 손으로 되돌립니다."
    // "아군을 손으로 되돌립니다."

    // Triggered abilities
    구현화 = "▸ <b>구현화:</b>"
    소멸시 = "▸ <b>소멸:</b>"
    심판 = "▸ <b>심판:</b>"
}
```

---

## CJK Summary Table

| Feature | Chinese | Japanese | Korean |
|---------|---------|----------|--------|
| Counters/Measure words | ✓ | ✓ | ✓ |
| Particles | - | ✓ | ✓ (with variation) |
| Pluralization | - | - | - |
| Honorific levels | - | ✓ | ✓✓ |
| Word order | SVO-ish | SOV | SOV |
| Writing system | Hanzi | Hiragana/Katakana/Kanji | Hangul |

## Common Pitfalls

### Japanese
1. **Counter selection** - Use the right counter for the noun type
2. **Particle usage** - は vs が is nuanced; を for direct objects
3. **Politeness level** - Stay consistent throughout

### Korean
1. **Batchim-based particles** - 을/를, 이/가, 은/는 selection
2. **Number systems** - Native vs Sino-Korean in appropriate contexts
3. **Honorific consistency** - Choose a level and maintain it
