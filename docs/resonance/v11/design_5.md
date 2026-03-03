# Design 5: Player Information Systems

## Key Findings

- **The strategic sweet spot is partial information about pool state, not
  individual card information.** Archetype-level availability bars communicate
  direction without resolving the pick. Eight archetypes with overlapping
  resonance symbols (Tide/Stone/Ember/Zephyr) ensure that even knowing "Tide
  cards are available" still requires a decision between Sacrifice/Abandon and
  Warriors/Midrange.

- **Quality is hidden even when quantity is visible.** Showing relative
  archetype counts masks the critical variable — which specific cards in that
  archetype are S/A tier. A "full" Tide bar could be full of C/F filler. This
  structural gap is what preserves the skill axis even under explicit pool
  information.

- **Round-start snapshots are time-bounded design assets.** A snapshot taken
  at refill is accurate for picks 1-3 of a new round and becomes increasingly
  stale as AIs draft. This matches the natural urgency arc: information is
  richest at the moment of recalibration and noisiest at round's end. It
  rewards planning, not just reaction.

- **Depletion trend indicators are the closest substitute for human
  table-reading.** A shrinking archetype bar implies agency — someone is taking
  those cards. This is what "reading the table" means when there is no pack to
  read. Crucially, trend direction (shrinking fast vs. stable) is informative
  without being perfectly predictive.

- **AI activity hints by archetype category, not AI identity, hit the right
  calibration.** "A Tide card was taken" vs. "AI 3 took a Tide card" — the
  former reads like pool-state inference (what you would deduce from a depleting
  Tide bar); the latter reads like tracking a state machine. Category-level
  hints give signal-readers a genuine advantage without making the draft feel
  like solving a lookup table.

- **Refill preview creates second-order skill at the cost of early commitment
  pressure.** Knowing what the next refill contains rewards players who can plan
  multi-round arcs, which is a real skill. The downside: players who see a
  strong refill incoming may defer commitment, weakening M5 (convergence pick
  target: 5-8). Timing the preview to end-of-round (not beginning) partially
  mitigates this.

- **Information and M12 are structurally linked but not the same thing.**
  M12 measures signal-reader advantage over committed players. Information
  creates M12 only when (a) the information is readable, (b) signal-readers
  act on it, and (c) committed players cannot access the same advantage by
  ignoring it. Over-explicit information erases M12 by making every player a
  signal-reader who just executes a lookup — committed players lose their
  comparative advantage without signal-readers gaining more value.

---

## Three Algorithm Proposals

### Proposal A: Static Bars Only

**Description:** Show archetype availability bars (relative counts, updated
per pick) throughout the draft with no additional information.

**Technical spec:** Eight bars, one per archetype, showing relative card count
vs. maximum. Bars update after each pick cycle (player + AI picks resolve).
No round-start snapshot, no trend indicators, no refill preview, no AI hints.
Bars display symbolic resonance labels (Tide, Stone, Ember, Zephyr) not
archetype names, so the player must still infer which specific archetype within
a symbol to target.

**Predicted metrics:**
- M3: 2.0-2.2 (bars give signal-readers a meaningful but not decisive edge)
- M10: 2.5-3.0 (concentration remains high enough to avoid consecutive bad packs)
- M11': 2.3-2.6 (late-draft bars are accurate and concentrated, aiding final picks)
- M6: 65-80% (moderate archetype concentration)
- M12: 0.2-0.3 (bars alone give only a small signal-reader advantage; committed
  players track their lane naturally and don't need bars)

**Assessment:** Functional but undershoots M12 target. Bars reward
signal-readers minimally because committed players don't need them — they
already know their lane.

---

### Proposal B: Bars + Round-Start Snapshot + Depletion Trends

**Description:** Combine three information channels — bars, a round-start
composition snapshot, and live depletion trend indicators — into a layered
display that rewards both planning (snapshot) and in-round reading (trends).

**Technical spec:**
- Availability bars: As in Proposal A, updated per pick cycle.
- Round-start snapshot: At the beginning of each round (after refill resolves),
  show a textual summary of pool composition: approximate total cards per
  resonance symbol and a relative quality indicator (approximate proportion of
  high-impact cards visible through bar height, not exact counts). Snapshot
  does not update mid-round; it becomes a reference against which depletion is
  measured.
- Depletion trend: Each bar displays a subtle directional indicator — a small
  arrow showing whether that archetype's count is shrinking faster or slower
  than average since round start. "Depleting fast" = someone is drafting it.
  "Stable" = available. Trend updates per pick cycle alongside bars.
- Refill preview: Not shown. Players infer next round from current round trends.

**Predicted metrics:**
- M3: 2.1-2.4
- M10: 2.0-2.5
- M11': 2.5-3.0
- M6: 68-82%
- M12: 0.35-0.5 (trend indicators strongly reward signal-readers; committed
  players gain nothing from trends in their committed lane, so the gap widens)

**Assessment:** Hits M12 target. Three-layer information is at the edge of
cognitive complexity but each layer has a different time horizon (snapshot for
planning, bars for current state, trends for inferring AI behavior), so they
serve distinct purposes without redundancy.

---

### Proposal C: Bars + Snapshot + Category-Level AI Pick Hints

**Description:** Replace depletion trend indicators with explicit AI pick
hints at archetype category level. After each pick cycle, show which resonance
category (not which AI, not which card) was most recently taken.

**Technical spec:**
- Availability bars: As in Proposal A.
- Round-start snapshot: As in Proposal B.
- AI pick hint: After each AI pick resolves, display a brief log entry at the
  bottom of the information panel: "Tide card drafted." The log shows the
  most recent three entries, scrolling. No AI identity is attached. The symbol
  is visible; the archetype within that symbol is not.
- No trend indicators (the pick log serves a similar function).

**Predicted metrics:**
- M3: 2.0-2.3
- M10: 2.0-2.5
- M11': 2.4-2.8
- M6: 65-78%
- M12: 0.25-0.4 (pick hints help signal-readers but also help committed players
  confirm their lane is intact; slightly lower M12 than Proposal B)

**Assessment:** More intuitive than trend indicators (explicit log entry vs.
inferred arrow direction) but lower M12 ceiling. Also risks making the draft
feel like pattern-matching a log rather than reading a draft.

---

## Champion: Proposal B — Bars + Snapshot + Depletion Trends

**Justification:**

Proposal B hits the M12 target (0.35-0.5 predicted vs. >= 0.3 required) while
keeping each information channel distinct in purpose. The depletion trend
creates a genuine skill gap between signal-readers and committed players because
committed players have no use for trends in their own lane — they know what
they are drafting. Signal-readers can read trends across multiple archetypes
to identify which lanes are being drained by AI activity and which remain open.

The round-start snapshot anchors planning without being a crutch: it is most
useful on picks 1-3 of each round (when it is relatively accurate) and
naturally becomes noise by round's end. This matches the desired urgency arc.
Proposal C's pick log is more intuitive but creates lower M12 because both
player types benefit from it nearly equally. Proposal A's bars alone undersell
the information system's potential.

---

## Champion Deep-Dive: Bars + Snapshot + Depletion Trends

### Round 1 (Picks 1-10): Exploration

**What the player sees at pick 1:** Eight resonance bars, each at full height
(pool is fresh). Depletion trend arrows all neutral (no data yet). A round-start
snapshot panel shows approximate composition: "~15 Tide, ~15 Stone, ~15 Ember,
~15 Zephyr resonance cards available. Pool quality: balanced."

**Information flow:** The player has no useful trend data yet. The snapshot
confirms the pool is fresh. The first 1-3 picks are exploratory — the player
samples available cards, noting which resonance types have high-quality options.

**Picks 3-6:** Trend arrows begin differentiating. Two or three archetypes show
fast-depleting trends (their bars shrink quickly relative to others) — these
are AI-contested lanes. Two or three archetypes trend stable. A skilled
signal-reader identifies which symbol categories are moving fastest and infers
that AIs are drafting those lanes.

**Picks 6-10:** Bars now show visible disparity. Two Tide bars have dropped
significantly; one Ember bar is stable. Player who identified Ember as open
in picks 3-6 is now 4-5 picks into their Ember archetype (convergence target
met: M5 pick 5-8). Late picks in round 1 confirm the commitment. At pick 10,
the round ends. Remaining pool: ~60 cards with visible archetype skew.

**Round 1 skill axis:** Signal-readers use trend data from picks 3-10 to
identify their lane 1-2 picks earlier than committed players. Committed players
pick highest fitness for their preselected archetype and ignore trends.
Expected M12 contribution from round 1 alone: modest (~0.1-0.15 in M3 terms).

### Round 2 (Picks 11-20): Commitment

**What the player sees at pick 11:** Refill resolves. Bars reset upward (or
partially, depending on refill volume). New round-start snapshot: "~12 Tide,
~13 Stone, ~14 Ember, ~11 Zephyr. Pool quality: Ember and Stone slightly
elevated." The snapshot reflects what the refill added plus what AI depletion
left behind — this is the most information-dense moment in the draft.

**Depletion trends reset** to neutral at round start (new baseline). Players
must re-read the pool from scratch — which archetypes deplete first in round 2
may differ slightly from round 1 if any AI has entered saturation behavior.

**Picks 11-15:** Signal-reader compares round 2 trend rates to round 1 memory.
If Ember was stable in round 1 and remains stable in round 2, the read is
confirmed: commit to Ember. If round 1's stable archetype is now depleting
fast in round 2, signal-reader has new information: another AI may have pivoted
(or the player's read was wrong). This mid-draft correction is the core of
signal-reading skill.

**Picks 15-20:** Committed player benefits from increasingly concentrated open
lane. Signal-reader who committed correctly at pick 8-10 is now drafting the
same pool as committed player — advantage realized. Signal-reader who committed
late (picks 12-14) is 2-4 picks behind on coherent archetype building.

### Round 3 (Picks 21-30): Execution

**What the player sees at pick 21:** Second refill resolves. Snapshot shows
a visibly skewed pool — if refill is partial or declining, AI lane archetypes
have lower bars than open lane archetypes. This snapshot is the most
strategically confirming moment: the player can see whether their lane is as
concentrated as expected.

**Depletion trends in round 3** are most informative for identifying the
remaining competition. If a previously-open lane is now depleting fast, a
new AI may be competing in that lane (implying the pool ran dry in their
primary lane and they are drafting secondaries). This is the highest-signal
moment in the draft.

**Picks 21-30:** No pivoting. The information system serves purely as
confirmation and quality targeting. Signal-readers who correctly identified
their lane pick 1-2 more S/A cards per pack on average in round 3 than
committed players who stumbled into a contested lane.

**Failure modes:**
- **Analysis paralysis:** Players who monitor all three information channels
  simultaneously may slow down. Mitigate by making bars the prominent display
  and trends/snapshot secondary (smaller, glanceable).
- **Trend misread:** A fast-depleting bar in round 1 pick 3 may be noise (early
  picks concentrate in a short window). Trend indicators should smooth over 2-3
  pick cycles before showing as "fast." Single-pick spikes should not trigger
  the fast-depletion indicator.
- **Snapshot staleness confusion:** Players who treat the round-start snapshot
  as a live display (checking it at pick 8 expecting current data) will
  misread it. UI must clearly timestamp the snapshot ("Pool state at round
  start") and dim it as picks progress to signal it is historical.
- **Over-confirmation:** A player committed to an archetype may selectively
  interpret trend data to confirm their commitment even when trends suggest
  switching. This is a cognitive bias risk, not a systems design risk —
  the information is accurate, the player is choosing not to use it.

---

## Complete Specification

### Information Elements

1. **Archetype availability bars (primary display)**
   - Eight bars, one per archetype, grouped by resonance symbol
   - Display: Relative height showing approximate card count as fraction of
     round-start count (not absolute numbers). Full = round-start count.
     50% = half of round-start cards remain.
   - Labels: Resonance symbol only (Tide, Stone, Ember, Zephyr). Player infers
     specific archetype from symbols. This preserves the archetype-within-symbol
     decision.
   - Update frequency: After each full pick cycle (all AIs + player pick).

2. **Depletion trend indicators (secondary display)**
   - Small directional arrow attached to each bar
   - Logic: Compare current depletion rate (picks taken from this archetype
     per cycle) to pool-average depletion rate. "Fast" = >1.5x average.
     "Stable" = 0.5-1.5x average. "Slow" = <0.5x average.
   - Smoothing: Average over last 2 pick cycles to reduce noise.
   - Update frequency: After each full pick cycle, using same data as bars.
   - Visual: Upward arrow (slow depletion, lots remaining), rightward (stable),
     downward (fast depletion, contested). Color-coded: green/grey/orange.

3. **Round-start snapshot (reference panel)**
   - Appears at beginning of each round (immediately after refill resolves)
   - Content: Approximate card counts per resonance symbol (not per archetype).
     A brief quality descriptor ("pool quality: balanced" vs. "elevated Ember
     concentration") based on proportion of available S/A cards (calculated
     from pool state, not revealed card by card).
   - Does not update mid-round. Dims progressively from pick 4 onward to signal
     staleness.
   - Reset: Replaced entirely at next round start.

### Information Not Shown

- Specific card counts or rarity breakdowns
- Individual AI pick logs
- AI identity or AI archetype assignments
- Exact S/A vs. C/F breakdown per archetype
- Refill preview (no advance notice of next round composition)

### Timing

- Bars and trends: Live throughout each round
- Snapshot: One per round boundary, displayed on round-start screen before
  picks begin
- No information withheld pending player action (all information is free)

### Interaction with Round Structure (Config B, 3 rounds of 10 picks)

- Round 1: Snapshot provides baseline; trends differentiate by pick 4-6;
  bars show growing disparity by pick 8-10.
- Round 2: New snapshot is primary recalibration tool. Trend reset allows
  re-reading. Signal-readers who confirmed their lane in round 1 use round 2
  snapshot primarily to validate their read persists.
- Round 3: Final snapshot shows most concentrated pool state. Trends in final
  round serve as confirmation of archetype availability, not discovery.

### Interaction with Player Strategies

- **Committed player:** Ignores trends and snapshot after round 1. Uses bars
  to confirm their lane is open, ignores everything else. M12 gap is created
  by committed players getting less value from information than signal-readers.
- **Signal-reader:** Uses snapshot at round start + trend data to identify
  open lane. Commits 2-4 picks later than committed player on average but
  with higher accuracy. Expected M3 benefit: +0.3-0.5 over committed players
  (meeting M12 target).
- **Power-chaser:** Ignores archetype-level information entirely. Bars are
  irrelevant to this strategy. Power-chaser M3 is lowest of three strategies
  regardless of information system.

### M12 Prediction

Predicted M12 = 0.35-0.45. The depletion trend mechanism is the primary
driver: signal-readers acting on trend data identify open lanes 2-4 picks
earlier and with fewer mispicks than committed players who guess. This gap
widens when combined with the round-start snapshot's archetype quality hint
(signal-readers prioritize open lanes with elevated quality indicators).

Upper bound (0.5) requires that refill structure also concentrates open lanes
meaningfully (Config B + partial refills). Lower bound (0.3) is achievable
even with full balanced refills because bars + trends create readable signal
regardless of pool concentration.

---

## Post-Critique Revision

### Orthogonal Nature of This Design

The critic's framing is correct: this proposal is an information layer, not a
mechanism design. It does not independently produce M3 or M11' scores — those
depend entirely on the underlying pick structure. The right way to read this
design is as an amplifier. Given any mechanism, the information system raises
or lowers M12 without materially changing M3 or M11'. This is a feature, not
a gap. It means the system can be paired with multiple mechanism proposals
without requiring redesign. The champion (Proposal B) should be evaluated
alongside a mechanism, not in isolation.

### M12 Dependency on Companion Mechanism — Accepted

The critic's constraint is accepted with one clarification. Under balanced
refills, M12 from this system is 0.15-0.25, not the 0.35-0.45 predicted in
the standalone spec above. Those higher figures assumed the refill structure
would create meaningful concentration gradients. The critic is right that the
information system amplifies an existing gradient — it does not create one. If
the companion mechanism produces only flat, balanced refills, the depletion
trend arrows converge toward equal rates and signal-reading advantage collapses.
The M12 floor of 0.15 under balanced refills is still positive (trends remain
readable even in flat pools) but falls short of the 0.3 target. The system
reaches 0.35-0.50 under biased refills, where concentration gradients exist
for signal-readers to act on. This makes pairing with Design 4 Proposal C
(graduated bias) the correct combination, as confirmed by its inclusion in
Hybrid A.

### Continuous-Market Adaptation (Design 6 / Hybrid B)

Round-based snapshots do not translate to a continuous market because there is
no round boundary to anchor them. The correct adaptation is a rolling 5-pick
window snapshot: show pool composition as it stood 5 picks ago, updated each
pick. This preserves the staleness property (the snapshot trails current state)
without requiring a round structure. The depletion trend calculation also
shifts from round-relative to window-relative: "fast" means >1.5x average
depletion rate over the last 5 picks, same threshold as the round-based spec.
The visual dimming behavior remains unchanged — the snapshot dims immediately
upon update rather than across a fixed round arc. This adaptation requires
no structural changes to the information display; only the baseline period
changes from "round start" to "5 picks ago."

### Round-Based vs. Rolling Windows

For round-based formats (Config B, Hybrid A), round-based windows are
preferable. The round boundary is a natural cognitive anchor — players already
orient their strategy to round starts. Resetting trends at round start matches
the planning cadence. Rolling windows would smooth out the round-boundary
recalibration moment, which is the most strategically dense point in the draft.
Rolling windows are correct only for continuous-market formats where no such
anchor exists.
