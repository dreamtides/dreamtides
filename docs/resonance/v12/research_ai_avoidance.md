# Research: AI Avoidance in Competitive Drafting

## Question

How do human drafters read opponents and adjust strategy in competitive draft
formats, and how can AI drafters replicate this behavior? Specifically for V12:
in a face-up shared pool where picks are secret, how do depletion patterns
enable archetype inference, how quickly does inference become reliable, and how
should avoidance timing and strength be tuned?

---

## Finding 1: How Humans Read Tables in Shared-Pool Games

### The Two Signal Modes

Human drafters use two fundamentally different signal modes:

**1. Pack-passing signals (MTG booster draft):** You receive a pack that was
previously drafted from. Missing cards reveal what the player upstream valued.
If a pack has no white cards but otherwise looks intact, the player upstream is
likely drafting white. This is a strong, noisy signal: strong because the
missing slot is a deliberate choice, noisy because the previous drafter may
have taken a white card out of pure power, not archetype commitment.

**Skilled play:** In booster draft, signal reading works well from picks 5-8
because by that point you have seen 3-4 packs from each neighbor, and the
pattern of absences becomes readable. A single missing slot means little; a
consistent absence over 4 packs means someone upstream is committed.

**2. Face-up shared pool depletion (7 Wonders, Ascension, and V12):** You see
what is in the pool at all times but not who took what. Players infer intent
from which cards disappear, not from individual picks. This is weaker signal
per pick than MTG's pack-passing (you see the absence but don't know who caused
it), but it is continuous and cumulative.

**The key asymmetry:** In MTG, you see absences caused by one specific upstream
neighbor. In V12 with 5-7 AIs, any pick cycle removes 5-7 cards from the pool.
Attribution is confounded from the start — you cannot know if Ember card X was
taken by an Ember AI, a Zephyr AI opportunistically picking a useful filler, or
the player. This confounding is the central inference challenge.

### Shared-Pool Game Evidence

**7 Wonders:** This is the closest analog to V12. All players draw from a common
pool of cards across rounds. Players observe what resources their neighbors are
building and what card categories are thinning. Skilled players:
- Track which wonder categories (military, science, commerce) are depleting
  faster than expected
- Shift away from contested categories when they notice rivals accumulating the
  same type
- The avoidance is reactive but not immediate — players typically commit through
  1-2 rounds before pivoting

The documented strategy in 7 Wonders: *"If you see your neighbor is heavy
science by Round 2, don't fight them for science cards in Round 3 — the supply
has already been depleted by their Round 2 picks."* Avoidance at that point is
mostly futile anyway, but the lesson is that experienced players read the board
state (not individual picks) and adjust 2-4 picks after the pattern is clear.

**Ascension / Market-Row Games:** The continuous-replace market is informative
but less relevant. In Ascension, the market refreshes after each buy, so
depletion patterns are less persistent. Players observe what their opponent is
building (total accumulated cards visible) rather than what is disappearing from
the market. This is a different information model — in V12, what opponents have
accumulated is secret; only what remains in the pool is public.

**Sushi Go Pass-and-Pick:** Closer to MTG draft. Players pass hand subsets
around the table. Signals are from what was passed, not a shared pool. Less
relevant to V12's information model.

---

## Finding 2: MTG Signal Reading and its V12 Translation

### How MTG Signals Work

In MTG booster draft, the classic table-reading heuristic is:
- **Picks 1-2:** Open lane unclear; take the strongest card available
- **Picks 3-5:** If a color/archetype is consistently missing from passed packs,
  someone upstream is in it; avoid
- **Picks 6-8:** Commit to an open archetype; signals should be clear by now
- **Picks 9-15:** Confirmation phase; packs confirm or deny the read

The signal requires 3-5 consistent data points across different packs to be
actionable. A single absence is noise; five consecutive absences is signal.

### V12 Translation: Pool Depletion vs. Pack Absence

V12's information model inverts the MTG model. Rather than seeing what *was
taken* (empty slots in a passed pack), players see what *remains*. The
inference process is:

- **MTG:** "This pack had no blue removal, so someone upstream drafted blue
  removal." Direct attribution, one data point per pack rotation.
- **V12:** "There were 6 Ember cards in the pool two pick cycles ago; now there
  are 4. 2 Ember cards were taken in 2 pick cycles (12 total picks across all
  drafters). Is 2 Ember departures in 12 picks a signal or noise?"

The baseline Ember departure rate matters. If Ember is 15% of the pool and
12 picks were made, expected Ember departures = 12 × 15% = 1.8. Observing 2
Ember departures is within noise. Observing 6 Ember departures in 12 picks
(2x expected) is a meaningful signal.

**Conclusion for V12:** Depletion signals only become statistically meaningful
when one archetype depletes at 1.5-2x its expected rate. With 8 archetypes and
a pool of 120 cards, this requires roughly 4-6 pick cycles of observation before
inference is confident. This maps to approximately pick 4-6 of the human player's
draft — consistent with the orchestration plan's M14 target of picks 4-7.

---

## Finding 3: Pick Cycles Needed for Reliable Inference

### The Inference Math

With N drafters (5-7 AIs + 1 player), each pick cycle removes N cards.
Each archetype starts at approximately 1/8 of the pool.

For an AI drafting archetype X to become detectable from depletion:
- X depletes at the AI's pick rate for X plus random contributions from other
  drafters' incidental picks
- Other archetypes deplete at random rates driven by their fraction of the pool

**Key calculation:** With 6 total drafters, one archetype-committed drafter
takes roughly 1 card/pick from their archetype. The pool removes 6 cards/pick
total. If archetype X is 15% of the pool, random picks remove 0.9 X-cards/pick
in expectation. One committed AI adds 1.0 X-cards/pick → observed rate is 1.9x
expected. After 3 pick cycles (18 total picks), expected X-departures = 5.4 vs.
observed ~9 if one AI is committed.

**Effect of AI count:** More AIs = more confounding. With 7 AIs instead of 5:
- More total picks per cycle → faster depletion across all archetypes
- Harder to attribute specific-archetype depletion to a single committed drafter
- Signal-to-noise ratio decreases

**Practical implications:**
- With 5 AIs: reliable inference requires approximately 3-5 pick cycles (picks
  3-5 of the player's draft). At that point, an archetype being drained at 2x
  expected rate is detectable with reasonable confidence.
- With 7 AIs: inference requires 5-7 pick cycles (picks 5-7). More drafters
  create more cross-archetype noise.
- The player's own picks are also confounded with AI picks in the overall
  depletion signal. This creates symmetric uncertainty — AIs trying to infer the
  player's archetype face the same statistical challenge the player faces.

**For V12 AI design:** The AI should not begin avoidance based on a single
pick's depletion. A minimum window of 3-4 pick cycles is needed before any
archetype depletion pattern is reliable. Avoidance should ramp as evidence
accumulates, not switch instantly.

---

## Finding 4: Avoidance Timing and Dynamics

### Early vs. Late Avoidance

**Early avoidance (picks 3-5):** AIs react quickly to depletion patterns. The
benefit is that AIs preserve the player's S/A cards from the very first picks
where the player is establishing their archetype. The cost is high false-positive
rate: with few pick cycles of data, the AI may avoid an archetype the player is
NOT drafting, leaving that archetype's S/A cards in the pool for the player to
cherry-pick — a neutral or even positive outcome from the player's perspective.

**Late avoidance (picks 8-10):** AIs wait until the signal is strong. Lower
false-positive rate but significant S/A loss in picks 1-7 before avoidance
begins. In a 30-pick draft, picks 1-7 are about 23% of the draft — if AIs take
S/A cards during this window, the player's archetype is meaningfully depleted
before avoidance preserves anything.

**The asymmetry:** False positives in early avoidance (AI avoids wrong
archetype) leave the player with extra supply in both the falsely avoided
archetype AND their real archetype. This is net positive for the player. False
negatives in late avoidance (AI takes S/A before pivoting) are net negative —
irreversible depletion. This asymmetry suggests **earlier is safer than later**
for the player's experience.

**Gradual ramp is the right model:** Starting with low avoidance weight at pick
3 and ramping to full avoidance by pick 10-12 captures the benefits of early
detection without over-committing to noisy early signals. A sigmoid ramp
matches how confidence should naturally accumulate.

### Impact on Draft Dynamics

V12's orchestration plan identifies an important cascade effect: when AIs avoid
the player's archetype, they shift their demand toward adjacent archetypes. This
creates secondary depletion patterns in neighboring archetypes (sharing a
resonance symbol with the player's). If the player chose a Flash/Zephyr
archetype and all AIs avoid Flash, those AIs shift toward Ramp/Zephyr and
Blink/Ember — slightly depleting cards the player might want as incidental
picks.

This cascade is mild but real. The practical implication: avoidance should be
archetype-specific, not resonance-symbol-wide. An AI avoiding "Flash" should
continue to freely draft non-Flash Zephyr cards.

---

## Finding 5: The Surveillance Boundary

### What Feels Acceptable vs. Invasive

In competitive drafting, "reading the table" is universally accepted as a skill.
What feels invasive or unfair:

**Acceptable (Level 0-1):**
- Reacting to what has left the pool (public information, same data available
  to everyone)
- Responding to aggregate depletion trends over multiple pick cycles
- Adjusting archetype priority based on observed supply levels
- Symmetric behavior: AIs behave exactly as a skilled human opponent would
  behave observing the same information

**Borderline (Level 1-2):**
- Using the player's visible resonance signature (computed from their drafted
  cards' symbols) as a direct inference input. This is derived from public
  information (what cards are gone from the pool) but requires the AI to
  aggregate information that the player would find opaque. If the AI explicitly
  reasons "the player has picked 3 Ember cards and 1 Stone card, so they are
  probably Blink or Storm," this may feel uncomfortably precise.

**Unacceptable (Level 2+):**
- Directly accessing the player's pick history (which specific cards the player
  took). This is private information — even though depletion is public, *who*
  took each specific card is secret.
- Modifying behavior based on the player's internal evaluation or commitment
  signal. The AI should not know whether the player is committed or exploring.
- Retroactively attributing depletion to the player when multiple drafters
  could explain the same pattern.

**The practical boundary for V12:** AI inference must be based on aggregate
pool-state changes, not on specific card-level attribution to the player. The AI
can observe "Ember S/A cards are depleting fast" but should not observe "the
player took Ember card #47 at pick 3." The first is public pool information; the
second requires secret information.

The face-up pool in V12 makes this boundary clear and honest: both the player
and the AIs observe the same pool state. Neither sees who took what. The
information is symmetric. AIs that use only pool-state information are playing
by the same rules as the player. This is the "public information honesty
criterion" from Key Principle 4.

---

## Finding 6: Avoidance Strength Calibration

### The Right Avoidance Magnitude

"Full avoidance" (90% weight reduction on the player's archetype) is appropriate
only after confident inference. Premature full avoidance on noisy early signals
produces erratic AI behavior and over-protects the player before they have
committed. Graduated avoidance is more realistic and better calibrated.

**Recommended scale based on inference confidence:**
- 0-20% archetype depletion signal (picks 1-3): no avoidance weight change
- 20-50% depletion signal (picks 3-6): 20-40% weight reduction
- 50-80% depletion signal (picks 6-10): 40-70% weight reduction
- 80%+ depletion signal (picks 10+): 70-90% weight reduction

"Inference confidence" here means the ratio of observed-to-expected depletion
for the suspected player archetype. At 2x expected depletion rate sustained
over 4+ pick cycles, confidence should be high.

**Saturation behavior compounds with avoidance:** A realistic AI also eases
off an archetype when it has collected enough cards (D3's saturation finding
from V10: after ~12 archetype cards, AIs naturally slow their drafting of that
archetype). Combined with avoidance, this means late-draft AIs have two reasons
to reduce pressure on the player's archetype: explicit avoidance logic AND
natural saturation. This compound effect should produce strong concentration
even with moderate avoidance weights.

---

## Connections to Other Research

**Agent B (Pool Contraction):** Avoidance timing directly determines how many
S/A cards survive into the late pool. Early avoidance (pick 3) preserves ~7-10
more S/A cards for the player than late avoidance (pick 8). This is the primary
input Agent B needs for contraction trajectory modeling. The critical question:
what is the S/A count remaining in the player's archetype at pick 10, given
avoidance starting at pick 3 vs. pick 8? That count drives the late-pool
density calculation.

**Agent C (Concentration Math):** The inference reliability finding (3-5 pick
cycles for 5 AIs, 5-7 cycles for 7 AIs) sets the earliest pick at which
avoidance can be confident. Agent C's models should parameterize avoidance onset
pick (3, 5, 8) as a sensitivity variable.

**Round 2 Agents:** The surveillance boundary finding bears directly on the
AI Avoidance Model table in Variable 1. Model D (Immediate Avoidance, pick 2+)
may cross into the borderline zone — two pick cycles is insufficient statistical
evidence for archetype inference. Model C (Gradual Avoidance, ramp from pick 3)
is best aligned with the inference timeline. Model B (Delayed Avoidance, pick
8+) is late enough to incur meaningful S/A loss.

---

## Summary Findings

1. **Inference timeline:** With 5-7 AIs sharing a face-up pool, reliable
   archetype inference from depletion patterns requires 3-5 pick cycles (picks
   3-6 for a 5-AI table; picks 5-8 for a 7-AI table). The M14 target of picks
   4-7 is well-calibrated.

2. **Avoidance onset:** Gradual ramp starting pick 3, reaching full weight
   by pick 10-12, is the correct model. Early onset with low weight avoids
   both the false-negative cost of late avoidance and the false-positive
   overreaction of immediate full avoidance.

3. **Avoidance benefit asymmetry:** False positives (AI avoids wrong
   archetype) are neutral-to-positive for the player. False negatives (AI
   takes S/A before pivoting) are negative. Earlier-than-necessary avoidance
   is safer than later.

4. **Surveillance boundary:** AIs must infer from aggregate pool-state
   changes only, not from individual card-level attribution to the player.
   The face-up pool enforces this naturally: both player and AIs see the same
   pool state, neither sees who took what.

5. **Confounding at high AI count:** 7 AIs significantly increase depletion
   confounding. Agent 5's 7-AI configuration should expect slower inference
   and longer avoidance ramp than 5-AI designs, but stronger absolute
   avoidance once triggered (7 AIs fully avoiding = zero competition).

6. **Saturation compounds avoidance:** AIs that have already collected ~12
   archetype cards will naturally draft their archetype more slowly regardless
   of explicit avoidance logic. Late-draft compound behavior (saturation +
   avoidance) should produce strong concentration even with moderate avoidance
   weights.

---

## Open Questions

- **How precisely can AIs isolate the player's depletion from other AIs'
  depletion?** With 5 AIs each taking 1 card/pick from their archetype, and
  all archetypes overlapping in resonance symbols, it is unclear how much
  archetype-level attribution is possible. A simulation-time sensitivity test
  should compare naive depletion inference (count all departures per archetype)
  vs. symbol-weighted inference (weight by visible resonance symbol match). How
  much does the method of inference affect avoidance accuracy?

- **What happens when the player pivots?** If the player switches from archetype
  X to archetype Y at pick 10, AIs that have been avoiding X since pick 5 will
  lag in detecting the pivot. The pivot detection lag could be 3-5 more pick
  cycles. Do AIs need explicit pivot detection, or does gradual avoidance
  naturally decay if X-depletion slows?

- **Is the avoidance signal visible to the player?** In a face-up pool, the
  player may notice that their archetype's S/A cards are persisting longer than
  expected. This is an advanced signal that rewards careful observation — is it
  a skill axis to be amplified, or an obvious tell that makes avoidance feel
  too mechanical?
