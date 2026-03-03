# Research Agent B: Fitness Calibration for Cross-Archetype Card Design

## The Central Question

V7 assumed uniform fitness across all sibling pairs: every pair of archetypes
sharing a primary resonance was modeled at the same sibling A-tier rate (50%
Moderate, 25% Pessimistic, etc.). The V8 designer believes even 50% may be
optimistic. This research establishes whether uniform fitness is realistic,
proposes per-pair calibration, and defines what each fitness level actually
demands of card designers.

______________________________________________________________________

## 1. The Archetype Circle and Its Sibling Relationships

The 8 archetypes form a circle where adjacency = shared resonance:

```
Flash(Ze/Em) -- Blink(Em/Ze) -- Storm(Em/St) -- Self-Discard(St/Em)
   |                                                      |
Ramp(Ze/Ti) -- Warriors(Ti/Ze) -- Sacrifice(Ti/St) -- Self-Mill(St/Ti)
```

There are two categories of sibling relationship:

**Co-primary siblings** share a primary resonance (the draft algorithm's main
targeting axis). These are the pairs that determine S/A precision when the
algorithm draws from a resonance-filtered pool:

| Resonance | Co-Primary Pair          | Shared Primary |
| --------- | ------------------------ | -------------- |
| Zephyr    | Flash + Ramp             | Zephyr         |
| Ember     | Blink + Storm            | Ember          |
| Stone     | Self-Discard + Self-Mill | Stone          |
| Tide      | Sacrifice + Warriors     | Tide           |

**Complementary siblings** share both resonances but reversed (primary of one =
secondary of the other). These pairs are adjacent on the circle twice removed
and share the deepest structural connection:

| Complementary Pair                    | Resonance Overlap     |
| ------------------------------------- | --------------------- |
| Flash (Ze/Em) + Blink (Em/Ze)         | Both Zephyr and Ember |
| Storm (Em/St) + Self-Discard (St/Em)  | Both Ember and Stone  |
| Self-Mill (St/Ti) + Sacrifice (Ti/St) | Both Stone and Tide   |
| Warriors (Ti/Ze) + Ramp (Ze/Ti)       | Both Tide and Zephyr  |

Complementary pairs are not co-primary siblings (they have different primaries),
but they share the most resonance DNA. Under pair-matching algorithms like V5's
Pair-Escalation, complementary pairs have the highest natural overlap.

______________________________________________________________________

## 2. Mechanical Analysis: Per-Pair Overlap Assessment

Using Dreamtides' actual keyword and mechanical vocabulary (Dissolve, Banish,
Abandon, Reclaim, Kindle, Foresee, Prevent, Materialize triggers, Figments,
Fast, Discover, Copy, Gain Control, subtypes like Warrior/Ancient/Spirit-Animal,
card types Character/Event), I assess each co-primary pair's natural mechanical
distance.

### Tide Co-Primary: Warriors vs. Sacrifice

**Warriors/Midrange:** Character-centric, tribal synergies (Warrior subtype),
Kindle effects, Materialized/Judgment triggers on characters, midrange cost
curve (3-4 energy average). Wants to build a board of characters and win through
spark accumulation.

**Sacrifice/Abandon:** Also character-centric but uses Abandon as a
cost/enabler, Dissolved triggers, death-benefit effects. Wants characters on the
battlefield but also wants to remove them for value. Lower cost curve (cheap
sacrifice fodder + expensive payoffs).

**Overlap assessment: HIGH.** Both archetypes fundamentally care about
characters on the battlefield. A character with "Materialized: Kindle 1" is
S-tier in Warriors and A-tier in Sacrifice (the character has spark and a useful
enter-play effect even if you later sacrifice it). A character with "Dissolved:
Draw 2" is S-tier in Sacrifice and B+ in Warriors (you can still play it as a
body; if the opponent dissolves it, you get value). The shared "creatures
matter" theme creates a large surface area for dual-playable cards. Generic
removal (Dissolve target character) works in both. **Estimated natural A-tier
overlap: 35-45%.**

### Ember Co-Primary: Blink vs. Storm

**Blink/Flicker:** Character-centric with Materialized triggers, bounce effects,
re-entry synergies. Wants characters that generate value when entering play and
effects that return characters to hand for re-use.

**Storm/Spellslinger:** Event-centric, cares about casting multiple spells,
Prevent/counter effects, possible Foresee and draw chains. Wants to cast many
Events and accumulate incremental advantage.

**Overlap assessment: LOW.** These archetypes have fundamentally different
card-type preferences. Blink wants characters with enter-play triggers; Storm
wants cheap events that chain together. A "Materialized: Foresee 2" character is
S-tier in Blink but C-tier in Storm (Storm does not want to spend 3+ energy on a
character body when it could cast two events instead). An event that says "Draw
2 cards" is S-tier in Storm and B-tier in Blink (Blink can use it but would
prefer a character with similar value). The only shared ground is generic card
advantage (draw effects) and fast interaction (Prevent). **Estimated natural
A-tier overlap: 15-25%.**

### Stone Co-Primary: Self-Discard vs. Self-Mill

**Self-Discard:** Cares about cards leaving hand (discard triggers, hand-size
manipulation). Wants effects that discard as a cost and cards that benefit from
being discarded.

**Self-Mill/Reanimator:** Cares about cards in the void (Reclaim, cards milled
from deck to void). Wants to fill the void from the deck and then Reclaim
powerful cards from it.

**Overlap assessment: MODERATE.** Both archetypes interact with the void, but
they get cards there differently (discard from hand vs. mill from deck). A card
with "Reclaim 3" is S-tier in Self-Mill and B+ in Self-Discard (Self-Discard
puts cards in the void via discard, so it can use Reclaim but less efficiently).
Cards that say "When this card is put into your void, draw a card" work for both
(milling triggers it, discarding triggers it). The void-as-resource theme
provides moderate overlap, but the mechanical engines are different enough that
many cards are narrow. **Estimated natural A-tier overlap: 25-35%.**

### Zephyr Co-Primary: Flash vs. Ramp

**Flash/Tempo:** Speed-oriented, Fast cards, low-cost efficient plays, possibly
bounce/tempo effects, wants to play reactively during opponent's turn.

**Ramp/Spirit Animals:** Energy acceleration, expensive finishers, Spirit Animal
subtype synergies, wants to reach high energy totals and play powerful expensive
cards.

**Overlap assessment: VERY LOW.** These archetypes have diametrically opposed
strategic goals. Flash wants cheap, efficient, reactive plays; Ramp wants
expensive, proactive haymakers. A 1-cost fast event that bounces a character is
S-tier in Flash and F-tier in Ramp (Ramp has no use for tempo plays; it wants to
BE the player deploying threats). A 7-cost Spirit Animal with "Materialized:
Gain 3 energy" is S-tier in Ramp and F-tier in Flash (Flash cannot afford it and
does not want expensive cards). The only shared ground is generic utility (card
draw, removal) that works at any speed. **Estimated natural A-tier overlap:
10-20%.**

______________________________________________________________________

## 3. Per-Pair Fitness Summary

| Co-Primary Pair                  | Mechanical Distance | Natural A-Tier (no design effort) | With Moderate Design Effort | With Heavy Design Effort |
| -------------------------------- | :-----------------: | :-------------------------------: | :-------------------------: | :----------------------: |
| Warriors / Sacrifice (Tide)      |         Low         |              35-45%               |           50-60%            |          65-75%          |
| Self-Discard / Self-Mill (Stone) |       Medium        |              25-35%               |           40-50%            |          55-65%          |
| Blink / Storm (Ember)            |        High         |              15-25%               |           30-40%            |          45-55%          |
| Flash / Ramp (Zephyr)            |      Very High      |              10-20%               |           25-35%            |          40-50%          |

This table reveals a critical finding: **fitness is not uniform across sibling
pairs.** The Tide pair (Warriors/Sacrifice) naturally produces 2-3x the
cross-archetype overlap of the Zephyr pair (Flash/Ramp). Any fitness model that
assumes uniform rates across all pairs is masking significant per-pair variance.

______________________________________________________________________

## 4. What "15% Sibling A-Tier" Actually Means

At 15% sibling A-tier (V8's Harsh model), out of every 20 cards in an archetype,
only 3 need to be at least A-tier in the sibling. This is approximately what a
designer gets "for free" -- without deliberately trying to create
cross-archetype cards:

- Generic utility cards (unconditional draw, unconditional removal) that any
  deck wants
- Characters with no archetype-specific text that simply have good spark/cost
  ratios
- Cards whose mechanical theme incidentally serves both archetypes

For the Tide pair (Warriors/Sacrifice), 15% is conservative -- the shared
"creatures matter" theme probably produces 20-30% overlap without any effort.
For the Zephyr pair (Flash/Ramp), 15% may require mild intentional effort
because the archetypes have so little mechanical common ground.

**15% is the realistic floor for pairs with moderate overlap and
achievable-without-trying for pairs with high overlap.** It represents the "I
designed each archetype independently and didn't think about the sibling at all"
scenario.

______________________________________________________________________

## 5. Four Fitness Models with Per-Pair Granularity

### Model 1: Optimistic (V7 backward-compatible)

All sibling A-tier = 100%. Every card in a resonance pool is playable in every
archetype sharing that resonance. Unrealistic but useful as a ceiling.

### Model 2: Graduated Realistic (V8 recommended default)

Per-pair rates reflecting natural mechanical overlap plus moderate design
effort:

| Pair                     | Sibling A-Tier | Rationale                                                  |
| ------------------------ | :------------: | ---------------------------------------------------------- |
| Warriors / Sacrifice     |      50%       | High natural overlap; moderate effort yields 50%           |
| Self-Discard / Self-Mill |      40%       | Void theme links them; some cards bridge naturally         |
| Blink / Storm            |      30%       | Low natural overlap; requires deliberate bridging          |
| Flash / Ramp             |      25%       | Very low natural overlap; even with effort, hard to bridge |

**Weighted average (equal archetype weight): 36%.** This is harsher than V7's
Moderate (50%) but more realistic. It also captures that some archetypes will
experience the draft algorithm differently -- a Warriors player gets better
packs than a Flash player because Warriors shares more with its sibling.

### Model 3: Pessimistic (low design effort)

Per-pair rates reflecting natural overlap with minimal deliberate
cross-archetype design:

| Pair                     | Sibling A-Tier | Rationale                                   |
| ------------------------ | :------------: | ------------------------------------------- |
| Warriors / Sacrifice     |      35%       | Natural overlap only, minimal bridging      |
| Self-Discard / Self-Mill |      25%       | Some void synergy, mostly different engines |
| Blink / Storm            |      15%       | Almost entirely different card types wanted |
| Flash / Ramp             |      10%       | Fundamentally opposed strategies            |

**Weighted average: 21%.** Under this model, the draft algorithm struggles
everywhere and fails outright for Zephyr archetypes.

### Model 4: Hostile (archetypes designed in isolation)

All sibling A-tier = 0-10%. Each archetype was designed without any
consideration of its sibling. The only cross-archetype playability comes from
truly generic cards. This represents the scenario the designer fears most.

**Weighted average: ~8%.** Under this model, resonance targeting provides almost
no cross-archetype benefit. Drawing from the R1 pool yields 50% home cards
(S-tier) and 50% sibling cards that are mostly B/C/F. Effective S/A precision
per R1 slot: ~54%.

______________________________________________________________________

## 6. Implications for Algorithm Design

### The Per-Archetype Experience Problem

Under Graduated Realistic, the Warriors player sees 2 S/A cards per
post-convergence pack while the Flash player sees 1.5. This 33% quality gap
between archetypes is visible to players and feels unfair. Algorithms must
either:

1. Accept the gap and mitigate through other means (B-tier cards being "good
   enough")
2. Compensate with pair-specific tuning (more aggressive surges for low-overlap
   pairs)
3. Redesign the archetype circle to equalize mechanical overlap

Option 3 is the most effective but requires the most design investment. The
designer could intentionally create "bridge mechanics" for low-overlap pairs:

- **Flash/Ramp bridge:** Cards that scale with available energy (cheap when you
  are low, powerful when you have excess). A card that says "Fast. Cost equals
  your current energy. Draw cards equal to the cost paid" is playable in both
  Flash (cheap reactive draw) and Ramp (massive draw when energy-rich).
- **Blink/Storm bridge:** Characters with "Materialized: Copy the next Event you
  play this turn." Gives Blink a character body with an enter-play trigger while
  giving Storm a spell-doubling effect.

### The Weighted Average Trap

V7 used a single uniform fitness rate. If V8 uses a uniform rate, it should use
the **weighted average of the per-pair rates, not the best pair's rate.** Under
Graduated Realistic, the correct uniform proxy is ~36%, not 50%. V7's Moderate
model was effectively assuming all pairs look like Warriors/Sacrifice -- the
highest-overlap pair.

### What the Card Designer Gets "For Free"

At every fitness level, the designer gets certain cross-archetype cards without
trying:

- **Generic removal** (Dissolve target character, Prevent target card): playable
  in every archetype. ~5-8% of the pool.
- **Unconditional draw** (Draw N cards): playable everywhere. ~3-5% of the pool.
- **Efficient vanilla bodies** (3-cost 3-spark character with no text): usable
  as curve-fillers in most archetypes. ~5% of the pool.
- **Total "free" cross-archetype floor: ~15% of cards.** This is the baseline
  that requires zero design consideration.

Above 15%, every percentage point requires intentional design: cards must be
written to serve two mechanical themes simultaneously. This is genuinely
difficult for mechanically distant pairs and straightforward for mechanically
close ones.

______________________________________________________________________

## 7. Recommendations for V8

1. **Adopt per-pair fitness as the default simulation model.** Uniform fitness
   masks the real problem. The Graduated Realistic model (36% weighted average)
   should replace V7's Moderate (50%) as the primary test target.

2. **Report per-archetype M3, not just the average.** If Warriors gets 2.1 S/A
   while Flash gets 1.4, the average of 1.75 hides a serious imbalance. Every V8
   simulation should report the worst-archetype M3.

3. **Identify whether pool composition can substitute for fitness.** If
   Flash/Ramp cards carry more resonance symbols (enabling better
   pair-matching), the algorithm may compensate for low mechanical overlap
   through better targeting. This is a question for Research Agent A's pool
   composition analysis.

4. **Establish a "minimum viable fitness" per pair.** Below what sibling A-tier
   rate does the draft experience become unacceptable for a given archetype? If
   the answer is 15% and Flash/Ramp naturally achieves 15% without effort, then
   the designer need not invest in bridging that pair -- the algorithm must
   solve it structurally instead.

5. **Test the "bridge card" concept.** If the designer creates 5-8 intentional
   bridge cards per low-overlap pair (cards specifically written to be A-tier in
   both archetypes), how much does this move the fitness needle? At 40 cards per
   archetype, 8 bridge cards = 20% sibling A-tier, which would lift Flash/Ramp
   from 10% to 20%. This is the minimum viable design intervention.

6. **Consider the Graduated Realistic model's interaction with pair-matching
   algorithms.** V5's Pair-Escalation Slots achieved 80% S-tier precision under
   Optimistic fitness by matching ordered pairs. Under Graduated Realistic with
   per-pair rates, pair-matching benefits high-overlap pairs enormously
   (Warriors/Sacrifice) while barely helping low-overlap pairs (Flash/Ramp).
   This could widen the per-archetype quality gap rather than narrowing it.

______________________________________________________________________

## 8. Summary: The Fitness Landscape

The fitness landscape V8 algorithms must navigate is not flat. It has peaks
(Warriors/Sacrifice at 35-50% natural overlap) and valleys (Flash/Ramp at
10-25%). The V7 assumption of uniform 50% sibling A-tier was optimistic by
roughly 14 percentage points on a weighted-average basis and by 25-40 percentage
points for the worst pair.

The honest answer to "what is the right fitness model?" is: **it depends on the
pair, and any realistic model must account for mechanical distance between
sibling archetypes.** A single number can serve as a summary statistic, but the
per-pair distribution is what determines whether players in every archetype have
a good experience.

For V8's default simulation target, the Graduated Realistic model (Warriors 50%,
Self-Discard/Self-Mill 40%, Blink/Storm 30%, Flash/Ramp 25%; weighted average
36%) represents an honest assessment of achievable cross-archetype fitness with
moderate but not heroic design effort.
