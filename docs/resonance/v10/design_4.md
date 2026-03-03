# Design 4: Aggressive vs. Passive AI Spectrum

## 1. Key Findings

- **Aggression is a direct contraction knob.** V9's 12% contraction removes ~38 cards per pick at pick 4 (pool=317). An aggressive AI taking 4-5 cards per round from its archetype pool removes ~12 high-affinity cards from one lane. To match V9's total removal, the system needs ~7-8 AIs at moderate-high aggression, with each removing 4-5 cards per round. Variable aggression across AIs naturally produces variable lane widths without any reactivity.

- **Mixed aggression creates organic lane width.** When some AIs draft aggressively (taking 5 cards per round, always top archetype picks) and others draft passively (taking 2-3 cards per round, sometimes generics), the player faces a landscape where some lanes are thin and competitive while others are wide and welcoming. This is the "open lane" signal without any reactive mechanism -- the passive AI's lane is open because the AI simply doesn't take many cards.

- **Aggression replaces reactivity for convergence.** V9's convergence comes from the pool concentrating toward the player's archetype as contraction removes irrelevant cards. AI aggression achieves the same net effect differently: aggressive AIs rapidly deplete their own archetypes, so the remaining pool naturally enriches in under-drafted archetypes. The player who reads which lane is open benefits from this enrichment without any AI needing to "notice" the player.

- **Fully aggressive AIs are punitive when competing with the player.** If the Warriors AI takes 5 top Warriors cards every round and the player also drafts Warriors, the combined depletion rate leaves too few quality Warriors cards by pick 10. The result is "unplayable" rather than "challenging but viable." Aggression in the player's lane must be bounded.

- **Passive AIs create insufficient signals.** If all AIs are passive (take only 2 cards per round, often generics), total pool removal is ~16 cards per round -- far below V9's 38-card target. The pool stays diffuse, M3 stays near baseline (~2.0), and M11 cannot reach 3.0. Some AIs must be aggressive for convergence to work.

- **The "not punitive" constraint demands a cap on per-archetype depletion.** Even when the player fights an aggressive AI for its lane, the player should still see 1.5+ S/A cards per pack. This means no single AI should strip more than ~60% of its archetype's top-quartile cards before pick 15. This translates to a per-AI aggression ceiling of roughly 4 archetype-specific cards per round.

- **Pair-affinity discrimination is still required for M11.** Aggression controls total contraction intensity, but the V9 finding holds: AIs that distinguish Warriors from Sacrifice within Tide cards (using pair-affinity scores) achieve M11 ~ 3.2; AIs using only visible resonance cap M11 at ~2.1. The AI's internal preference function must include pair-affinity data.

---

## 2. Three Algorithm Proposals

### Proposal A: "Uniform Moderate" (All AIs at 70% aggression)

**Player description:** "You're drafting against 7 AI opponents who each have a preferred archetype. They're decent drafters -- they usually take good cards for their deck, but sometimes they grab a powerful generic instead."

**Technical description:** 7 AIs, one per non-player archetype (randomly assigned each game, leaving one archetype uncontested). Each AI takes 4 cards per round: 70% chance each pick is the top pair-affinity card for its archetype, 30% chance it's the highest-power generic or off-archetype card. AIs use pair-affinity scores for archetype picks. No reactivity (Level 0). Total removal: 28 cards/round (7 AIs x 4 cards).

**AI behavior:** Consistent within a draft, varied across drafts via archetype assignment. The 30% off-archetype rate creates some noise that prevents perfect prediction while still maintaining clear lane signals.

**Predicted metrics:** M3 ~ 2.45, M10 ~ 3.5, M11 ~ 2.85, M6 ~ 82%. Contraction rate (28/317 = 8.8%) undershoots V9's 12%, limiting M11. The uniform aggression means no lane is dramatically thinner than another, reducing signal strength.

### Proposal B: "Tiered Aggression" (AIs assigned high/medium/low aggression)

**Player description:** "You're drafting against 7 AI opponents. Some are focused specialists who snap up their archetype aggressively; others are more casual drafters who leave more on the table. Reading which lanes are heavily contested and which are open is the key skill."

**Technical description:** 7 AIs, one per non-player archetype. Each AI is assigned an aggression tier: 2 Aggressive (5 cards/round, 90% archetype-specific), 3 Moderate (4 cards/round, 75% archetype-specific), 2 Passive (3 cards/round, 60% archetype-specific). Aggression tiers are randomly assigned each game. AIs use pair-affinity scores. Level 0 reactivity. Total removal: 2(5) + 3(4) + 2(3) = 28 cards/round.

**AI behavior:** Aggressive AIs create visibly thin lanes (strong signals). Passive AIs create wide-open lanes (easy drafting). Moderate AIs are in between. The player's skill is identifying which AIs are aggressive (avoid those lanes) and which are passive (draft there for easy convergence).

**Predicted metrics:** M3 ~ 2.55, M10 ~ 3.2, M11 ~ 3.0, M6 ~ 84%. The aggressive AIs boost contraction in their lanes to V9-equivalent levels; the passive AIs provide "open lane" experiences. Per-archetype variance is higher but the open lanes hit M11 targets.

### Proposal C: "Escalating Aggression" (AIs start passive, become aggressive)

**Player description:** "You're drafting against 7 AI opponents who start exploring and become more focused as the draft goes on. Early packs are wide open; late packs get competitive."

**Technical description:** 7 AIs, one per non-player archetype. All AIs start at low aggression (2 cards/round, 50% archetype-specific in picks 1-5) and escalate to high aggression (5 cards/round, 95% archetype-specific by pick 15+). The escalation curve is: picks 1-5 = 2 cards/round; picks 6-10 = 3 cards/round; picks 11-15 = 4 cards/round; picks 16+ = 5 cards/round. AIs use pair-affinity scores. Level 0 reactivity. Total removal: 14 early, 21 mid, 28 late, 35 very late.

**AI behavior:** Early draft is wide open (low total removal = many viable archetypes, good M1). Mid-draft begins concentrating (AIs settle into lanes). Late draft is heavily concentrated (AIs have stripped most of their archetype, remaining pool is enriched in open lanes). This naturally produces the V9 convergence ramp.

**Predicted metrics:** M3 ~ 2.65, M10 ~ 2.8, M11 ~ 3.15, M6 ~ 85%. Early openness solves M1/M2. Escalating removal creates the convergence ramp that V9's 12% flat rate provides. Late-draft aggression pushes M11 above 3.0. M10 improves because early packs are reliably good (low AI competition) and late packs are reliably concentrated (high AI competition has cleared other archetypes).

---

## 3. Champion Selection

**Champion: Proposal C -- "Escalating Aggression"**

Justification: Proposal C is the only design that directly addresses V9's M5 failure (convergence at pick 9.6) and M10 failure (3.8 consecutive bad packs). By starting AIs passive and escalating, it creates natural draft phases: exploration (picks 1-5), transition (picks 6-10), and convergence (picks 11+). The escalation curve matches the player's own drafting arc -- early exploration, mid-draft commitment, late-draft deck completion -- which makes the AI behavior feel like authentic drafting rather than algorithmic tuning.

Proposal A's uniform aggression produces flat metrics without solving V9's phase-specific failures. Proposal B's tiered aggression creates interesting signal reading but doesn't solve the transition zone problem -- moderate AIs still create M10 streaks in picks 6-10. Proposal C's escalating structure directly targets the transition zone by keeping AI competition low during the critical commitment window.

---

## 4. Champion Deep-Dive: Escalating Aggression

### How It Works

Seven AI drafters are assigned to seven of the eight archetypes at random each game (one archetype is left uncontested). Each AI has pair-affinity scores for all cards in the pool, identical to V9 Hybrid B's encoding. The AI's pick logic is deterministic given its aggression phase:

- **Phase 1 (picks 1-5):** Each AI takes 2 cards per round. 50% of picks are the top pair-affinity card for its archetype; 50% are the highest-power available card regardless of archetype. This simulates early-draft exploration where even focused drafters grab powerful cards.
- **Phase 2 (picks 6-10):** Each AI takes 3 cards per round. 75% archetype-specific, 25% best-available power.
- **Phase 3 (picks 11-15):** Each AI takes 4 cards per round. 90% archetype-specific, 10% best-available power.
- **Phase 4 (picks 16+):** Each AI takes 5 cards per round. 95% archetype-specific, 5% best-available power.

Total cards removed per round: Phase 1 = 14, Phase 2 = 21, Phase 3 = 28, Phase 4 = 35. As a percentage of pool at each phase boundary: Phase 1 ~ 4.4% (14/317), Phase 2 ~ 8.3% (21/253), Phase 3 ~ 14.7% (28/190), Phase 4 ~ 26.9% (35/130). The escalation naturally produces an accelerating contraction curve that matches V9's 12% average while front-loading openness and back-loading concentration.

### What the Player Sees vs. What the AIs Do

**Picks 1-5 (player sees: wide-open packs).** Each pack contains 4 cards drawn from a pool that has lost only ~14 cards per round. Most archetypes are well-represented. The player sees 3+ archetypes with S/A options (M1 target). AIs are quietly exploring -- taking some archetype cards and some generics. The player cannot yet tell which lanes are contested.

**Picks 6-10 (player sees: some lanes thinning).** AIs have escalated. The 2 aggressive-phase AIs are now taking 3 archetype-specific cards per round. Archetypes assigned to these AIs start showing fewer S/A options. The player who reads the signal -- "I'm seeing fewer good Storm cards but lots of good Warriors cards" -- can commit to the open lane. The 1 uncontested archetype is visibly abundant.

**Picks 11-15 (player sees: committed lane delivers).** If the player committed to an open lane (no AI or passive AI), packs are rich with S/A options (2.5-3.0 per pack). If the player is fighting an AI's lane, packs still contain 1.5-2.0 S/A options (not great, but playable). AIs at Phase 3 are stripping their own archetypes heavily.

**Picks 16+ (player sees: deep convergence).** The pool has contracted to ~130 cards. AIs are taking 5 cards each per round, but most archetype-specific cards in their lanes are already gone. Remaining cards in contested lanes are bridge cards and generics. The player's open lane is now heavily concentrated in the remaining pool, delivering 3+ S/A cards per pack (M11 target).

### Example Draft: Player Drafts Warriors (Open Lane)

Game setup: AIs assigned to Flash, Blink, Storm, Self-Discard, Self-Mill, Sacrifice, Ramp. Warriors is the uncontested archetype.

| Pick | AI total removed | Pool remaining | Player's Warriors S/A per pack | Notes |
|:----:|:----------------:|:--------------:|:------------------------------:|-------|
| 1-5  | ~70 cards        | ~286           | 1.5-2.0                        | Wide open, exploring |
| 6-8  | ~63 more         | ~220           | 2.0-2.5                        | Player reads Warriors open, commits |
| 9-12 | ~98 more         | ~118           | 2.5-3.0                        | Other archetypes thinning, Warriors enriched |
| 13-18| ~155 more        | ~60            | 3.0-3.5                        | Pool heavily concentrated |
| 19-25| remaining        | ~30 (floor)    | 3.0+                           | Near-maximum concentration |

### Example Draft: Player Fights Sacrifice AI (Contested Lane)

Same setup, but player commits to Sacrifice instead of open Warriors.

| Pick | Sacrifice AI removed | Player's Sacrifice S/A per pack | Notes |
|:----:|:--------------------:|:-------------------------------:|-------|
| 1-5  | ~5 Sacrifice cards   | 1.5-2.0                         | AI taking some, but pool is large |
| 6-10 | ~12 more             | 1.5-1.8                         | AI escalating, Sacrifice thinning |
| 11-15| ~16 more             | 1.5-2.0                         | Bridge cards and generics help |
| 16+  | ~15 more             | 1.2-1.5                         | Thin but playable via bridges |

The contested lane produces a weaker deck (M6 ~ 70% vs. 85% in open lane) but not an unplayable one. Bridge cards (moderate affinity for both Sacrifice and Self-Mill) survive because the Self-Mill AI takes the Self-Mill-specific cards first. This satisfies the "not punitive" constraint.

### Failure Modes

1. **Player ignores signals entirely.** A power-chaser who takes the highest-raw-power card each pick will end up with a scattered deck (~55% concentration). This is intentional -- the system rewards archetype commitment.

2. **Two AIs on adjacent archetypes create a "dead resonance."** If both the Warriors AI and Sacrifice AI are present (they share Tide), Tide cards are depleted from both sides. A player committing to either Tide archetype faces double competition. Mitigation: the uncontested archetype is always on a different resonance, providing a clear escape. Additionally, bridge cards survive longer when both sibling AIs want them but neither prioritizes them highly.

3. **Late-game pool exhaustion.** By pick 25, the pool may be at the 30-card floor. Packs become repetitive. Mitigation: the pool minimum floor prevents complete depletion, and the escalation curve means most archetype-stripping happens in picks 11-20, not 25+.

4. **Predictable AI escalation curve.** Experienced players may learn that AIs always escalate on the same schedule and plan accordingly. Mitigation: per-game AI archetype assignment provides variety, and the specific cards AIs take depend on the pool's random composition. The escalation timing is consistent (like a real drafter "settling in") rather than reactive.

---

## 5. AI Drafter Specification

| Parameter | Value |
|-----------|-------|
| Number of AIs | 7 (one per non-player archetype) |
| Archetype assignment | Random each game; one archetype uncontested |
| Pick logic | Top pair-affinity card for own archetype (archetype picks) or highest-power available (generic picks) |
| Pair-affinity data | 8 bits per card (two 4-bit floats), identical to V9 Hybrid B encoding |
| Reactivity level | Level 0 (fully predetermined; all picks deterministic from seed) |
| Phase 1 (picks 1-5) | 2 cards/round, 50% archetype / 50% power |
| Phase 2 (picks 6-10) | 3 cards/round, 75% archetype / 25% power |
| Phase 3 (picks 11-15) | 4 cards/round, 90% archetype / 10% power |
| Phase 4 (picks 16+) | 5 cards/round, 95% archetype / 5% power |
| Archetype pick selection | Highest pair-affinity score for AI's assigned archetype among remaining pool |
| Power pick selection | Highest raw power score among remaining pool, any archetype |
| Pack construction | After all AIs remove their cards for the round, draw 4 random cards from remaining pool |
| Pool minimum floor | 30 cards (AIs stop picking when pool reaches floor) |
| Per-game variety | AI archetype assignment varies; which archetype is uncontested varies |
| Signal mechanism | Thinning of specific archetypes visible through card availability in packs |

---

## Post-Critique Revision

### Summary of Critic's Verdict

Ranked 4th on M3/M11 (early phases remove only 14 cards/round, delaying pool concentration), 7/10 on player experience, 7/10 on signal reading, 7/10 on "not on rails." Retained for simulation as Slot 3. The critic's preferred improvement is Hybrid Y: 5 AIs with escalation and 3 open lanes, rather than 7 AIs with escalation and 1 open lane.

### 1. The Late Signal Problem

The critic is correct. At picks 6-8, AIs have only recently escalated from 2 to 3 cards per round. Total removal through pick 5 is roughly 70 cards (14/round x 5), leaving a pool of ~286 cards. That is not enough depletion for archetype-specific thinning to be legible to the player. A Warriors lane with 35 total Warriors cards in the starting pool loses perhaps 8-10 of those cards to off-archetype AI exploration in Phase 1. The signal is noise at that stage.

The signal only sharpens in Phase 2 (picks 6-10), precisely when the player needs to commit. This is backward: the player commits during the period they are reading signals, but in this design those two events are simultaneous rather than sequential. A player who delays commitment to wait for signal clarity pays a real cost in draft position.

**Mitigation within the 7-AI structure:** Increase Phase 1 archetype specificity. Rather than 50% archetype / 50% power, shift Phase 1 to 70% archetype / 30% power. This makes the 14 cards removed per round more concentrated in their respective lanes, producing visible lane thinning by pick 3-4. The tradeoff is that Phase 1 feels less exploratory — AIs are "already focused" from the start. That is a smaller narrative cost than late-arriving signals.

### 2. Should 7 AIs Reduce to 5

Yes, on balance. The critic's Hybrid Y argument is sound. With 7 AIs and one uncontested archetype, the player's optimal path is always "find the gap." That is a single-answer puzzle. With 5 AIs and three uncontested archetypes, the player faces a genuine three-way choice among open lanes, each requiring different card types and deck construction paths. The puzzle has texture.

The 7-AI structure was justified on contraction grounds: more AIs drive higher total removal. But the critic is also correct that escalating pick rates compensate for the lower AI count in Hybrid Y. Five AIs escalating to 5 cards per round in Phase 4 remove 25 cards/round late, plus market culling bridges the remaining gap. The contraction argument for 7 AIs weakens considerably when escalation is the primary contraction mechanism.

This design should accept the Hybrid Y structure for the simulation slot, or acknowledge explicitly that it is testing the 7-AI variant of a hypothesis Hybrid Y tests more cleanly.

### 3. Synchronized Escalation Artificiality

The critic is right that all 7 AIs escalating on the same schedule is detectable as artificial. Real drafters do not all "settle in" at pick 6 simultaneously. The synchronized phase transition is the clearest tell that these are algorithmic entities rather than opponents with individual arcs.

However, the fix is not complex. The escalation schedule can be varied per AI by applying a small individual offset at assignment time: each AI receives a phase-shift parameter drawn from {-1, 0, +1} picks, applied uniformly across its phase boundaries. The player-facing result is that some AIs appear to "find their lane" earlier and others later, which matches observable human drafting behavior. The mathematical impact on total contraction is negligible (averaging to zero across AIs) but the phenomenology improves substantially.

### 4. Staggered vs. Per-AI-Varied Escalation

Staggered escalation (each AI transitions on a different pick number) and per-AI aggression tiers (Proposal B from the original design) address the same problem through different mechanisms. Staggered escalation preserves the uniform end-state (all AIs converge to high aggression by pick 16+) while varying the transition timing. Per-AI tiers fix aggression across the whole draft, producing permanent lane-width differences.

Staggered escalation is preferable here because it preserves the draft arc that is this design's core contribution: wide-open early, concentrated late. Per-AI tiers (Proposal B) were rejected precisely because they do not solve the transition zone — moderate AIs still produce M10 streaks regardless of tier assignment. Staggered escalation with individual offsets keeps the arc intact while making the synchronized-escalation artifact undetectable.

**Recommended revision for simulation:** Apply per-AI phase offsets of {-1, 0, +1} picks, drawn at game setup. Keep the 7-AI structure for this slot (to isolate the escalation variable from the lane-count variable tested by Hybrid Y). Accept that Hybrid Y is the likely winner if signal reading matters more than contraction intensity — the simulation results will determine this directly.
