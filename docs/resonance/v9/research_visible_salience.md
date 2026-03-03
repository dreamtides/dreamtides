# Research: Visible Resonance Salience

**Assignment:** Research Agent C, Round 1 **Question:** Under what conditions
does the visible resonance system feel like the primary drafting signal, even
when hidden manipulation is active?

______________________________________________________________________

## 1. The Attribution Problem

When a player drafts well and ends up with a strong archetype deck, they
attribute that outcome to their decisions. This is the self-serving attribution
bias: positive outcomes are attributed to internal causes (skill, strategy, good
reads), negative outcomes to external ones (bad luck, weak pool). The design
implication is significant: **players will credit themselves for what the
algorithm actually did**, as long as the visible signals gave them something to
act on.

This is not deception — it is the fundamental condition of good game design.
Mario Kart rubber-banding is deceptive because it punishes skilled play
directly; the player's skill led to the bad outcome. Dreamtides' hidden metadata
is different: it creates conditions where the player's visible-signal strategy
produces good outcomes. The algorithm amplifies player decisions rather than
overriding them.

The key distinction: **deceptive hidden manipulation produces good outcomes
despite bad visible decisions. Good hidden manipulation produces good outcomes
that the player's visible decisions genuinely contributed to.** If a player who
ignored all resonance symbols got the same quality packs as a player who tracked
them carefully, visible resonance would be decorative. If the
resonance-attentive player does measurably better, the visible system is real —
even if the hidden layer adds additional precision the player cannot see.

This maps directly to V9's V1 metric (visible symbol influence). V1 should be
measured by stripping hidden metadata and running the algorithm on visible
symbols only. If visible-only delivers M3 = 1.6 and full hidden delivers M3 =
2.2, visible symbols account for ~73% of the gain over baseline (~0.6/0.8 of
total improvement). That is a visible-primary system. If visible-only delivers
M3 = 0.8 and full hidden delivers M3 = 2.2, visible symbols account for only 11%
of the total gain. That is a hidden-primary system with decorative visible
signals.

______________________________________________________________________

## 2. What Creates the Strongest "Building Toward Tide" Sensation

V8's player experience research identified three non-negotiable perceptual
requirements: the player must feel that (1) packs are improving as they commit,
(2) drafting on-resonance causes that improvement, and (3) meaningful choices
remain. These are the foundation for visible salience.

Beyond the minimum, these visible patterns create the strongest archetype
identity sensation — ranked by psychological impact:

### 2a. Accumulating Synergy — The Most Powerful Signal

The strongest "I am building toward Tide" feeling comes not from seeing more
Tide symbols, but from seeing Tide cards that **interact with each other**. When
a new pack contains a card that directly references another card already in the
player's draft pile — or that clearly belongs to the same mechanical ecosystem —
the player feels authorship over a coherent deck, not just accumulation of a
faction.

This is the Slay the Spire effect: the card pool for each character is curated
so that cards share mechanical DNA. When the Silent offers three cards and two
of them involve Poison, the player feels they are building a Poison deck,
regardless of any hidden weighting. The pool design does the work.

For Dreamtides: \*\*single-resonance Tide cards with tribal or mechanical cross-
references (e.g., "when you play a Warrior," "if you have a character with Spark

> = 3") create more archetype identity than generic Tide cards with a single
> symbol.\*\* Visible resonance symbols are the first signal; mechanical synergy
> language is the second, stronger signal. The dual-resonance signpost cards
> (~10%) should lean into this: they are most valuable when they are the hinge
> of a mechanical cluster, not merely a card with two symbols.

### 2b. Power Progression Over Time — The Second Signal

The second-strongest signal is seeing higher-power Tide cards appear as the
draft advances. This creates the "the game learns what I want" sensation that V8
rated as Narrative Gravity's defining experiential quality (7.9/10). The player
need not understand pool contraction to feel it: they observe that pick 20 packs
contain stronger cards for their strategy than pick 5 packs did.

For the visible system to get credit for this, the improvement must correlate
visibly with resonance commitment. If packs improve regardless of resonance
behavior (because the hidden algorithm is driving all quality), the player
attributes improvement to "getting lucky" or "the game being generous," not to
their resonance strategy. The improvement must feel **earned by visible
decisions.**

This argues for a design choice: the visible floor in early packs should be
noticeably lower than the visible quality in late committed packs. If early
packs contain a mix of resonance types and late packs are heavily concentrated
in the player's resonance, the player's eye will perceive the improvement and
attribute it correctly to their decisions — even if pool contraction is what
actually caused it.

### 2c. Quantity Signal — The Weakest But Most Legible Signal

Seeing more Tide cards over time is the most legible signal but the weakest
psychologically. Quantity (card count) is what players consciously track and
discuss: "I'm getting a lot of Tide cards." But quantity without quality and
synergy produces a different feeling: "I am being given Tide cards," not "I am
building a Tide deck." The passive voice matters. Quantity signals feel like
delivery; synergy and power signals feel like authorship.

**Design implication:** The hidden algorithm should prioritize delivering
higher-power on-archetype cards as commitment deepens (quality) over simply
delivering more single-resonance cards (quantity). The player experiences
quality as earned, quantity as received.

______________________________________________________________________

## 3. The Deception Threshold

Is there a threshold below which hidden manipulation is undetectable and above
which it becomes obvious?

The just-noticeable difference (JND) literature establishes that humans detect
relative changes more reliably than absolute ones (Weber's Law). For draft pack
quality, the analogous question is: can a player notice when their packs are
"suspiciously good" given their visible resonance commitment?

The evidence suggests two distinct thresholds:

**The invisibility threshold (~40% hidden contribution):** When hidden
manipulation accounts for less than ~40-50% of the total M3 improvement over
baseline, the effect is undetectable in normal play. The player experiences the
quality improvement as the natural result of their resonance choices. They
cannot distinguish between "the visible symbols are working" and "hidden
metadata is boosting my packs." This aligns with V9's V1 target of >= 60%
visible contribution: if visible symbols are doing 60%+ of the work, hidden
metadata stays below this threshold.

**The salience threshold (~70-80% hidden contribution):** When hidden
manipulation accounts for more than ~70-80% of pack quality decisions, observant
players begin to detect it — not by reverse-engineering the algorithm, but by
noticing anomalies. The most common detection pattern: "I took all off-archetype
cards early but my packs are still great." If the hidden metadata provides
near-full quality regardless of visible resonance behavior, players who
experiment by ignoring resonance will notice. This is the signal that the system
is operating without them.

**The transparency threshold (explicit and obvious):** When manipulation is so
strong that the draft feels solved or predetermined, players notice independent
of whether they try to detect it. CSCT at M6 = 99% crossed this threshold:
"autopilot after pick 5" and "every pack looks identical" are observations about
visible experience, not reverse-engineered mechanics.

**The critical asymmetry:** Detection of hidden manipulation is much harder when
the manipulation is *correlated with visible behavior*. If hidden metadata
boosts packs only when the player is already drafting on-visible-resonance, the
two signals are indistinguishable from the player's perspective. Detection
becomes possible only when players deliberately test by violating the visible
resonance signal and observe that packs remain good anyway.

**Design recommendation:** Ensure that a player who ignores visible resonance
completely has noticeably worse pack quality than one who follows it. This is
not punitive — it is the condition required for visible resonance to feel real.

______________________________________________________________________

## 4. Strategy Allocation: Visible Symbols vs. Card Mechanics vs. Power Level

How much of the player's strategy should each signal type determine?

### The Three Signal Hierarchy

In functional draft formats, player strategy emerges from three signal sources
in rough order of weight:

**Power level** (raw card quality) is the coarsest signal and the entry point.
New players draft by power. It is a reliable heuristic but does not determine
archetype: the best card in the pack might not fit the player's archetype.

**Visible symbols** (resonance tags) are the primary archetype signal. They
answer "does this card belong to my strategy?" before the player evaluates
synergy. In Dreamtides, visible resonance is the faction marker — the first
filter.

**Card mechanics** (synergy text) are the finest signal. They answer "does this
card make my specific deck better?" A card might have the right resonance symbol
but wrong mechanical theme (e.g., a self-mill Tide card in a Warriors deck).
Mechanics distinguish within a resonance, not across resonances.

### The Problem With Over-Weighting Visible Symbols

V8 identified the core failure mode: when 40% of cards carry dual-resonance
symbols, "the correct pick becomes obvious: just grab whatever matches your
(Tide, Zephyr)." The player stops evaluating mechanics and power and
pattern-matches symbols. This is not richer decision-making — it is shallower.

**The V9 target distribution of decision weight:**

| Signal                 | Target contribution to pick decision | Notes                                                   |
| ---------------------- | ------------------------------------ | ------------------------------------------------------- |
| Power level            | ~25%                                 | Floor — cards below power threshold rejected regardless |
| Visible resonance      | ~40%                                 | Primary archetype filter; narrows candidates            |
| Card mechanics/synergy | ~35%                                 | Final selection within resonance-filtered candidates    |

This distribution means the visible resonance system does real work (40%)
without overwhelming the other factors. The player thinks about symbols *and*
mechanics *and* power. No single signal is an automatic-pick trigger.

With only ~10% visible dual-resonance cards, this balance is achievable.
Dual-res signpost cards appear rarely enough that they are noteworthy, not
mechanical. The player actively evaluates them: "Is this (Tide, Zephyr) card
good enough in my Warriors deck to take over a better-powered single-symbol
card?" That is the kind of decision texture V9 needs to preserve.

______________________________________________________________________

## 5. Designing Dual-Resonance Signpost Cards for Maximum Signal Value

The ~10% visible dual-resonance cards (approximately 36 of 360) are V9's primary
visible signal instruments. Their design determines whether the visible system
feels like a real drafting force or a rare novelty.

Five design principles that maximize signal value despite rarity:

### 5a. Make Them Definitively Good

The MTG signpost uncommon literature is clear: signpost cards should be
"first-pick pickable." A weak or marginal dual-resonance card teaches players
nothing — it sits in the pack undrafted. A strong dual-resonance card that a
player *wants* to take creates an active decision: do I take this archetype-pair
card (signal: commit to Warriors) or the higher-power single-symbol card
(signal: stay flexible)?

That tension is the signal. If the dual-res card is weak, there is no tension.
If it is dominant, it removes evaluation (auto-pick). The target: slightly above
the average power of single-symbol cards in the same pack position, but not so
far above that evaluation collapses.

### 5b. Design Them as Mechanical Hinges, Not Just Symbol Bearers

A dual-res card that is merely "has two symbols and is a good card" provides
identity signal but not cohesion signal. A dual-res card whose **mechanics
reference the archetype's identity** ("Warrior characters you control have +1
Spark during combat") does both. The player who takes this card understands not
just "I am building Warriors" but "I am building a Warriors deck that cares
about Spark during combat." This is a narrower strategic commitment, and it
feels more intentional.

Mechanical hinge cards also solve the Goblin Artisans critique of signpost
uncommons (that they make choices "too obvious"): when the card's mechanical
text matters, the evaluation is about fit, not just symbol matching.

### 5c. Lean Into Scarcity — Treat Each As a Narrative Moment

At ~10% of the pool (~36 cards), a player will see approximately 4-6 dual-res
cards in a 30-pick draft. Each sighting is significant. Design each dual-res
card as if it is telling a story: "Here is the convergence point between Tide's
patience and Zephyr's speed. This is what Warriors looks like."

The scarcity creates natural sacredness. When dual-res cards appear rarely,
players notice them, evaluate them more carefully, and remember them. This is
the opposite of the V8 problem where 40% dual-res cards became background noise.

### 5d. Distribute Across Draft Phases, Not Just Early

If all 36 dual-res cards tend to appear in picks 1-10 (because they are higher
power and get drafted early), late-draft players in a committed archetype see no
visible dual-res signals during the consolidation phase (picks 15-30). This
disconnects the visible signal from the phase when the player feels most
committed.

Design several dual-res cards as explicitly late-pick value: moderate power but
high mechanical synergy payoff for committed decks. These cards appear in packs
alongside other committed cards and reinforce "yes, I am deep in Warriors" at
exactly the moment when M11 (late-draft quality) matters most. This supports the
V9 M11 >= 3.0 target through visible reinforcement, not just algorithmic
density.

### 5e. Ensure Asymmetric Signal Strength Between Archetype Pairs

The two archetypes sharing a resonance (Warriors: Tide/Zephyr and Ramp:
Zephyr/Tide) have subtly different dual-res card designs. Warriors' (Tide,
Zephyr) cards should feel mechanically different from Ramp's (Zephyr, Tide)
cards — not just in symbol order but in what the card *does*. A Warriors
dual-res card cares about combat; a Ramp dual-res card cares about mana
progression. This ensures that the visible signal distinguishes archetypes
within a resonance pair, not just between resonance pairs.

______________________________________________________________________

## 6. The Attribution Balance: Good Design vs. Deception

**The core question posed in the research assignment deserves a direct answer.**

If the algorithm secretly makes 60% of pack quality decisions, but the player
attributes quality to their visible-resonance strategy — is that good design or
deception?

**It is good design under the following conditions, and deception when they
fail:**

**Condition 1: Visible behavior predicts outcome.** If a player who carefully
tracks visible resonance symbols and commits early consistently gets better
results than one who ignores them, visible resonance is a real strategy. The
algorithm amplifying that strategy is not deception — it is support. The player
is not wrong to credit their visible decisions; the decisions were genuinely
predictive.

**Condition 2: The hidden layer is an extension of the visible layer's logic.**
If visible (Tide) symbols cause the player to draft Tide cards, and the hidden
metadata on those cards says "this is a Warriors card," the hidden layer is a
refinement of the visible signal, not a contradiction of it. Players who
discovered this would likely say "that makes sense." If hidden metadata
contradicts visible signals — a card showing (Tide) tagged as a Zephyr card for
algorithmic reasons — players who discovered it would feel deceived.

**Condition 3: The system is reversible.** Players can change strategy mid-draft
by shifting which visible resonance they prioritize, and the algorithm follows.
If the algorithm locks in based on early hidden decisions and ignores later
visible changes, players who try to pivot are punished by a system they cannot
see.

When these three conditions hold, hidden metadata at any percentage is
defensible. V9's design integrity spectrum (levels 1-4 in the orchestration
plan) maps to these conditions: levels 1-3 (metadata derived from real card
properties) satisfy all three. Level 4 (labels disconnected from mechanics)
risks failing Condition 2 for observant players.

______________________________________________________________________

## 7. Framework for V9 Algorithm Designers: Evaluating Visible Salience

Use this checklist to evaluate whether a proposed V9 algorithm keeps visible
resonance as the primary signal:

**Quantitative tests (from V4 criterion):**

1. Strip hidden metadata; run 100 drafts on visible signals only. Record
   M3_visible.
2. Run 100 drafts with full hidden metadata. Record M3_full.
3. **V1 score:** (M3_visible - M3_baseline) / (M3_full - M3_baseline). Target >=
   0.6.
4. Run 100 drafts where the committed player strategy ignores visible resonance
   entirely (picks by power only). Record M3_power.
5. **Gap test:** M3_full (resonance-reader) should exceed M3_power by >= 0.4.
   This confirms visible resonance choices are meaningfully rewarded.

**Qualitative tests (player experience simulation):**

6. Trace a 30-pick draft. Count the number of picks where the "best
   visible-resonance pick" and the "best hidden-metadata pick" are different
   cards. Target: < 20% of picks should diverge. High divergence means hidden
   metadata is overriding visible signal.
7. Trace the same draft. Identify the 3 most memorable picks (highest decision
   tension). Are they driven by visible resonance evaluation, power evaluation,
   or mechanical synergy evaluation? All three should appear. If all 3 are
   mechanical (ignoring symbols), visible resonance is underweighted.
8. Check whether dual-resonance signpost cards appear in approximately the right
   phase distribution: ~2 in picks 1-10, ~2 in picks 11-20, ~2 in picks 21-30.
   Heavy front-loading means the visible archetype signal vanishes in late
   draft.

**Red flags that indicate visible resonance has become decorative:**

- A power-chaser (ignores resonance entirely) gets M3 within 0.3 of a resonance-
  reader. Visible resonance is not rewarded.
- No draft picks involve genuine tension between a dual-res signpost and a
  higher-power single-res card. Signposts are either always auto-takes or always
  skipped.
- Players can describe their archetype choice only in terms of "which cards
  appeared" rather than "which resonance symbols I saw and committed to."
- Late-draft packs (picks 20-30) feel like the algorithm delivered the deck
  rather than the player built it.

______________________________________________________________________

## 8. Summary: The Conditions for Visible Salience

Visible resonance feels primary — even when hidden manipulation is active — when
five conditions are jointly satisfied:

1. **Visible behavior is predictive.** Players who follow visible signals
   measurably outperform those who ignore them.
2. **Mechanical synergy reinforces the signal.** Cards with resonance symbols
   also share mechanical vocabulary, so the symbols are redundant confirmation,
   not the only clue.
3. **Dual-resonance cards are memorable.** At ~10% density, they are rare enough
   to be noteworthy, powerful enough to be decision-forcing, and mechanically
   specific enough to narrow the player's strategic direction.
4. **Quality improvement is felt as earned.** Late-draft pack quality is visibly
   higher than early-draft quality, and the player's resonance commitment is the
   legible cause (even if pool contraction is the actual mechanism).
5. **Hidden metadata aligns with visible signals.** The algorithm's targeting
   follows what the visible symbols predict, so there is no contradiction a
   player could discover.

When these conditions hold, the V9 design goal is met: visible resonance is the
primary drafting signal. Hidden metadata is infrastructure, not the strategy
itself.

______________________________________________________________________

## Sources Consulted

- Orchestration Plan V9: `/docs/resonance/v9/orchestration_plan_v9.md`
- V8 Final Report: `/docs/resonance/v8/final_report.md`
- V8 Algorithm Overview: `/docs/resonance/v8/algorithm_overview.md`
- V8 Player Experience Research:
  `/docs/resonance/v8/research_player_experience.md`
- V8 Pool Composition Research:
  `/docs/resonance/v8/research_pool_composition.md`
- [Attribution Theory in Gaming](https://www.roffle.net/attribution-theory/)
- [Drafting 101: Understanding Signals — Magic: The Gathering](https://magic.wizards.com/en/news/feature/drafting-101-understanding-signals-2016-04-12)
- [Goblin Artisans: Signpost Uncommons: A Critique](http://goblinartisans.blogspot.com/2020/11/signpost-uncommons-critique.html)
- [The Algorithmic Alibi: Procedural Generation and Agency — Wayline](https://www.wayline.io/blog/algorithmic-alibi-procedural-generation-agency)
- [Illusion of Control — Wikipedia](https://en.wikipedia.org/wiki/Illusion_of_control)
- [Rubber-Banding as a Design Requirement — Gamedeveloper.com](https://www.gamedeveloper.com/design/rubber-banding-as-a-design-requirement)
- [User Research: Why Players Love Games That Make Them Frustrated — Gamedeveloper.com](https://www.gamedeveloper.com/business/user-research-why-players-love-game-that-makes-them-frustrated)
- [Hearthstone Arena Revamp 2025 — Blizzard Watch](https://blizzardwatch.com/2025/05/30/hearthstone-revamps-arena/)
