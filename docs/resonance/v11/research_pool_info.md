# Pool Information in Draft Games: Research Findings

## The Core Tradeoff

Every draft game sits on a spectrum from fully hidden to fully visible pool
information. The tradeoff is not simply "more info = better decisions." It is
about what kind of skill the game rewards and whether that skill feels earned.

**Hidden information** (classic MTG booster draft): Players infer the pool from
what is passed to them. Signals are indirect — a missing card communicates that
someone upstream took it. This rewards a deductive skill: reading absence. The
skill ceiling is high, but the skill floor is low. New players get no feedback
on why their signals were wrong until it is too late.

**Open information** (shared market games like Dominion, Terraforming Mars):
All players see the same supply. The skill axis shifts to engine optimization
and timing — when to take a card vs. when a rival needs it more. No inference
required; the strategy is reactive competition for the same visible pool.

**Partial information** (7 Wonders: visible tableaux, hidden hands): Players
see what opponents have built but not what cards they are holding. This is
arguably the most strategic zone. You can read opponent direction without
knowing exactly what they will do next turn. Adaptation is required.

## What "Reading the Table" Actually Means

In MTG booster draft, the canonical skill is inferring which colors/archetypes
are open based on pack contents. Three concrete signal sources:

1. **Late picks on normally-early cards**: If a strong card arrives at pick 7,
   someone upstream passed it. Their archetype probably doesn't want it.
2. **Pack density**: A thin pack means upstream players took heavily from it.
3. **Wheel tracking**: Cards seen in pack 1 that return in pack 3 confirm what
   upstream took.

This system is elegant because the information is real — it is physically
encoded in what arrives — but it is also lagged (pack 1 signals inform picks
4-8, not immediately) and noisy (a pass could mean hate-drafting, wrong
evaluation, or genuine openness).

MTG Arena's Quick Draft (bots) vs. Premier Draft (humans) illustrates the gap.
Research from 17Lands shows bot drafts are more consistent signal senders, but
exploiting bot tendencies does not transfer to human drafts. The signal-reading
skill is different because the underlying agent behavior is different. When you
know the AI's heuristics, signals become mechanical extraction rather than
deduction.

## When Explicit Information Helps vs. Hurts

**It helps when:**
- Players lack the reference knowledge to interpret indirect signals. Showing
  archetype availability reduces the knowledge barrier without eliminating the
  decision of what to do with that information.
- The decision space is large enough that the information does not reduce picks
  to a solved lookup. Eight archetypes with overlapping symbols creates enough
  ambiguity that "Tide cards are available" still requires reasoning about which
  Tide archetype to commit to.
- The game's AI opponents are not human-readable. With AI drafters whose
  behavior is fixed and consistent (Level 0), there is no "body language" to
  read — explicit signals substitute for the social inference that is absent.

**It hurts when:**
- Information resolves the pick completely. If "archetype X has the most cards"
  perfectly predicts "draft archetype X," the skill axis collapses to a
  lookup table. The information needs to be a useful input, not a direct answer.
- Players have no way to act on the information. Showing future refill
  composition matters only if players can plan for it — meaning the refill
  preview must arrive early enough in a round to influence early-round picks.
- Information overload causes analysis paralysis. Game design research
  consistently shows that too many simultaneous data streams degrade decision
  quality as badly as too little information. Players need enough to plan, not
  enough to solve.

## Information Formats: What Works

**Counts/bars (archetype availability):** Best for communicating relative
scarcity. "Ember: much available / Tide: scarce" gives direction without
revealing specific cards. Retains the player's decision of which Ember
archetype to pursue. Low cognitive load. The tradeoff is that bars mask quality
— a "full" Ember pool could be full of bad Ember cards.

**Trend indicators (depletion rate):** Better than raw counts for inferring AI
behavior. A shrinking bar is more informative than a static count because it
implies agency — someone is taking those cards. This is the closest substitute
for reading actual human signals. Risk: if depletion rate directly maps to AI
archetype, it becomes a lookup rather than inference.

**AI pick hints (last archetype taken):** Most direct substitution for human
table-reading. "AI 3 took a Tide card" is the AI equivalent of "player 3
passed a strong Zephyr card." This is the format most grounded in existing
draft precedent. Risk: if AI behavior is simple and predictable enough, this
reduces to tracking a state machine rather than reading a player.

**Refill preview:** Unique to multi-round draft structures. No direct analog in
MTG. Most useful for experienced players who can reason about multi-round arcs.
Creates a second-order skill: not just "what is open now" but "what will be
available when I need it." Disadvantage: rewards deferral and may reduce early
commitment pressure.

**Specific card counts / full pool composition:** The most transparent option.
Research on analysis paralysis is clear that this level of specificity can
overwhelm decision-making without improving outcomes. In a pool of hundreds of
cards across 8 archetypes, listing counts by rarity and resonance type produces
numbers that require expert-level pattern recognition to use well. Better for
post-draft analysis than in-draft decisions.

**Round-start snapshot:** A time-bounded version of full composition. Because
the snapshot is taken at round start and becomes stale as picks are made, it
rewards planning at round start without guaranteeing continued accuracy. This
is well-matched to the multi-round structure — the snapshot is useful for the
first few picks of each round but becomes noise by the end.

## The "Solved Puzzle" Failure Mode

The risk with any explicit pool information is that a sophisticated player can
construct a decision procedure from it that is near-optimal, removing the
uncertainty that makes drafting interesting. This is analogous to what happened
with MTG Arena bot exploitation: players learned the bot pick order and
extracted value mechanically, which was both less skill-intensive and less fun.

The antidote is ensuring that pool information is a useful *input* to a
decision that still has meaningful uncertainty downstream. Three mechanisms
that preserve this uncertainty even with visible pool info:

1. **Quality is hidden even when quantity is visible.** Knowing there are 15
   Tide cards does not tell you which 3 of them are S/A tier. Bars/counts
   reveal availability but not quality distribution.
2. **AI behavior has a stochastic component.** If AIs can occasionally deviate
   from their archetype (saturation mechanic, round-aware picks), trend data
   becomes probabilistic inference rather than certain tracking.
3. **Information arrives at a lag.** Round-start snapshots are already
   partially outdated by pick 2. Trend indicators update per-pick but do not
   predict individual picks. This preserves short-term uncertainty within a
   longer-term trend.

## AI Opponents and Information Transparency

The question of whether the player should know what AIs are drafting is
separate from whether they should know pool state. These have different effects:

**Pool state information** (archetype bars, composition summary): Lets the
player read the *result* of AI behavior without understanding *intent*. This is
analogous to reading a pack — you see what was taken, not why.

**AI pick hints** (last archetype taken per AI): Reveals intent, which is
higher-fidelity than pool state. This is closer to seeing opponents' tableaux
in 7 Wonders — you know what they are building, which changes the strategic
calculation.

Both are legitimate design choices. Pool state information preserves inference;
AI pick hints reduce inference and increase reaction. For M12's target of
>= 0.3 advantage for signal-readers over committed players, some level of
readable signal is necessary. If signals are too opaque (hidden pool state),
committed players catch up because signal-reading provides no advantage. If
signals are too explicit (full AI pick tracking), committed players suffer
unfairly because commitment is punished by too-perfect information updates.

The calibration sweet spot is probably: pool-level trend indicators (bars,
depletion rates) rather than per-AI per-pick tracking. This gives
signal-readers a meaningful advantage from reading trends while leaving enough
noise that commitment remains valuable.

## Practical Recommendations for V11

**Most likely to work:** Archetype availability bars (relative counts, updated
per pick) combined with a round-start snapshot. This gives signal-readers a
clear advantage early in each round and a planning edge at round start, while
preserving quality-level uncertainty throughout. Cost: free. Requiring a pick
to see the pool state would be novel but untested in analogous games.

**Second priority:** Archetype trend indicators showing depletion rate by
archetype since last refill. This is the most direct substitute for human
table-reading and the most grounded in existing draft precedent.

**Avoid:** Full card-level or rarity-level pool counts. These add cognitive
load without proportionate decision quality improvement. The risk of analysis
paralysis is higher than the risk of too little information.

**On AI pick hints:** Consider showing archetype category (Tide, Ember, etc.)
rather than specific AI identity. "A Tide card was just taken" vs. "AI 3 took
a Tide card" — the former is closer to pool-state reading; the latter is closer
to player tracking. For M12 targets, archetype category hints without AI
identity should be sufficient to create a signal-reader advantage.

**On timing:** Refill preview should arrive at the end of a round (before the
refill resolves) so players can adjust their final picks of the round based on
what is coming. Providing it at round start alongside the snapshot gives
players redundant information.

## Sources Consulted

- [Drafting 101: Understanding Signals — Magic: The Gathering](https://magic.wizards.com/en/news/feature/drafting-101-understanding-signals-2016-04-12)
- [Signals in Booster Draft — MTG](https://magic.wizards.com/en/news/feature/signals-booster-draft-2015-01-19)
- [Do the Bots Send Signals? — 17Lands Blog](https://blog.17lands.com/posts/bot-signals/)
- [Quick vs. Premier Draft on MTGA — Draftsim](https://draftsim.com/mtg-arena-quick-draft-vs-premier/)
- [Open Drafting vs. Closed Drafting — Bombard Games](https://bombardgames.com/board-game-mechanics-open-drafting-closed-drafting/)
- [Card Drafting Mechanic — Nerd Lab Games](https://nerdlab-games.com/028-card-drafting-mechanic-part-1/)
- [Designing Games to Prevent Analysis Paralysis — League of Gamemakers](https://www.leagueofgamemakers.com/designing-games-to-prevent-analysis-paralysis-part-1/)
- [Designing Interesting Decisions in Games — Gamedeveloper](https://www.gamedeveloper.com/design/designing-interesting-decisions-in-games-and-when-not-to-)
- [7 Wonders Strategy — Cardboard Mountain](https://cardboardmountain.com/7-wonders-strategy/)
- [AI Solutions for Drafting in MTG — arXiv](https://arxiv.org/pdf/2009.00655)
