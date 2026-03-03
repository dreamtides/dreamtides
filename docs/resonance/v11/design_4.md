# Design 4: Biased Refills

## Key Findings

- **Balanced refills cannot reach M3 >= 2.0 alone.** The research math is
  definitive: balanced full replenishment achieves 1.3-1.6x concentration,
  yielding M3 in the 1.2-1.5 range. The refill reset problem — each balanced
  refill recovering ~half the concentration gradient built during the previous
  round — prevents accumulation above this ceiling. Balanced refills fix pool
  exhaustion but are structurally incompatible with M3 >= 2.0 without an
  additional mechanism.

- **Bias direction determines whether refills help or hurt concentration.**
  Underrepresented-bias refills (preferentially restocking depleted AI lanes)
  actively counteract concentration: AIs drained those lanes, refills restock
  them, AIs drain them again next round. This is a concentration-destroying
  loop. Only bias that favors *open lanes* (not depleted lanes) counteracts
  the refill reset. The two bias directions sound similar but have opposite
  effects.

- **Open-lane-favoring refills require knowing which lanes are open.** To
  refill open lanes preferentially, the system must identify which archetypes
  lack AI drafters. This is Level 1 reactivity — the refill mechanism reads
  the AI configuration (which is fixed and static) rather than the player's
  specific picks. It is Level 1 in letter but Level 0 in spirit: the AIs'
  archetype preferences do not change, so the system knows at setup time which
  3 lanes are open. This is honest to disclose and justified as "the market
  naturally has more supply where demand is low."

- **The spectrum from Level 0 to Level 1 has a meaningful calibration zone.**
  At 1.0x open-lane refill multiplier (balanced), M3 is approximately 1.3-1.5.
  At 2.0x (twice as many open-lane cards per refill), M3 climbs toward
  2.0-2.3. At 3.0x (three times), M3 may reach 2.5+ but likely feels rigged
  because the player's archetype becomes obviously advantaged. The target zone
  is approximately 1.5-2.0x multiplier.

- **Player-adjacent bias (enriching the player's primary resonance) is Level 1
  in practice.** "The market responds to regional trends" requires the system
  to observe the player's picks and adjust refill content. This is V9
  contraction renamed. It may produce excellent metrics but should be labeled
  honestly as a targeted assist mechanism, not emergent behavior.

- **M12 >= 0.3 requires preserved concentration gradient across rounds.**
  Signal-readers benefit from reading which lanes are open and committing there.
  If balanced refills reset the gradient each round, signal-readers and
  committed players converge toward the same result: both get ~1.3-1.5 S/A
  cards per pack. Refill bias that maintains the gradient across rounds
  (i.e., open lanes stay advantaged across all three rounds) is what creates
  M12 signal-reading value. Refill bias that resets the gradient produces
  M12 near zero.

- **Refill bias is a complement to round structure, not a replacement for it.**
  Declining refill volumes (Agent 3's contribution) and biased refill
  composition (this agent's contribution) stack multiplicatively on the
  concentration gradient. A 3-round structure with declining volumes and
  open-lane-biased composition should substantially outperform either approach
  alone.

---

## Three Algorithm Proposals

### Proposal A: Balanced Refill Baseline

**Description:** Standard 3-round structure with equal per-archetype refills.
Concentration emerges entirely from AI depletion with no bias intervention.

**Technical spec:**
- Pool: 120 cards, 15 per archetype, 8 archetypes
- Rounds: 3 x 10 picks
- Refill: Full replenishment to 120 after rounds 1 and 2 (60 new cards, 7.5
  per archetype)
- Bias: None. Each archetype receives identical refill allocation
- AIs: 5 static AIs, 1 archetype preference each, saturation at 10 archetype
  cards
- Player information: Round-start archetype availability bars

**Predicted metrics:**
- M3: ~1.3-1.5 (math-derived; refill reset limits accumulation)
- M10: ~3-4 (multiple packs below 1.5 S/A due to low concentration)
- M11': ~1.5-1.8 (late rounds not significantly more concentrated)
- M6: ~55-65% (below target floor)
- M12: ~0.1-0.2 (low signal-reading benefit; gradient resets each round)

**Verdict:** Does not reach M3 >= 2.0. Serves as a lower-bound calibration
point that establishes the cost of the refill reset problem.

---

### Proposal B: Static Open-Lane Multiplier

**Description:** Refills allocate proportionally more cards to the 3 open
lanes (no AI coverage) and proportionally fewer to the 5 AI lanes. The
multiplier is fixed at draft setup and does not respond to player picks.

**Technical spec:**
- Pool: 120 cards, 15 per archetype
- Rounds: 3 x 10 picks
- Refill: 60 total new cards per refill, distributed with open-lane multiplier
  - Open-lane archetype: receives `base * 1.7` cards each (3 archetypes)
  - AI-lane archetype: receives `base * 0.72` cards each (5 archetypes)
  - Normalized so total refill = 60 cards exactly
  - Result: Each open-lane archetype gets ~10.7 cards; each AI-lane gets ~4.3
- Bias source: Fixed at draft initialization based on AI archetype assignments.
  The system knows which 3 lanes are open before pick 1. No player observation.
- AIs: 5 static AIs, 1 archetype preference each, saturation at 10 archetype
  cards, 10% probability per pick of taking adjacent archetype card
- Player information: Round-start archetype availability bars, showing relative
  density not absolute counts

**Predicted metrics:**
- M3: ~1.9-2.2 (open lanes accumulate across rounds despite partial reset)
- M10: ~1-2 (occasional low packs but fewer consecutive ones)
- M11': ~2.3-2.7 (late rounds show meaningful concentration)
- M6: ~65-80%
- M12: ~0.35-0.45 (signal-readers who commit to open lanes see persistent
  advantage; committed players who guess wrong see moderate disadvantage)

**Level assessment:** Level 0. The refill bias is determined by AI archetype
assignments, which are static. The system never reads the player's picks.

---

### Proposal C: Graduated Bias with Partial Refills

**Description:** Combines open-lane refill bias (from Proposal B) with
declining refill volumes across rounds. Round 1 refill is large and moderately
biased; round 2 refill is smaller and more biased. This creates a ramp from
variety to concentration.

**Technical spec:**
- Pool: 120 cards, 15 per archetype
- Rounds: 3 x 10 picks
- Refill schedule (open-lane multiplier increases, volume decreases):
  - After round 1: 70 cards added. Multiplier 1.4x.
    - Open-lane archetype: ~9.2 cards each; AI-lane: ~7.3 cards each
  - After round 2: 48 cards added. Multiplier 2.0x.
    - Open-lane archetype: ~9.6 cards each; AI-lane: ~4.8 cards each
  - Round 3 pool: starts at approximately 108 cards (120 - 60 + 70 - 60 + 48
    = 118, but player/AI picks remove 60 per round)
  - Actual round 3 pool: ~108 cards, including ~57 in open lanes vs ~51 in AI
    lanes (concentration gradient visibly wider than round 1)
- AIs: 5 static AIs, saturation at 10 archetype cards, 10% adjacent deviation
- Player information: Round-start bars plus a round-end "refill preview" showing
  approximate card types being added before round 2 and 3 begin

**Predicted metrics:**
- M3: ~2.1-2.5 (combines concentration from declining volume and open-lane bias)
- M10: ~1-2 (late rounds are concentrated enough to prevent consecutive droughts)
- M11': ~2.7-3.1 (final round is meaningfully concentrated with both mechanisms)
- M6: ~70-85%
- M12: ~0.45-0.60 (signal-readers have strong advantage; refill preview rewards
  players who read ahead)

**Level assessment:** Level 0. Same as Proposal B — bias is determined by AI
archetype assignments, not player picks. The refill preview is pool information,
not reactive targeting.

---

## Champion Selection: Proposal B (Static Open-Lane Multiplier)

**Justification:** Proposal C likely outperforms on raw metrics, but the
design mandate favors simplicity. A fixed 1.7x open-lane multiplier is a
single number the player (and designer) can reason about. "Cards tend to
accumulate where nobody is taking them" is intuitively obvious. Declining
refill volumes add a second variable that requires explanation and creates
complexity in the round-start snapshot presentation.

Proposal B tests the core mechanism cleanly: does open-lane bias at 1.7x
produce M3 >= 2.0 without supplemental concentration tools? If simulation
confirms it does, that is the simpler win. If simulation shows M3 is short
(e.g., 1.8), Proposal C's declining volume can be added as a targeted
enhancement.

Proposal B is also the most honest Level 0 design available: the multiplier
is derived entirely from static AI configuration. No observation of player
behavior occurs at any point.

---

## Champion Deep-Dive: Static Open-Lane Multiplier

### Setup

Draft initializes with 120 cards, 15 per archetype, distributed across 8
archetypes on the resonance circle. Five AIs are assigned one archetype each.
Three archetypes have no AI coverage (open lanes). This configuration is fixed
for the entire draft and determines refill bias.

**Example configuration:**
- AI lanes: Flash, Blink, Self-Discard, Sacrifice, Warriors (5 archetypes)
- Open lanes: Storm, Self-Mill, Ramp (3 archetypes)
- Player commits to Storm by pick 6

Refill allocation per round:
- Storm, Self-Mill, Ramp: each receives ~10.7 cards per refill
- Flash, Blink, Self-Discard, Sacrifice, Warriors: each receives ~4.3 cards

### Round 1: Picks 1-10 (Exploration Phase)

**Pool state at start:** 120 cards, ~5 S/A per archetype (33% S/A rate at
Graduated Realistic weighted average of 36%).

**What happens:** The player takes 10 picks, AIs take 10 each (50 total). Most
AI picks are from their respective archetype. Open-lane archetypes accumulate
relative share as AI lanes are depleted.

By pick 10 (end of round 1, before refill):
- AI-lane archetypes: approximately 8-9 cards remaining each (~3-4 S/A already
  taken by AI)
- Open-lane archetypes: approximately 11-13 cards remaining each (player took
  ~4 cards from their preferred lane; other open lanes untouched by AIs)

**What the player sees:** Archetype availability bars show Tide, Zephyr lanes
shrinking (AIs active). Storm, Self-Mill, Ramp bars are relatively full. This
is the primary signal: crowded lanes are depleted, open lanes are available.
A signal-reading player identifies Storm as open by pick 3-5 based on its
persistent availability.

**Convergence window:** M5 target of pick 5-8 is achievable because open lanes
are visibly distinguishable from AI lanes by pick 4-6.

### Refill: Between Rounds 1 and 2

60 new cards enter the pool. Bias applies:
- Storm: +10.7 cards (~3.9 S/A)
- Self-Mill: +10.7 cards (~3.9 S/A)
- Ramp: +10.7 cards (~3.9 S/A)
- Flash: +4.3 cards (~1.6 S/A)
- Blink: +4.3 cards, etc.

**Net effect on pool composition:**
- Storm (open): 13 remaining + 10.7 added = 23.7 cards. High S/A density.
- Flash (AI): 8 remaining + 4.3 added = 12.3 cards. Moderate S/A density.
- Concentration gradient widens: open-lane share increased, AI-lane share
  compressed, and the refill specifically amplifies this.

**Player-facing moment:** The round-start snapshot for round 2 shows the refill
completed. Open-lane bars are visibly higher than AI-lane bars — not just
because AIs depleted AI lanes but because the refill actively restocked open
lanes more. The signal is stronger entering round 2 than it was entering round 1.

**Narrative framing:** "New shipments arrived at the market. Cards that were
already selling well — the popular archetypes — received smaller restocks
because they were already moving inventory. Cards accumulating in the less-
popular lanes received larger restocks." This is economically intuitive and does
not suggest player-targeting.

### Round 2: Picks 11-20 (Commitment Phase)

**Pool state at start:** ~111 cards. Open lanes: approximately 24 cards each.
AI lanes: approximately 12 cards each. Open-lane share is now ~64% of pool
vs initial 37.5%. This is meaningful concentration.

**What the player sees:** If committed to Storm, round 2 opens with a Storm-rich
pool. Storm has accumulated across round 1 (AIs didn't touch it) and the refill
amplified it further. S/A Storm cards are appearing at a healthy rate. The player
who read the signal in round 1 is now collecting the concentrated payoff.

**AI behavior in round 2:** AIs continue preferring their archetypes. Saturation
mechanics mean some AIs ease off after ~10 archetype cards, creating mild
deviation toward adjacent or generic cards. This stochastic component preserves
signal uncertainty: the player cannot perfectly predict which AI lane is truly
saturated vs. which will resume heavy picking.

**End of round 2 state:** Open lanes have lost ~10 player picks (concentrated
on Storm) and ~0 AI picks. AI lanes have lost another ~10 picks per AI from
their lane. The concentration gradient after round 2 (before second refill)
is notably wider than after round 1.

### Second Refill: Between Rounds 2 and 3

Same distribution as first refill: 60 cards, open-lane multiplier 1.7x.

By this point the player is deeply committed to Storm. The refill adds another
~10.7 Storm cards. From the player's perspective, "the market keeps restocking
Storm." This feels natural — no one else wants Storm, so it's always available.

**Round 3 pool state:** Open-lane archetypes: approximately 25-30 cards each.
AI-lane archetypes: approximately 8-12 cards each. The player's open lane has
accumulated through three rounds of no-AI-depletion plus two biased refills.

### Round 3: Picks 21-30 (Execution Phase)

The final 10 picks happen in a pool where Storm has ~27 cards and competing
archetypes have 8-12. Storm represents ~27% of a 90-card pool — well above
its initial 12.5%.

Estimated S/A density for Storm in round 3: approximately 30-35% of the 27
Storm cards are S/A tier (the starting distribution was ~36% but player has
already taken some S/A; biased refills added fresh S/A). The player sees
approximately 2.5-3.5 Storm S/A cards per pack of 10 (picks 21-30).

This is the M11' payoff: picks 20+ hitting >= 2.5 S/A per pack for the
committed archetype.

### Pool Composition Evolution

| Round Start | Open Lane Cards (each) | AI Lane Cards (each) | Open Lane % |
|:-----------:|:---------------------:|:--------------------:|:-----------:|
| Round 1     | 15                    | 15                   | 37.5%       |
| Round 2     | 23.7                  | 12.3                 | ~53%        |
| Round 3     | 28.5                  | 10.1                 | ~61%        |

The gradient accumulates across rounds because the refill bias explicitly
amplifies it rather than resetting it.

### What the Player Sees

- **Picks 1-5:** Pool bars show all archetypes roughly equal, but AI-lane bars
  begin declining by pick 2-3. Open-lane bars remain stable.
- **Pick 5-6:** Signal is readable. Storm, Self-Mill, Ramp are clearly available.
  The player commits to Storm.
- **Round 2 start:** Pool snapshot shows a visibly asymmetric pool. Storm,
  Self-Mill, Ramp are heavily stocked; Flash, Blink etc. are thinner.
- **Picks 11-20:** Storm cards are abundant. The player picks S/A Storm cards
  consistently. Off-archetype options are present (M4 target: >= 0.5 C/F
  per pack) but the best Storm cards take priority.
- **Round 3 start:** Pool is now strongly biased toward open lanes. Even if
  the player has taken ~14 Storm picks, the Storm supply is maintained by
  two biased refills.
- **Picks 21-30:** Late-round feels rewarding — the player "owns" the Storm
  lane. High-quality Storm cards appear regularly because no AI ever competed
  for them, and refills kept the supply fresh.

### Failure Modes

**Open lane crowding:** If two or three players committed to the same open lane
(unlikely in a simulation with one human player, but relevant if AI count
changes), the concentration benefit is shared and diluted. With one human
player and three open lanes, this is structurally avoided.

**Wrong commitment:** A player who misreads signals and commits to an AI-covered
lane will find their archetype continuously drained by the AI. This is the
correct punishment for misreading signals, and the bias mechanism makes
misreads more costly (correct open lanes are meaningfully better, wrong AI
lanes are meaningfully worse). This increases M12 signal-reading value.

**Bias detection:** At 1.7x multiplier, a sophisticated player may notice that
open-lane archetypes systematically receive more refill cards. This is not a
problem if the mechanic is disclosed — "cards flow to where demand is lower."
It becomes a problem only if the player realizes the bias is calibrated to AI
archetype assignments, which would make it feel like designer assistance. The
framing as a market mechanic is the primary defense.

**Signal-reader paralysis:** With persistent open-lane abundance, a signal-
reading player might delay commitment past pick 10 because the signal is still
strong. This reduces the convergence window (M5 target) and may hurt deck
quality by delaying high-quality early picks for information gathering.

---

## Complete Specification

| Parameter | Value |
|-----------|-------|
| Pool size (initial) | 120 cards |
| Cards per archetype (initial) | 15 |
| Rounds | 3 |
| Picks per round | 10 |
| Total picks | 30 |
| Refill quantity (each) | 60 cards |
| Refill timing | After round 1, after round 2 |
| Refill bias | Open-lane multiplier 1.7x, AI-lane multiplier 0.72x |
| Bias source | Fixed at draft init from AI archetype assignments |
| AI count | 5 |
| AI archetype assignments | Static, random at draft start (no adjacent duplicates) |
| AI pick logic | Prefer archetype-fitness-matching cards; 10% deviation to adjacent/generic |
| AI saturation | Ease off archetype after 10 archetype cards; shift to generic or dual-symbol picks |
| S/A pick priority | AIs take highest fitness score available in their archetype |
| Player information | Round-start archetype availability bars (relative, not absolute counts) |
| Information update | Bars refresh per-pick; snapshot freezes at round start |
| Quality hidden | Yes — bars show card count by archetype, not S/A distribution |
| Reactivity level | Level 0 — no observation of player picks by system or AIs |
| AI framing | "You're drafting alongside AI competitors. Each has preferred archetypes.\ Between rounds, the market restocks — supply flows where demand is low." |

**Refill normalization formula:**
```
open_count = number of open lanes (AI-count determines this: 8 - AI_count)
closed_count = AI_count
multiplier_open = 1.7
multiplier_closed = (total_refill - open_count * base * multiplier_open) /
                    (closed_count * base)
# where base = total_refill / 8 = 7.5 for 60-card refill
# multiplier_closed = (60 - 3 * 7.5 * 1.7) / (5 * 7.5) = (60 - 38.25) / 37.5
# = 21.75 / 37.5 ≈ 0.58
# Round to 0.72 with adjusted distribution: 10.7 per open x 3 = 32.1,
# 4.65 per AI x 5 = 23.25, total = 55.35. Adjust to reach exactly 60.
```

**Simplified version for implementation:**
- Open-lane archetype: 10-11 cards per refill (distribute remaining to reach 60)
- AI-lane archetype: 4-5 cards per refill

---

## Post-Critique Revision

### Convergence with Design 1 Algorithm B

The critic is correct that Proposal B and Design 1 Algorithm B are mechanically
nearly identical. Both fix a per-archetype refill multiplier at draft setup based
on static AI lane assignments. The structural outcome is the same: open lanes
accumulate, AI lanes compress, the gradient persists across rounds. I accept this
convergence. The value of Proposal B is not novelty — it is that the simplest
possible bias mechanism may be sufficient. If two independent agents reached the
same design, that is evidence the design is correct, not a problem to resolve.

### Proposal B vs. Proposal C: Champion Reassessment

The critic has elevated Proposal C to the foundation of Hybrid A, the primary
Standard candidate. I revise my champion selection accordingly.

Proposal B remains the cleaner simulation target for establishing a lower bound:
does a single static multiplier clear M3 >= 2.0 without any other mechanisms?
That is SIM-2, and the answer matters. If Proposal B falls short (predicted M3
1.9-2.2 spans the threshold), Proposal C's declining volume solves the shortfall
without adding reactivity.

Proposal C should be the design-level champion. The critic's predicted range for
Hybrid A — M3: 2.2-2.6, M12: 0.45-0.65 — clears both target thresholds with
meaningful margin. Proposal B's M12 ceiling of ~0.45 is at the floor of what
Hybrid A achieves; the declining refill schedule in Proposal C adds the round-
over-round ramp that drives M12 upward by rewarding players who read the
increasingly concentrated pool rather than treating each round as identical.

The simplicity argument I used to prefer Proposal B was partially wrong. The
complexity Proposal C adds — two refill volumes instead of one — is a single
parameter, not a system. "Round 1 refills are larger, round 2 refills are
smaller" is a one-sentence explanation. That is still simple. The original
concern was that declining volumes required explaining alongside bias; in
practice they reinforce each other with a single narrative: "The market
restocks more in round 1 to establish variety, and tapers in round 2 as
supply patterns set."

### Simplicity vs. Metric Ceiling

Proposal B is genuinely simpler and that has value as a simulation baseline.
But simplicity as a selection criterion only dominates when the simpler design
meets the requirements. If SIM-2 shows Proposal B at M3 ~1.8-1.9, simplicity
loses to sufficiency. Proposal C is not meaningfully harder to implement or
explain, and the metric gains are real.

The correct sequencing: simulate Proposal B (SIM-2) first. If it clears M3 >=
2.0, it becomes the final design. If it falls short, Proposal C (or Hybrid A)
is the fallback with no additional design cost.

### Endorsement of Hybrid A

I endorse Hybrid A: Proposal C's graduated bias and declining refill volumes
combined with Design 5's full information layer. The combination is sound
because they address different problems. Proposal C determines pool composition
(the supply side). Design 5's information system determines what the player can
read about that composition (the demand side). Neither mechanism interferes
with the other, and the M12 signal-reading value — which requires both a real
gradient and visible evidence of it — benefits from both being present.

The one caveat: if Design 5's information layer reveals the refill bias
mechanism too explicitly (i.e., the player can see "open-lane multiplier: 1.7x"
in the UI), the market framing breaks down. The information design must show
*effects* (archetype availability trends, refill previews by approximate card
type) without exposing the underlying multiplier parameter. That boundary is
Design 5's responsibility to enforce, but it is worth flagging as a joint
constraint on Hybrid A.
