# Design Agent 5: Symbol-Rich Architectures

## Key Takeaways

- **The current 1-2 symbol norm wastes the symbol system's information
  capacity.** With 4 resonance types and only 1-2 symbols per card, the
  algorithm cannot distinguish between the 2 archetypes sharing a primary
  resonance. Moving to 3 symbols per card with repetition allowed expands the
  signal space from ~8 meaningful patterns to ~40, enabling archetype-level (not
  just resonance-level) targeting.
- **Repetition is the key unlock, not symbol count alone.** Research Agent A
  showed that 3 distinct symbols from 4 types yields only 4 possible sets
  (identified by the missing type) -- strictly worse for filtering. But (Tide,
  Tide, Zephyr) uniquely identifies Warriors at ~95% precision vs. 50% for
  single-resonance filtering.
- **Symbol-rich filtering creates a precision/pool-size tradeoff.** Triple-match
  filtering (e.g., Tide-Tide-Zephyr) yields ~8 cards per archetype -- too few
  for bulk slot-filling but ideal for 1 high-confidence slot per pack combined
  with broader R1 filtering for remaining slots.
- **Symbol richness partially decouples algorithm performance from fitness.**
  Pair-matching achieves 85% S/A precision under Pessimistic fitness (Research
  Agent A), compared to 62.5% for R1 filtering. Triple-match filtering could
  reach ~90-95%, making the algorithm largely immune to cross-archetype fitness
  problems.
- **The card design cost is moderate but real.** Every non-generic card needs 3
  symbols with meaningful repetition patterns. The designer assigns symbols to
  encode archetype identity, not arbitrary decoration. This is a one-time
  symbol-assignment effort, distinct from the harder problem of mechanical
  cross-archetype fitness.
- **Combining symbol-rich precision with Surge delivery addresses both M3 and
  M10.** High-precision slots reduce variance between packs (fewer "wrong
  archetype" draws), smoothing delivery without sacrificing average quality.

______________________________________________________________________

## Five Algorithm Proposals

### Proposal 1: Triple-Signal Surge

**One sentence:** Surge packs fill 1 slot via archetype-triple matching (e.g.,
Tide-Tide-Zephyr for Warriors) and 2 slots via R1 matching, with 1 random; floor
packs use 1 triple-matched slot and 3 random.

**Technical description:** Maintain token counters as in V7 Surge+Floor. When a
surge fires on resonance R, determine the player's likely archetype by examining
the secondary counter: if R=Tide and Zephyr is highest secondary, the player is
Warriors, so the archetype triple is (Tide, Tide, Zephyr). Fill 1 surge slot
from cards matching this triple (pool ~8 cards, ~95% S-tier), fill 2 slots from
the R1 pool (pool ~80, ~50% home-archetype), fill 1 random. Floor packs: 1
triple-matched slot, 3 random.

**Predicted behavior:**

| Fitness                    | M3 (est.) | Notes                                                |
| -------------------------- | :-------: | ---------------------------------------------------- |
| Optimistic                 |   ~2.8    | Triple slot near-guaranteed S-tier; R1 slots all S/A |
| Moderate (36% weighted)    |   ~2.1    | Triple slot ~95% S-tier; R1 slots ~68% S/A           |
| Pessimistic (21% weighted) |   ~1.7    | Triple slot ~90% S-tier; R1 slots ~60% S/A           |

**Pool requirement:** 40% dual-resonance, all non-generics carry 3 symbols with
repetition. ~8 cards per archetype triple.

### Proposal 2: Weighted Symbol Accumulation (WSA)

**One sentence:** Cards with repeated symbols earn bonus tokens
(Tide-Tide-Zephyr earns +3 Tide, +1 Zephyr instead of +2/+1/+1), accelerating
surges for committed drafters while keeping the V7 Surge+Floor delivery
mechanism.

**Technical description:** Token earning scales with symbol repetition: each
occurrence of a resonance earns +1, plus +1 bonus for the leftmost (primary)
position. A (Tide, Tide, Zephyr) card earns Tide: 2+1=3, Zephyr: 1. A (Tide,
Zephyr, Ember) card earns Tide: 1+1=2, Zephyr: 1, Ember: 1. Surge threshold
remains T=3, but committed players hit surges faster (~every 1.0-1.2 picks vs.
1.5-2.0). Surge and floor pack composition identical to V7 Surge+Floor.

**Predicted behavior:**

| Fitness     | M3 (est.) | Notes                                                          |
| ----------- | :-------: | -------------------------------------------------------------- |
| Optimistic  |   ~2.9    | More frequent surges; ~70% of post-commitment packs are surges |
| Moderate    |   ~2.0    | Surge frequency compensates for fitness loss                   |
| Pessimistic |   ~1.5    | Improvement over V7 but still below 2.0                        |

**Pool requirement:** All non-generics carry 3 symbols. No change to
dual-resonance cap needed; the benefit comes from faster accumulation, not
better filtering.

### Proposal 3: Archetype Fingerprint Matching

**One sentence:** Each card's 3-symbol sequence is its "fingerprint"; the
algorithm matches pack slots against the player's accumulated fingerprint
profile, selecting cards whose symbol pattern is closest to the player's
drafting pattern.

**Technical description:** Track a 4-dimensional resonance profile vector (count
of each resonance across all drafted cards). For each pack slot designated as
"targeted," score candidate cards by cosine similarity between the card's symbol
vector and the player's profile vector. Select the highest-similarity card. This
naturally prefers cards with the same repetition pattern as the player's
archetype. A Warriors player (heavy Tide, moderate Zephyr) naturally draws
(Tide, Tide, Zephyr) cards over (Tide, Stone, Ember) cards. Use 2
fingerprint-matched slots + 2 random on every post-commitment pack (no
surge/floor distinction).

**Predicted behavior:**

| Fitness     | M3 (est.) | Notes                                                      |
| ----------- | :-------: | ---------------------------------------------------------- |
| Optimistic  |   ~2.5    | High precision per slot; smooth delivery (no surge spikes) |
| Moderate    |   ~1.9    | Fingerprint matching achieves ~80-85% S/A precision        |
| Pessimistic |   ~1.5    | Precision degrades but remains above R1 filtering          |

**Pool requirement:** All non-generics carry 3 symbols with meaningful
repetition. Moderate dual-resonance cap (30%+).

### Proposal 4: Layered Symbol Filtering (LSF)

**One sentence:** Each targeted slot applies progressively tighter symbol
filters -- the first slot matches R1 only (broad pool), the second matches the
R1 pair (medium pool), the third matches the full triple (narrow pool) --
adapting precision to available pool depth.

**Technical description:** On surge packs, fill 3 targeted slots with layered
filtering: Slot A draws from the full R1 pool (~80 cards, 50% home-archetype).
Slot B draws from the R1+R2 pair pool (~18-24 cards at 40% dual-res, ~80%
home-archetype). Slot C draws from the full triple pool (~8 cards, ~95%
home-archetype). Slot D is random. On floor packs: 1 slot uses R1+R2 pair
filtering, 3 random. If any filtered subpool is exhausted or below minimum size
(5 cards), fall back to the next-broader filter.

**Predicted behavior:**

| Fitness     | M3 (est.) | Notes                                                       |
| ----------- | :-------: | ----------------------------------------------------------- |
| Optimistic  |   ~2.8    | Layered precision yields high S/A across all slots          |
| Moderate    |   ~2.15   | Pair+triple slots maintain ~85% S/A even at low fitness     |
| Pessimistic |   ~1.75   | Triple slot still ~90% S-tier; pair slot ~85%; R1 slot ~60% |

**Pool requirement:** 40% dual-resonance cap, all non-generics carry 3 symbols
with repetition.

### Proposal 5: Symbol-Weighted Surge + Pair Floor (Champion Candidate)

**One sentence:** Surges fill 3 slots from the archetype pair-filtered pool
(R1+R2 matching, ~80% home-archetype) instead of R1-only, and floor packs
guarantee 1 pair-filtered slot; all non-generic cards carry 3 symbols enabling
pair identification across 40% of the pool.

**Technical description:** Token earning uses symbol repetition bonuses (as in
Proposal 2). Surge threshold T=3. On surge, identify the player's archetype pair
from top two counters (e.g., Tide primary + Zephyr secondary = Warriors). Fill 3
slots from cards whose first two symbols match this pair (~18-24 cards at 40%
dual-res). Fill 1 slot random. On floor packs (from pick 3+): 1 slot
pair-filtered, 3 random. Fallback: if pair-filtered subpool has fewer than 5
remaining unseen cards, broaden to R1 filtering for that slot.

**Predicted behavior:**

| Fitness                    | M3 (est.) | Notes                                                              |
| -------------------------- | :-------: | ------------------------------------------------------------------ |
| Optimistic                 |   ~2.9    | Pair-filtered surges yield ~95% S/A per slot                       |
| Moderate (36% weighted)    |   ~2.2    | Pair precision (~85%) dominates R1 precision (~68%)                |
| Pessimistic (21% weighted) |   ~1.8    | Pair-matched cards 80% home-archetype; partially immune to fitness |
| Hostile (8%)               |   ~1.5    | Still functional due to pair precision bypassing fitness           |

**Pool requirement:** 40% dual-resonance, all non-generics carry 3 symbols,
minimum 18 pair-matched cards per archetype.

______________________________________________________________________

## Champion Selection: Proposal 5, Symbol-Weighted Surge + Pair Floor

**Justification:** Proposal 5 combines the three strongest findings from V3-V8
research into one system:

1. **Surge delivery** (V7's best mechanism class) for slot concentration and
   natural variance.
2. **Pair filtering** (V5's key insight, validated by Research Agent A) for
   fitness-robust precision at 85% vs. R1's 62.5% under Pessimistic fitness.
3. **Symbol-rich accumulation** (this agent's contribution) for faster
   convergence and archetype disambiguation.

Proposal 1 (Triple-Signal) achieves higher per-slot precision but its ~8-card
subpool creates repetition problems over 25+ post-commitment packs. Proposal 3
(Fingerprint) delivers smooth quality but sacrifices variance (M9) by
eliminating surge/floor alternation. Proposal 4 (Layered) is mechanistically
elegant but its 3 different filter levels per pack adds implementation
complexity without clear M3 gains over Proposal 5's uniform pair filtering.
Proposal 2 (WSA) improves accumulation but does not address the core precision
problem.

Proposal 5 hits the critical sweet spot: pair-filtered subpools of ~18-24 cards
are large enough to sustain 3 draws per surge pack across 25+ packs (~75 draws
from an 18-24 card pool = each card seen ~3-4 times, within acceptable
repetition), while achieving ~85% S/A precision under Pessimistic fitness.

______________________________________________________________________

## Champion Deep-Dive

### Example Draft: Warriors (Tide/Zephyr) Player, Graduated Realistic Fitness

**Picks 1-2:** Fully random packs. Player sees diverse resonances, takes a
strong Tide character (Tide, Tide, Zephyr symbols) and a generic removal spell
(0 symbols). Tide counter: 3, Zephyr counter: 1.

**Pick 3 (Surge!):** Counter hit T=3. Archetype pair = (Tide, Zephyr). Surge
pack: 3 cards from (Tide, Zephyr) pair pool + 1 random. Player sees 2 Warriors
cards (S-tier), 1 Ramp card that happens to be A-tier in Warriors, 1
off-archetype random. Takes the best Warriors card. Tide counter resets; Zephyr
grows.

**Picks 4-5 (Floor):** 1 pair-filtered slot (Tide, Zephyr card), 3 random.
Player continues taking Tide/Zephyr cards, building counters. Each floor pack
has 1 guaranteed relevant card plus random chances.

**Picks 6-10:** Alternating surges and floors. Surge frequency ~every 1.5 picks
due to 3-symbol cards with repetition bonuses. Player consistently sees 2-3 S/A
cards per pack. Draft identity is clear: Warriors.

**Picks 11-30:** Fully converged. Surges deliver 3 pair-matched cards routinely.
Floor packs provide 1 pair-matched card. The player builds a Warriors deck with
~75% S/A-tier cards, some Ramp splash, and a few power picks.

### Failure Modes

1. **Pair-pool exhaustion.** With ~18 cards per archetype pair, aggressive
   surging can deplete the pool by pick 20. **Mitigation:** Fallback to R1
   filtering when pair pool drops below 5 unseen cards. This gracefully degrades
   to V7 Surge+Floor performance.

2. **Archetype misidentification.** If the player drafts Tide cards evenly split
   between Zephyr-secondary and Stone-secondary early, the algorithm may
   oscillate between Warriors and Sacrifice targeting. **Mitigation:** Use a
   3-pick trailing window for secondary resonance identification rather than
   cumulative totals, allowing faster correction.

3. **Low-overlap pair penalty.** Flash/Ramp (Zephyr pair, 25% natural fitness)
   gets worse surges than Warriors/Sacrifice (Tide pair, 50%). **Mitigation:**
   Pair filtering partially bypasses this since it targets 80% home-archetype
   cards regardless of fitness. The remaining 20% sibling cards are where
   fitness matters, compressing the cross-pair gap from 25 percentage points (R1
   filtering) to ~5 points.

### Parameter Variants

| Parameter         | Default         | Variant A            | Variant B                                |
| ----------------- | --------------- | -------------------- | ---------------------------------------- |
| Surge threshold   | T=3             | T=2 (more frequent)  | T=4 (less frequent, higher precision)    |
| Pair-pool minimum | 5 cards         | 3 cards (aggressive) | 8 cards (conservative, earlier fallback) |
| Floor slots       | 1 pair-filtered | 2 pair-filtered      | 0 (pure surge, no floor)                 |

**Recommended testing:** Default + Variant A (T=2) which may over-converge but
could push M3 above 2.0 under Pessimistic fitness.

______________________________________________________________________

## Set Design Specification

### 1. Pool Breakdown by Archetype

| Archetype    | Total Cards | Home-Only | Cross-Archetype | Generic |
| ------------ | :---------: | :-------: | :-------------: | :-----: |
| Flash        |     40      |    22     |       18        |   --    |
| Blink        |     40      |    22     |       18        |   --    |
| Storm        |     40      |    22     |       18        |   --    |
| Self-Discard |     40      |    22     |       18        |   --    |
| Self-Mill    |     40      |    22     |       18        |   --    |
| Sacrifice    |     40      |    22     |       18        |   --    |
| Warriors     |     40      |    22     |       18        |   --    |
| Ramp         |     40      |    22     |       18        |   --    |
| Generic      |     40      |    --     |       --        |   40    |
| **Total**    |   **360**   |  **176**  |     **144**     | **40**  |

"Cross-Archetype" = cards designed to be at least A-tier in the sibling
archetype sharing the same primary resonance. 18 of 40 = 45% cross-archetype
rate per archetype, achievable with moderate design effort per Research Agent B.

### 2. Symbol Distribution

|        Symbol Count         | Cards | % of Pool | Example              |
| :-------------------------: | :---: | :-------: | -------------------- |
|         0 (generic)         |  40   |    11%    | No resonance symbols |
| 3 symbols (with repetition) |  320  |    89%    | (Tide, Tide, Zephyr) |

All non-generic cards carry exactly 3 symbols. Repetition is mandatory for
archetype-identity encoding.

**Repetition pattern breakdown (among 320 non-generic cards):**

| Pattern                                | Count | % of Non-Generic | Example                | Signal                                      |
| -------------------------------------- | :---: | :--------------: | ---------------------- | ------------------------------------------- |
| AAB (primary repeated, secondary once) |  176  |       55%        | (Tide, Tide, Zephyr)   | Strong archetype ID: Warriors               |
| ABB (primary once, secondary repeated) |  64   |       20%        | (Tide, Zephyr, Zephyr) | Archetype leaning toward secondary          |
| ABC (all different)                    |  64   |       20%        | (Tide, Zephyr, Ember)  | Archetype + splash color                    |
| AAA (all same)                         |  16   |        5%        | (Tide, Tide, Tide)     | Pure resonance; serves both Tide archetypes |

### 3. Dual-Resonance Breakdown

| Type                                    | Cards | % of Pool | Filtering Implications                                   |
| --------------------------------------- | :---: | :-------: | -------------------------------------------------------- |
| Generic (no symbols)                    |  40   |    11%    | Not filtered; appears in random slots only               |
| Single-resonance (AAA pattern)          |  16   |   4.5%    | Matches both archetypes of that resonance on R1 filter   |
| Dual-resonance (AAB, ABB, ABC patterns) |  304  |   84.5%   | Matchable by pair filter; ABC also matchable by tertiary |

Effective dual-resonance rate: **84.5%** (304/360). This is well above the 40%
threshold Research Agent A identified as critical for pair-matching
sustainability.

### 4. Per-Resonance Pool Sizes

| Resonance | Cards with this as primary (position 1) | Cards with this as any symbol |        Pair-filtered subpool per archetype        |
| --------- | :-------------------------------------: | :---------------------------: | :-----------------------------------------------: |
| Ember     |                   80                    |             ~200              |      ~22 per Ember archetype (Blink, Storm)       |
| Stone     |                   80                    |             ~200              | ~22 per Stone archetype (Self-Discard, Self-Mill) |
| Tide      |                   80                    |             ~200              |   ~22 per Tide archetype (Sacrifice, Warriors)    |
| Zephyr    |                   80                    |             ~200              |      ~22 per Zephyr archetype (Flash, Ramp)       |

Each archetype's pair-filtered subpool contains ~22 cards: the 22 home-only
cards (all carry the AAB pattern matching their archetype) plus ~0-2
cross-archetype cards from the sibling whose ABC pattern happens to match. The
18 cross-archetype cards per archetype also appear in the pair pool, bringing
the total pair-matchable pool to ~40 cards per archetype, of which ~22 are home
(S-tier) and ~18 are sibling (fitness-dependent).

**Precision calculation at Graduated Realistic fitness (36% weighted average):**

- Pair filter yields: 22 home (S-tier) + 18 sibling (36% A-tier = ~6.5 A-tier)
  out of 40 = 28.5/40 = **71% S/A precision**
- R1 filter yields: 40 home + 40 sibling (36% = ~14.4 A-tier) out of 80 =
  54.4/80 = **68% S/A precision**
- Pair advantage: +3 percentage points at this fitness level; advantage grows at
  lower fitness because pair filtering concentrates draws in home-archetype
  cards (55% home vs. 50% for R1).

### 5. Cross-Archetype Requirements

| Pair   | Archetype A  | Archetype B |  Cross-Arch Cards Required  | Target A-tier Rate |         Design Difficulty          |
| ------ | ------------ | ----------- | :-------------------------: | :----------------: | :--------------------------------: |
| Tide   | Warriors     | Sacrifice   | 18 per archetype (36 total) |        50%         |    Low (shared creature theme)     |
| Stone  | Self-Discard | Self-Mill   | 18 per archetype (36 total) |        40%         |     Medium (shared void theme)     |
| Ember  | Blink        | Storm       | 18 per archetype (36 total) |        30%         | High (character vs. event tension) |
| Zephyr | Flash        | Ramp        | 18 per archetype (36 total) |        25%         |   High (tempo vs. ramp tension)    |

Each archetype has 18 cross-archetype cards. The A-tier rate is the fraction of
these that the sibling archetype actually wants. Even at 25% (Zephyr pair), 4-5
of the 18 cross-archetype cards are genuinely good in the sibling, which is
sufficient for the algorithm because pair filtering concentrates draws in
home-archetype cards.

### 6. What the Card Designer Must Do Differently

Compared to V7's assumptions:

1. **Every non-generic card gets exactly 3 resonance symbols.** Previously most
   cards had 1-2. The designer must assign a meaningful 3-symbol sequence to
   each card that encodes its archetype identity. Use the AAB pattern (55% of
   cards) for core archetype cards: (Primary, Primary, Secondary). Use ABB (20%)
   for cards that lean toward the secondary archetype. Use ABC (20%) for
   splash-friendly cards. Use AAA (5%) for pure-resonance utility cards shared
   between both archetypes of that resonance.

2. **Symbol assignment IS archetype assignment.** The symbols are no longer
   decorative faction markers -- they are the algorithm's primary data source
   for archetype identification. A Warriors card MUST carry (Tide, Tide, Zephyr)
   or (Tide, Zephyr, Zephyr) or (Tide, Zephyr, X), never (Tide, Stone, Ember).

3. **Design 18 cross-archetype cards per archetype.** For each archetype, 18 of
   40 cards should be at least A-tier in the sibling. For high-overlap pairs
   (Warriors/Sacrifice), this is natural. For low-overlap pairs (Flash/Ramp),
   focus on generic-utility effects: unconditional draw, removal,
   energy-flexible cards.

4. **Generic card count increases from 36 to 40.** These are symbolless cards
   that appear only in random slots, providing splash diversity.

**Worked example for Warriors (Tide/Zephyr):** 40 total cards. 22 home-only:
core Warrior-tribal cards with symbols (Tide, Tide, Zephyr), designed to be
S-tier in Warriors and B/C in Sacrifice. 18 cross-archetype: cards with (Tide,
Tide, Zephyr) symbols but broader mechanics (e.g., "When a character enters
play, Kindle 1" -- useful in both Warriors and Sacrifice). Of these 18, the
designer targets ~9 (50%) to be genuinely A-tier in Sacrifice. The remaining 9
will be B-tier in Sacrifice but still carry the correct symbol pattern for
pair-filtering.
