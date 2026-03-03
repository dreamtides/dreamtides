# Design 6: Hybrid and Novel Approaches

## Key Findings

- **The contraction gap is the defining constraint.** V9's 12% contraction requires removing ~38 cards per pick. Natural AI drafter counts (3-5) with 1 card/pick produce only 1-3% contraction. Bridging this gap requires either many AIs, multi-card AI picks, supplemental culling, or reframing what "AI drafting" means at the structural level.

- **A pod-based draft table is the most natural framing for high AI counts.** Real booster drafts seat 8 players who each open a pack, pick one card, and pass the rest. This structure inherently removes 7 cards per pack-round per player, achieving high contraction without any single AI taking multiple cards per pick. The "pass packs around the table" framing makes 7 AIs + 1 player feel expected rather than excessive.

- **Personality variation across AIs solves the "fixed puzzle" failure mode without requiring reactivity.** Level 0 AIs with different aggression levels, focus depths, and occasional off-archetype power picks create unpredictable-feeling drafts within a fully predetermined system. The variety comes from AI personality, not AI reaction.

- **Pair-affinity-aware AI pick logic is non-negotiable for M11 >= 3.0.** The V9 finding that 8-bit pair-affinity is required for M11 translates directly: AIs must prefer their archetype's home cards over sibling cards, using the same pair-affinity scores. An AI that picks only by visible resonance caps M11 at ~2.1.

- **The "inverse contraction" direction is actually an advantage for the AI framing.** V9 removes cards irrelevant to the player (bottom-up culling). AIs remove cards relevant to themselves (top-down picking). The net effect on the player is the same: other-archetype cards thin out. But the AI framing is more intuitive -- "Warriors AI took the Warriors cards" is clearer than "the game removed non-Warriors cards."

- **Rivalry and mutual avoidance between AIs create emergent lane width without player-reactivity.** If two AIs share a resonance (e.g., Warriors and Sacrifice both want Tide cards), they compete with each other, thinning that resonance quickly. AIs on an uncontested resonance leave their pool richer. The player benefits from reading which resonances are over- vs. under-drafted by AIs -- a pure Level 0 signal.

- **Imperfect AI picks ("splashing" and "power-picking") solve both the verisimilitude problem and the M10 floor problem.** An AI that occasionally takes a high-power off-archetype card instead of a mediocre on-archetype card leaves more on-archetype cards in the pool, functioning as a natural floor slot without explicit mechanics.

---

## Three Algorithm Proposals

### Proposal A: "The Full Table" (Pod Draft Simulation)

**Player description:** "You're drafting at a table with 7 other players. Each round, everyone opens a pack and picks a card, then passes the remaining cards."

**Technical description:** 8-seat pod draft. Each round, 8 packs of 4 cards are generated from the pool. Each drafter (7 AIs + player) receives a pack, picks 1 card, passes remaining 3 cards to the next seat. Each pack circulates until empty (4 picks per pack). AIs use pair-affinity scores for pick evaluation. Each game randomly assigns 7 of 8 archetypes to AIs, leaving one archetype uncontested. AIs have personality profiles (aggression 0.6-0.95, focus 0.7-1.0) that vary per game seed.

**AI drafter behavior:** Each AI evaluates available cards as: `score = affinity[my_archetype] * focus + card_power * (1 - focus)`. High-focus AIs are archetype-purists; low-focus AIs splash for power. Each AI picks the highest-scoring card. Because packs circulate, AIs in adjacent seats create natural passing patterns -- the player learns to read "upstream" AIs' preferences from what they pass.

**Predicted metrics:** M3 ~2.5, M10 ~3.0, M11 ~3.0, M6 ~80%. The full table removes 7 cards per pack-round (28 cards per full 4-pick circulation), achieving ~8% contraction per round. M11 is borderline because pack circulation dilutes archetype targeting -- AIs take the best card available, not necessarily from their archetype.

### Proposal B: "The Personality Draft" (Static AIs with Diverse Personalities)

**Player description:** "Each game, several AI opponents join the draft. Each has their own strategy -- some are aggressive specialists, others are flexible generalists."

**Technical description:** Each game seats 9 AIs from a roster of personality templates. Each AI is assigned an archetype and one of 4 personality types: Specialist (picks home cards 90% of the time, high affinity threshold), Opportunist (picks home 70%, takes power cards 30%), Hoarder (always picks highest-affinity card regardless of power, creating deep but narrow competition), and Novice (picks semi-randomly with mild archetype preference, functioning as weak contraction). AIs pick 1 card per player-pick-round, predetermined at draft start using pair-affinity scores. Pool composition per game: 6-7 archetypes contested (2-3 AI pairs share resonances), 1-2 archetypes lightly contested or open.

**AI drafter behavior:** All picks predetermined (Level 0). The Specialist creates strong, readable lane signals. The Opportunist creates mild noise (occasionally takes cross-archetype power cards). The Hoarder drains deep into one archetype quickly. The Novice provides light contraction without strong signals. Per game, the mix of personalities creates unique drafting environments.

**Predicted metrics:** M3 ~2.4, M10 ~3.5, M11 ~2.8, M6 ~82%. With 9 AIs taking 1 card each, contraction is ~2.8% per round -- far below V9's 12%. M11 falls short because insufficient contraction fails to concentrate the pool for late-draft archetype density.

### Proposal C: "The Syndicate Draft" (AI Crews with Bulk Acquisition)

**Player description:** "You're competing against drafting syndicates -- organized groups that buy up cards in bulk. Each syndicate targets a specific strategy, and they take multiple cards each round."

**Technical description:** 5 AI "syndicates" each take 4 cards per player-pick (a full pack equivalent). Each syndicate targets one archetype using pair-affinity pick logic. Per game, 5 of 8 archetypes are assigned to syndicates, leaving 3 archetypes uncontested. Syndicates pick the top 4 cards by affinity for their archetype from the remaining pool. All picks predetermined (Level 0). Two syndicates may share a resonance, creating inter-syndicate rivalry that thins shared-resonance cards faster.

Supplemental "market noise" culling removes 2 additional random cards per round (framed as "cards sold to collectors" or simply folded into syndicate behavior as a 6th passive syndicate taking 2 cards). Total removal: 5*4 + 2 = 22 cards per round. At pick 4 (pool ~340), this is ~6.5% contraction. To reach 12%, syndicates escalate: from pick 10 onward, each syndicate takes 6 cards per round (30 total + 2 noise = 32, ~12-15% of a mid-draft pool of ~220).

**AI drafter behavior:** Each syndicate has a "demand curve" -- early in the draft, they take top-affinity home cards (cream-skimming). Mid-draft, they take any card with affinity > 0.5. Late-draft, they take anything with affinity > 0.3, including bridge cards. This mirrors how real collectors operate: best cards first, then filling gaps. The escalating pick rate simulates increasing desperation / market thinning.

**Predicted metrics:** M3 ~2.65, M10 ~2.5, M11 ~3.2, M6 ~85%. The bulk acquisition matches V9's contraction intensity. The 3-archetype-open design ensures the player always has viable lanes. Inter-syndicate rivalry on shared resonances creates readable signals.

---

## Champion Selection: Proposal C -- "The Syndicate Draft"

**Justification:** Proposal A (Full Table) is the most realistic draft simulation but its pack-passing circulation dilutes archetype targeting -- AIs pick the best available card from whatever pack reaches them, not necessarily from their archetype. This produces weaker lane signals and borderline M11. Proposal B (Personality Draft) has the best verisimilitude but cannot achieve sufficient contraction with only 1 card per AI per round; the math simply does not work for M11 >= 3.0 without either many more AIs or supplemental culling that breaks the narrative.

Proposal C solves the core contraction problem head-on. Five syndicates taking 4-6 cards each achieves V9-equivalent contraction while remaining narratively coherent: "organized buyers purchasing in bulk" is a natural real-world analog. The 5-syndicate / 3-open-archetype structure gives the player genuine lane choice. The escalating demand curve produces the convergence ramp V9 achieved through percentage-based contraction. The inter-syndicate rivalry on shared resonances creates organic lane signals without any player-reactivity.

---

## Champion Deep-Dive: The Syndicate Draft

### How It Works

**Setup (per game):** The system randomly selects 5 of 8 archetypes and assigns each to a syndicate. Each syndicate has a personality seed that controls its aggression (0.7-0.95) and focus (0.75-0.95). All syndicate picks for the entire draft are computed before the player's first pick (Level 0, fully predetermined). The system uses each card's pair-affinity scores as the AI evaluation function.

**Per round (player picks 1 card):** Before the player sees their pack, syndicates collectively remove cards from the pool. Early draft (picks 1-8): each syndicate removes 4 cards. Mid draft (picks 9-18): each syndicate removes 5 cards. Late draft (picks 19-30): each syndicate removes 6 cards. Cards are removed in descending order of affinity for each syndicate's archetype. Ties broken by card power, then random.

**Pack construction:** After syndicate picks, the player's 4-card pack is drawn from the remaining pool. One slot is a "floor" card: the highest-affinity card for the player's inferred archetype that no syndicate wanted (affinity < 0.3 for all active syndicates). This naturally produces high-quality archetype cards -- the ones syndicates passed over because they belong to an uncontested archetype.

**Convergence ramp:** As syndicates remove more cards in later rounds, the remaining pool concentrates toward the 3 uncontested archetypes plus generics. A player in an uncontested lane sees increasingly dense archetype packs. A player competing with a syndicate sees thinner but still viable packs (the syndicate takes the best, but bridge cards and secondary-affinity cards remain).

### What the Player Sees vs. What the AIs Do

**Player sees:** 4-card packs that start diverse (picks 1-5, pool barely thinned) and gradually concentrate toward certain archetypes. By pick 10, the player notices that some archetypes appear frequently (uncontested) while others appear rarely (syndicate-drafted). This is the lane signal. The player is never told which syndicates exist -- they infer from card availability.

**AIs do:** Each syndicate mechanically removes its best available cards from the pool before each player pack. Syndicates sharing a resonance (e.g., Warriors syndicate and Sacrifice syndicate both wanting Tide cards) compete with each other, draining Tide cards faster than other resonances. This creates an emergent signal: Tide cards are scarce (two syndicates competing), while Ember cards might be abundant (only one or zero syndicates on Ember).

### Example Draft Trace

**Game setup:** Syndicates on Warriors, Sacrifice, Storm, Self-Mill, Ramp. Open archetypes: Flash, Blink, Self-Discard.

**Picks 1-4:** Pool is 360. Each syndicate takes 4 cards (20 total removed). Player sees diverse packs. Tide cards are slightly underrepresented (two syndicates drafting Tide). Zephyr cards moderately available (Ramp syndicate takes some). Player notices: many Ember cards available, few Tide cards late in early packs. Signal: Ember archetypes (Flash/Blink) are open.

**Picks 5-8:** Player commits to Blink (Ember/Zephyr). Pool ~280. Syndicates continue removing 4 each (20/round). Player packs now skew toward Ember -- most Ember cards survive because only Storm syndicate (1 of 5) targets Ember. Blink cards flow freely: no syndicate assigned to Blink. Average 2.3 S/A Blink cards per pack.

**Picks 12-18:** Pool ~160. Syndicates escalate to 5 cards each (25/round). Tide cards nearly exhausted (Warriors + Sacrifice syndicates have taken 80+ Tide cards between them). Remaining pool is rich in Ember, Stone (Self-Discard is open), and generics. Player's Blink packs average 2.8 S/A cards.

**Picks 20-28:** Pool ~70. Syndicates take 6 each (30/round, but pool minimum prevents over-depletion). Remaining cards are heavily Ember/Zephyr/generic. Player sees 3.1+ S/A Blink cards per pack. Floor card ensures at least one strong Blink option even in unlucky draws.

### Failure Modes

1. **Player drafts a syndicate's archetype.** If the player goes Warriors against the Warriors syndicate, both compete for Tide cards. The player sees fewer Warriors cards (~1.5 S/A per pack instead of 2.5+). The deck is weaker but playable -- bridge cards and secondary-resonance cards survive. This is the intended "contested lane" experience.

2. **Two syndicates on the same resonance exhaust it too fast.** Warriors + Sacrifice syndicates could drain all Tide cards by pick 15. Mitigation: syndicates prefer home-affinity cards (> 0.7) before bridge cards (0.4-0.6). Bridge cards survive longer in the pool. Also, syndicates eventually run out of high-affinity targets and start taking lower-affinity cards, slowing the drain.

3. **Uncontested archetypes are too generous.** With 3 archetypes open and no competition, the player may always find a deep lane with no effort. Mitigation: even open archetypes share resonances with contested ones. Flash (Zephyr/Ember) is open, but Ramp syndicate takes Zephyr cards, thinning Flash's secondary resonance. No archetype is fully uncontested at the resonance level.

4. **Narrative stretch.** "Syndicates buying cards in bulk" is slightly less intuitive than "other players at the table." However, it maps to real-world analog (card shops, speculators buying out stock) and avoids the unrealism of 10 individual players each taking 1 card.

---

## AI Drafter Specification

| Parameter | Value |
|-----------|-------|
| Number of syndicates | 5 per game |
| Archetype assignment | Random 5 of 8, no duplicates |
| Open archetypes | 3 per game (unassigned) |
| Pick logic | Descending pair-affinity for assigned archetype |
| Cards per syndicate per round | 4 (picks 1-8), 5 (picks 9-18), 6 (picks 19-30) |
| Total AI removal per round | 20 / 25 / 30 (escalating) |
| Equivalent contraction rate | ~6% early, ~12% mid, ~18% late |
| Reactivity level | Level 0 (fully predetermined) |
| Aggression (per syndicate) | 0.7-0.95 (randomized per game seed) |
| Focus (per syndicate) | 0.75-0.95 (higher = more archetype-pure picks) |
| Personality noise | 10-20% of picks are off-archetype power picks (card power > 7.0 regardless of affinity) |
| Floor card | 1 per pack: highest-affinity card for player's inferred archetype among cards with < 0.3 affinity for all active syndicates |
| Pool minimum | 25 cards (syndicates stop picking) |
| AI information | Pair-affinity scores (same 8-bit data as V9 Hybrid B) |

---

## Post-Critique Revision

### 1. Is Syndicate Draft Equivalent to Hybrid Y?

The critic's equivalence claim is largely correct and I accept it. The mathematical skeleton of Proposal C -- 5 entities, escalating removal per entity, 3 open archetypes, pair-affinity pick logic, Level 0 -- is structurally identical to Hybrid Y (D1 + D4). Renaming "5 AIs with escalating picks" as "5 syndicates with escalating bulk acquisitions" changes the surface vocabulary, not the algorithm. I cannot honestly argue otherwise.

The one structural difference that survives scrutiny is the floor card mechanic: Proposal C explicitly reserves one pack slot for the highest-affinity uncontested card the syndicates passed over. Hybrid Y as specified in the critic's write-up does not include this mechanism. Whether this meaningfully improves M10 or is a redundant complication is a simulation question, not a design claim I can defend a priori.

### 2. What Does the Syndicate Framing Add?

Less than I thought. The original argument was that "syndicates buying in bulk" is a natural real-world analog that makes multi-card-per-round removal feel coherent. The critic is right that this fails: 5 entities each removing 4-6 cards per round before the player sees their pack does not feel like a draft table. It feels like a market mechanism, because it is one. The narrative does not earn its complexity.

The one genuine framing benefit is the demand curve language -- "cream-skimming early, gap-filling late" -- which gives an intuitive vocabulary for why pool concentration ramps over the draft. But this is a description of the escalation mechanism, not a unique feature of the syndicate concept. The same vocabulary applies to "AIs getting more desperate as the draft progresses."

### 3. Should the Design Survive as a Simpler Hybrid Y Variant?

No. The right move is to fold this design into Hybrid Y rather than preserve it as a separate entry. Hybrid Y is already in the simulation queue. If it passes, the escalating 5-AI / 3-open-lane structure is validated. Adding a sixth simulation slot for Syndicate Draft -- which differs from Hybrid Y only in narrative framing and the floor card -- consumes a slot without generating independent information. The critic made the correct call dropping it.

If the floor card mechanism proves valuable, it can be tested as a post-simulation enhancement to whichever algorithm passes, rather than as a design in its own right.

### 4. What Should Be Preserved?

One mechanism is worth carrying forward into whichever hybrid advances: the inter-syndicate rivalry on shared resonances as an emergent signal. Proposals C's observation that two AIs sharing a resonance (Warriors + Sacrifice both taking Tide cards) create a detectable scarcity signal is not present in D1 or D4 as written. This resonance-level competition is a natural consequence of pair-affinity pick logic across multiple AIs, but it is worth naming explicitly in the final algorithm specification so the signal can be tested and tuned. The design team should verify that resonance-level scarcity is legible to players reading pack contents -- and if it is, it is a genuine contribution from this proposal regardless of what the outer framing is called.

The floor card mechanic is also worth preserving as an optional enhancement, with the caveat that it adds implementation complexity and the "player's inferred archetype" inference requires its own logic that could misfire early in the draft.

Everything else -- the syndicate framing, the demand curve language, the bulk acquisition narrative -- should be retired. Hybrid Y with pair-affinity pick logic and explicit resonance-competition tracking is the correct inheritor of this design's mathematical structure.
