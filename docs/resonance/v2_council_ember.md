# Ember Advocate Position Paper -- V2 Design Council

## Executive Summary

Ember is the shared resonance for **Tempest** (Tide+Ember), **Gale** (Zephyr+Ember), **Crucible** (Stone+Ember), and **Cinder** (Ember+Ruin). With 45 mono-Ember cards (at the ceiling), Ember carries significant load across four diverse archetypes: storm/spellslinger (Tempest), fast-tempo (Gale), Warrior tribal (Crucible), and sacrifice/aristocrats (Cinder). This diversity is a strength -- Ember's card pool serves radically different strategies -- but it also means each archetype effectively sees only ~25-35 "on-plan" Ember cards rather than the full 45.

**Overall Assessment:** Ember archetypes are in good shape post-v2. Cinder is format-leading (8.4/10). Tempest is strong (8.2/10). Crucible made significant progress on the linearity problem (7.6/10). Gale is the archetype I am most concerned about (7.2/10) -- it has a structural accessibility gap that undermines the design intent.

---

## Archetype-by-Archetype Assessment

### Tempest (Tide+Ember) -- 8.2/10, A-

**Strengths:** Deepest event engine in the format. Stormtide Oracle is a top-3 signpost that successfully addresses the "homogeneous event-count trigger" problem from v1. Two genuinely distinct builds exist (Big Storm vs Slow Burn) with minimal overlap. Duskwatch Vigil creates a novel Prevent-to-discount bridge that makes reactive Tide cards productive for storm.

**Concerns:**
1. **Starcatcher allocation error.** The allocation document lists Starcatcher as "Primary Archetype(s): Tempest, Basalt" but places it in Stone. Tempest is Tide+Ember; it cannot access Stone. This is a genuine loss -- "when you play event, gain 1 energy" was a key self-sustaining piece for the storm engine. Either the archetype tag must be corrected (removing Tempest) or the card needs to return to Ember. Given Stone's hard-won deficit recovery, I lean toward accepting the loss and removing the Tempest tag. Tempest has 9/10 core depth and can absorb this.
2. **Ember noise.** ~15 Cinder-specific abandon cards and ~7 Crucible Warrior-tribal cards are dead weight for Tempest in the Ember pool. This is structural and unavoidable, but it does mean Tempest navigates more traps than most archetypes.
3. **High Legendary dependency.** Epiphany Unfolded and Moment Rewound create a large power variance between Tempest drafts.

**Verdict:** Tempest is healthy. No action required except fixing the Starcatcher tag.

---

### Gale (Zephyr+Ember) -- 7.2/10, B

**Strengths:** Deepest fast-card density in the format. Clear mechanical identity (fast-matters + tempo). Windstride Runner fixes the relative signpost deficit. Musician tribal + fast removal = crisp draft signals.

**CRITICAL CONCERN: The Accessibility Gap.**

Three cards explicitly designed for Gale are in resonances Gale cannot access:

| Card | Resonance | Gale Can Access? |
|------|-----------|-----------------|
| Duskwatch Vigil | Tide | NO |
| Voidthorn Protector | Ruin | NO |
| Dreamtide Cartographer | Tide | NO |

This means Gale received **the fewest directly-accessible new cards of any archetype** -- only 3 (Windstride Runner, Ashen Threshold, Nexus of Passing). Of these, only Windstride Runner was specifically designed for Gale.

**My proposal:** Move Dreamtide Cartographer from mono-Tide to mono-Zephyr. The card's "3 or fewer in hand: draw 2" mode is hand-emptying refuel, which is Zephyr's tempo identity, not Tide's information-control identity. This single change gives Gale, Mirage, Basalt, and Eclipse access while removing Tempest, Undertow, and Depths access. Tempest loses a supplementary card but has 9/10 core depth. The Tide advocate may object, but Tide at 40 cards can afford to drop to 39, and the card's mechanic genuinely fits Zephyr better.

For Duskwatch Vigil and Voidthorn Protector, I propose removing Gale from their archetype tags since the accessibility gap is too fundamental to fix by moving those cards. Duskwatch Vigil is genuinely a Tide card; Voidthorn Protector is genuinely a Ruin card. Their Gale tags are aspirational, not functional.

**Additional Gale concerns:**
- Musician concentration risk: Only 4 Musicians (Sage, Intermezzo, Melodist, neutral Minstrel). If 2+ are taken by other drafters, Gale's payoff layer collapses.
- No Prevent access without splash -- the Prevent-tempo sub-theme exists only in theory.
- Basalt Spirit Animal bloat eats ~12-14 of Zephyr's 43 cards. Effective Zephyr pool for Gale is ~29-31.

**Verdict:** Gale needs the Dreamtide Cartographer reassignment. Without it, Gale is the format's most underserved archetype relative to design intent.

---

### Crucible (Stone+Ember) -- 7.6/10, B+

**Strengths:** The purity-vs-power tension is the single biggest v2 improvement. Rails score dropped from 9/10 to 6/10 through five non-Warrior power pieces (Ironveil Watcher, Vanguard of the Summit, Kindlespark Harvester, Ironbark Warden, Everhold Protector). Forgeborn Martyr is an excellent signpost with the "3+ allies" death dividend enabling self-sacrifice. Two distinct builds exist: Warrior Phalanx (pure tribal) vs Crucible Engine (Judgment storm + purity breakers).

**Nuanced assessment of the purity tension:**

The tension IS working, but it is more subtle than dramatic. Both builds still deploy Warriors, pump them, and score via Judgment. The difference is HOW they score (spark vs direct points), not WHEN or WHY. Compare to Eclipse, which can genuinely pivot between aggro and engine. Crucible's two builds are "same plan, different tools" -- healthy for a tribal archetype but not as flexible as non-tribal archetypes.

**Concerns:**
1. **Warrior recursion isolation.** The most exciting recursion pieces (Risen Champion, Ashen Avenger) are in Ruin, outside Crucible's pool. Grim Reclaimer and Speaker for the Forgotten provide some recursion but are expensive or sacrifice-gated. After a board wipe, Crucible has limited recovery.
2. **Event poverty in Stone.** Only 4 Stone events means Starcatcher ("play event: gain energy") is a trap card in Crucible despite being in its pool.
3. **Sacrifice tension with persistence.** Ember's abandon cards conflict with Ironbark Warden and Everhold Protector's "stayed in play" requirements. This is intentional and creates good decisions.

**Verdict:** Crucible is in a healthy place. The linearity improvement is real and sufficient. No action required.

---

### Cinder (Ember+Ruin) -- 8.4/10, A

**Strengths:** The deepest, most self-consistent sacrifice engine in the format. 18+ abandon cards, deep recursion pipeline, two genuinely distinct builds (Sacrifice Engine vs Dissolved Aristocrats). Ashfire Ritualist is a model signpost. Draft signals are the clearest in the format ("abandon" is unambiguous). Tidechannel Observer creates a novel void-velocity scoring axis.

**Concern: Fodder Scarcity.**

Cinder has 18+ sacrifice outlets but a surprisingly thin supply of cheap, expendable bodies:
- Truly expendable (designed to be sacrificed repeatedly): Ashen Remnant, Resilient Wanderer, Exiles of the Last Light, Nightmare Manifest
- Figment generation: Radiant Trio (3 energy), Endless Projection (4 energy) -- expensive setup
- 0-cost bodies: Only Aspiring Guardian (Neutral)

The gap is at the 0-1 cost slot. Cinder needs cheap bodies to start the sacrifice engine, but by the time you pay 3 energy for Radiant Trio to generate figments, the tempo cost is significant.

**My proposal:** A new 1-cost Ember character (perhaps a Survivor or Outsider) with a small Materialized effect and no other text -- the explicit "sacrifice me" card. Something like: "Materialized: Gain 1 energy" on a 1-cost, 0-spark body. This would serve Cinder's fodder needs, Gale's hand-emptying plan, and Crucible's aggro deployment. However, Ember is at 45 (ceiling). This would require either raising the ceiling or finding an Ember card to shift elsewhere.

**Additional Cinder concerns:**
- Recovery from empty-board states is poor (all draw is sacrifice-gated)
- Ruin Survivor density creates Undertow trap cards (~10-12 off-plan Ruin cards)
- Banish effects bypass Cinder's entire recursion strategy
- Once-per-turn cap on Ashfire Ritualist kindle is correct but limits ceiling

**Verdict:** Cinder is excellent. The fodder issue is real but not critical -- it creates a healthy constraint on what would otherwise be overwhelming.

---

## Cross-Cutting Issues

### Issue 1: Ember at 45 (Ceiling)

Ember and Ruin are both at 45, the ceiling of the 38-45 range. This means:
- No room for new Ember cards without cuts or ceiling increases
- Any mono-resonance shifts INTO Ember are blocked
- Cinder (Ember+Ruin) cannot receive additional support through either resonance

If Dreamtide Cartographer moves from Tide to Zephyr (my proposal), this does not affect Ember count. But if we want to address Cinder's fodder problem or Gale's support gap through new Ember cards, we need headroom. I could support moving one marginal Ember card to Neutral if it creates space -- Data Pulse ("Gain 2 energy, draw 1") is generic enough to be Neutral, and freeing one Ember slot would allow a purpose-built card.

### Issue 2: Ashen Threshold as Cross-Pollinator

Ashen Threshold (mono-Ember) was designed for Mirage, Cinder, and Bedrock. Its "ally leaves play: gain 1 energy" + "once/turn void materialize: draw 1" is beautifully designed. But it lists Bedrock as a primary archetype, and Bedrock (Stone+Ruin) cannot access mono-Ember cards. This is the same class of error as the Gale accessibility problem. The Bedrock tag should be removed from Ashen Threshold's archetype listing.

### Issue 3: Former Dual-to-Mono Cards

Three former Ember+Ruin duals (Exiles of the Last Light, Eclipse Herald, Rebirth Ritualist) became mono-Ember. This is correct -- their primary mechanics are Ember. But it does mean these cards are now accessible to Tempest and Gale (where they are mostly off-plan) while remaining useful to Cinder. The main consequence is minor pool dilution for Tempest/Gale, which is unavoidable.

---

## Proposals for Other Advocates

### To the Tide Advocate:
1. **Concede Dreamtide Cartographer.** Its hand-emptying draw mode is mechanically Zephyr. In return, I acknowledge that Tempest loses a supplementary card, but Tempest's 9/10 core depth can absorb this.
2. **Fix the Starcatcher tag.** Remove Tempest from Starcatcher's archetype listing since Starcatcher is in Stone and Tempest cannot access it.
3. **Acknowledge the Duskwatch Vigil situation.** The card is correctly in Tide; removing the Gale tag is the right fix.

### To the Zephyr Advocate:
1. **Accept Dreamtide Cartographer into Zephyr.** This gives Gale its most-needed card (hand-emptying refuel) and also benefits Mirage, Basalt, and Eclipse. Zephyr goes from 43 to 44, within ceiling.
2. **Discuss Gale's Musician concentration risk.** Would you support a new Zephyr Musician to provide redundancy, or is the 4-card package sufficient?

### To the Stone Advocate:
1. **Crucible's purity-vs-power is working.** Thank you for the non-Warrior Stone power pieces. Ironveil Watcher, Kindlespark Harvester, and Ironbark Warden are excellent additions.
2. **Acknowledge Starcatcher's Stone assignment.** I accept the move -- Stone's deficit recovery is more important than Tempest's marginal storm enabler.
3. **Bedrock discussion.** Ashen Threshold's Bedrock tag is aspirational (Bedrock can't access Ember). Can we collaborate on a Stone card that bridges to void play for Bedrock?

### To the Ruin Advocate:
1. **Cinder fodder discussion.** The sacrifice engine needs more cheap expendable bodies. Could Ruin contribute a 1-cost Survivor with minimal abilities?
2. **Voidthorn Protector's Gale tag.** Should be removed -- the card is correctly in Ruin but Gale can't access it.
3. **Tidechannel Observer is excellent.** This card serves Cinder beautifully alongside Undertow and Eclipse. Well done.
4. **Cinder's Survivor noise.** ~10-12 Undertow-optimized Survivors create trap picks for Cinder drafters. This is mostly structural and unfixable, but worth acknowledging.

---

## Summary of Action Items

| Priority | Item | Affects | Proposal |
|----------|------|---------|----------|
| HIGH | Move Dreamtide Cartographer to Zephyr | Gale, Mirage, Basalt, Eclipse, (loses Tempest/Undertow/Depths) | Reassign resonance |
| HIGH | Fix Starcatcher archetype tag | Tempest | Remove Tempest from tag |
| MEDIUM | Fix Gale tags on Duskwatch Vigil & Voidthorn Protector | Gale | Remove Gale from archetype listings |
| MEDIUM | Fix Bedrock tag on Ashen Threshold | Bedrock | Remove Bedrock from archetype listing |
| MEDIUM | Fix Cinder tag on Kindlespark Harvester | Cinder | Remove Cinder from archetype listing (mono-Stone, Cinder can't access) |
| LOW | Cinder fodder scarcity | Cinder | Monitor in playtesting; existing figment generation is likely sufficient |
| LOW | Gale Musician redundancy | Gale | 4-card package is sufficient; not proposing new Musician |
| LOW | Bedrock Stone bridge | Bedrock | Propose new Stone card with void-play triggers to Stone advocate |

---

## FINAL REVISED ARCHETYPE ASSESSMENTS

*Written after three rounds of discussion with all four resonance advocates.*

---

### Tempest (Tide+Ember) -- FINAL: 8.2/10, A-

**V2 Verdict: Excellent. The homogeneity problem is solved.**

Tempest emerges from v2 as a format pillar. The critical v1 complaint -- "4 of 5 Tempest cards used the same '2+ events this turn' trigger" -- is comprehensively addressed:

1. **Stormtide Oracle** uses cumulative void-event count (not turn-based)
2. **Archivist of Vanished Names** provides type-based tutoring + incidental mill (no event-count dependency)
3. **Duskwatch Vigil** creates a Prevent-to-discount loop rewarding reactive play
4. **Dreamtide Cartographer** (if it remains in Tide) rewards hand-empty aftermath, not the storm itself

Two genuinely distinct builds exist:
- **Big Storm:** Chain events, copy/amplify, score through burst (Illumination of Glory) + cumulative (Stormtide Oracle)
- **Slow Burn:** Control with Prevents, grow Stormtide Oracle over 8-10 turns, grind through event retrieval

These builds share Keeper of the Lightpath and Stormtide Oracle but diverge on 80% of their remaining cards. Two Tempest drafters could coexist at a table with minimal card overlap.

**Residual concerns (all acceptable):**
- Starcatcher's move to Stone is a real loss but absorbable (9/10 core depth provides redundancy)
- ~22 Cinder/Crucible cards in Ember are noise, but navigating irrelevant cards in a shared pool is core to draft skill
- High Legendary dependency (Epiphany Unfolded, Moment Rewound) creates power variance between drafts

**Tag cleanup required:** Remove Tempest from Starcatcher's archetype listing.

**No mechanical changes recommended.**

---

### Gale (Zephyr+Ember) -- FINAL: 7.2/10 (7.6 with Cartographer fix), B (B+ with fix)

**V2 Verdict: Functional but underserved. One critical fix needed.**

Gale is mechanically sound -- its fast-matters identity is clear, its draft signals are crisp, and Windstride Runner resolves the relative signpost deficit. The 113-card accessible pool is the second-largest in the format. But the headline number is misleading:

**Effective pool analysis:**
- 43 mono-Zephyr minus ~12-14 Basalt Spirit Animals = ~29-31 on-plan Zephyr cards
- 45 mono-Ember minus ~15 Tempest storm cards minus ~7 Crucible Warrior cards minus ~5 deep Cinder sacrifice = ~18-20 genuinely Gale-relevant Ember cards
- Plus ~12 on-plan Neutrals and 1 signpost
- **Real on-plan pool: ~60-64 cards** (not 113)

This is above the 60-80 minimum, but just barely. And of the 25 new v2 cards, Gale can directly access only 3 (Windstride Runner, Ashen Threshold, Nexus of Passing). Three cards designed for Gale (Duskwatch Vigil, Voidthorn Protector, Dreamtide Cartographer) are in inaccessible resonances. This is the most significant accessibility failure in the v2 design.

**The Dreamtide Cartographer proposal (my highest-priority item):**
Moving this card from mono-Tide to mono-Zephyr is the single highest-impact fix available for the format. The card's hand-emptying draw mode is mechanically Zephyr (tempo refuel), not Tide (information control). The move gives Gale its most-needed card while also fixing Eclipse's accessibility issue. Net archetype impact: +3 meaningful gains (Gale, Eclipse, Basalt) vs -1 meaningful loss (Depths) and -2 marginal losses (Tempest, Undertow). Compromise alternative: make it Tide+Zephyr dual (11th dual, breaking the exactly-10 policy).

**Musician concentration risk:** After analysis, I've concluded the 4-card Musician package is sufficient, not ideal. Gale's fast-matters identity isn't solely Musician-dependent -- Windstride Runner's hellbent, Moonlit Dancer's fast-character energy, and the general fast-removal suite provide non-Musician payoffs. The Musicians are a reward for going deep, not the only path.

**Two distinct builds exist:**
- **Musician Crescendo:** Fast-matters aggro with Musician chain (Sage draws, Intermezzo pumps, Melodist scores)
- **Scorched Earth:** Abandon-tempo with figment sacrifice + fast removal + Nexus of Passing as finisher

**Tag cleanup required:** Remove Gale from Duskwatch Vigil and Voidthorn Protector archetype listings.

**Critical change recommended:** Reassign Dreamtide Cartographer from mono-Tide to mono-Zephyr.

---

### Crucible (Stone+Ember) -- FINAL: 7.6/10, B+

**V2 Verdict: The showcase improvement. Purity-vs-power tension is real and well-calibrated.**

Crucible's transformation from v1 to v2 is the most impressive archetype improvement in the format. The rails score dropping from 9/10 to 6/10 was achieved through five non-Warrior power pieces that each create a genuine "include this or maximize Blade of Unity?" decision:

| Non-Warrior | Crucible Benefit | Blade of Unity Cost |
|-------------|-----------------|-------------------|
| Ironveil Watcher (2 cost, Ancient) | 1-3 direct points per Judgment, doubles with Surge of Fury | -1 spark |
| Vanguard of the Summit (4 cost, Mage) | Draw 2 + 2 energy on 3rd character | -1 spark |
| Kindlespark Harvester (3 cost, Ancient) | Repeatable spark-to-removal conversion + kindle 2 | -1 spark |
| Ironbark Warden (4 cost, Ancient) | Board-wide +1 spark for persistent allies | -1 spark |
| Everhold Protector (2 cost, Ancient) | Passive kindle accumulation | -1 spark |

The key insight from my analysis: **Ironveil Watcher + Surge of Fury is the strongest 2-card combo in Crucible.** An extra Judgment phase doubles Ironveil's output (up to 6 direct points per turn). This is Build B's true engine and it's genuinely competitive with Build A's Blade of Unity ceiling.

**Two distinct builds:**
- **Warrior Phalanx:** 16-18 Warriors (~65-72%), Blade of Unity at 6-8 spark, lord-centric scoring
- **Crucible Engine:** 12-14 Warriors (~48-56%), Blade of Unity at 4-5 spark, Judgment storm with Ironveil Watcher + Kindlespark Harvester

Both builds still deploy Warriors and score via Judgment -- the strategic axis doesn't shift. But the tool selection and deckbuilding decisions are meaningfully different. This is the correct flexibility level for a tribal archetype.

**Forgeborn Martyr assessment:**
The signpost is excellent. The "3+ allies" death dividend (replacing v1's "by the opponent" clause) fixes the synergy audit's Weak rating by enabling self-sacrifice. Cinder can now trigger the death dividend through its own abandon effects, creating genuine cross-archetype reach. The Judgment pump (+1 spark to all Warriors) is bread-and-butter Crucible value.

**Residual concerns (all acceptable):**
- Board-wipe vulnerability is Crucible's intended weakness as the aggro archetype
- Warrior recursion (Grim Reclaimer, Speaker for the Forgotten) is adequate if not exciting
- Ember sacrifice cards create an intentional tension with persistence mechanics -- this is good design

**Tag cleanup required:** Remove Tempest from Starcatcher's archetype listing (Stone card, Tempest can't access). Remove Cinder from Kindlespark Harvester's listing (Stone card, Cinder can't access).

**No mechanical changes recommended.**

---

### Cinder (Ember+Ruin) -- FINAL: 8.4/10, A

**V2 Verdict: The format's best archetype. Constraints are features, not bugs.**

Cinder is a masterclass in archetype design. It has the deepest engine (18+ abandon cards), the clearest signals ("abandon" keyword), a model signpost (Ashfire Ritualist), and two genuinely distinct builds that use different Ruin sub-packages.

**The sacrifice-recursion loop:**
Sacrifice cheap body -> get value (spark via Forsworn Champion/Harvester of Despair, points via Fathomless Maw/The Forsaker, kindle via Infernal Ascendant, energy via Spirit Reaping/Soulbinder, draw via Desperation/Specter) -> Ashfire Ritualist returns different body from void -> replay body -> repeat. Each iteration generates multiple value types simultaneously. The once-per-turn cap on Ashfire Ritualist's kindle prevents degeneracy while still allowing the loop to generate 1 kindle + 1 retrieval per turn.

**Two distinct builds:**
- **Sacrifice Engine:** Maximize the abandon-recur loop. Infernal Ascendant + Fathomless Maw + The Forsaker as payoff suite. Kindle accumulation + direct points as win conditions. Nightmare Manifest forces symmetric sacrifice that's asymmetric in context.
- **Dissolved Aristocrats:** Build a board of dissolved-trigger bodies (Silent Avenger, Seer of the Fallen, Sunset Chronicler). The opponent faces a damned-if-you-do-damned-if-you-don't dilemma: kill them and trigger massive value, or let them accumulate spark. Apocalypse becomes a one-sided board wipe. Tidechannel Observer provides passive scoring.

**Fodder scarcity -- revised position:**
After detailed analysis, I've concluded this is a healthy constraint, not a design problem. The existing fodder tools (Aspiring Guardian at 0-cost, Nightmare Manifest at 1-cost, Radiant Trio for 3 figments, Endless Projection for ongoing figments) are sufficient IF drafted intentionally. The "problem" is really a draft skill test -- Cinder drafters who prioritize figment generators alongside sacrifice outlets will outperform those who load up purely on payoffs. This is good design: it prevents Cinder from being auto-pilot and rewards drafters who understand the engine's fuel requirements.

**The Ruin Survivor split is elegant:**
- "Cinder Survivors" (dissolved triggers): Silent Avenger, Seer of the Fallen, Dustborn Veteran, Sunset Chronicler -- want to DIE for value
- "Undertow Survivors" (tribal density): Hope's Vanguard, Soulkindler, Veil of the Wastes -- want to STAY ALIVE for synergy
- These sub-packages naturally sort between archetypes. A Cinder drafter and an Undertow drafter compete minimally within the Survivor pool.

**Tidechannel Observer is a breakout card:**
The void-velocity metric (3+ void entries this turn) naturally differentiates Undertow (trivially hits via mill), Cinder (hits via sacrifice), and Eclipse (hits via discard) from Bedrock (almost never hits, different Ruin sub-plan). This is one of the most important mechanical innovations in the v2 batch.

**Residual concerns (all acceptable):**
- Recovery from empty-board states is poor -- this is the intended counter-play against Cinder
- Banish effects (Judgment of the Blade, Soulflame Predator) bypass recursion -- healthy counterplay
- Ruin Survivor noise creates a learning curve for new drafters (~10-12 Undertow-optimized cards are trap picks)
- Ember ceiling at 45 means no room for additional support -- acceptable given 8.4/10 rating

**Tag cleanup required:** Remove Gale from Voidthorn Protector's archetype listing. Remove Cinder from Kindlespark Harvester's listing.

**No mechanical changes recommended.**

---

## Ember Advocate's Final Recommendations to the Council

### Priority 1 (MUST DO): The Three-Way Trade + Smolder Sprite

**Emerging consensus across all five advocates.** Stone proposed, Ember/Zephyr/Ruin endorsed:

| Change | From | To | Count Impact |
|--------|------|-----|-------------|
| Dreamtide Cartographer | mono-Tide | mono-Zephyr | Tide 40→39, Zephyr 43→44 |
| Data Pulse | mono-Ember | mono-Tide | Tide 39→40, Ember 45→44 |
| Smolder Sprite (NEW) | -- | mono-Ember | Ember 44→45 |

**Net result:** Tide 40→40, Zephyr 43→44, Ember 45→45. All counts within range.

**Smolder Sprite design (proposed by Ruin, endorsed by Ember):**
- 1 energy, Ember, Survivor, 0 spark, Common
- "Materialized: Materialize a figment. Abandon this character: Kindle 1."
- Serves Cinder (2 bodies for 1 energy = fodder), Gale (cheap fast deploy + figment for Windstride Runner), Crucible (non-Warrior sacrifice fodder), Tempest (cheap card-play trigger)

This package addresses:
- Gale's accessibility gap (gains Cartographer + Smolder Sprite = 2 new accessible cards)
- Eclipse's Cartographer accessibility (gains access through Zephyr)
- Cinder's fodder scarcity (cheap body designed for sacrifice-recur loops)
- Tide's count balance (Data Pulse returns; no net loss)
- Ember's ceiling constraint (Data Pulse out, Smolder Sprite in; stays at 45)

**Waiting on Tide advocate approval to finalize.**

### Priority 2 (MUST DO): Archetype Tag Corrections
The following tags create false expectations and should be corrected:

| Card | Current Tags | Corrected Tags | Reason |
|------|-------------|---------------|--------|
| Starcatcher | Tempest, Basalt | Basalt | Tempest can't access Stone |
| Duskwatch Vigil | Depths, Gale, Tempest | Depths, Tempest | Gale can't access Tide |
| Voidthorn Protector | Depths, Gale, Cinder | Depths, Cinder | Gale can't access Ruin |
| Ashen Threshold | Mirage, Cinder, Bedrock | Mirage, Cinder | Bedrock can't access Ember |
| Kindlespark Harvester | Crucible, Cinder, Depths | Crucible, Depths | Cinder can't access Stone |
| Forsworn Champion | Crucible, Cinder | Cinder (primary), Crucible (secondary) | Anti-synergistic with Crucible's lord/Blade plan; sacrifice card that happens to be Warrior |
| Dreamtide Cartographer | Gale, Tempest, Eclipse | Gale, Eclipse, Basalt (after Zephyr move) | Tempest loses access; Basalt gains access |
| Data Pulse | Tempest | Tempest, Undertow, Depths, Mirage (after Tide move) | Returns to Tide; generically useful for all Tide archetypes |

### Priority 3 (SHOULD DO): Bedrock Bridge
Propose a new Stone card with void-play triggers to strengthen the Stone-Bedrock connection. Stone is at 40 (room for more). Ember supports this through coalition agreement with Ruin.

### Priority 4 (MONITOR): Playtesting Watchpoints
- Cinder's fodder balance: Does Smolder Sprite (if approved) adequately raise Cinder's floor?
- Gale's Musician availability: Are 2+ Musicians consistently available for Gale drafters?
- Gale Build B viability under Cinder competition: When both are drafted, does Build A (Musician Crescendo) carry Gale?
- Crucible's Build B viability: Is the Judgment Engine build competitive with Warrior Phalanx?
- Crucible's sacrifice-vs-anchor tension: Do drafters self-correct into one sub-build or naively mix?
- Tempest's Legendary variance: How much do Epiphany Unfolded and Moment Rewound swing outcomes?
- Ember fast removal contention: Do 6 fast events + 3 neutral events adequately serve 2-3 Ember archetypes in the same pod?
- Cross-Ruin archetype contention: Are Architect of Memory, Path to Redemption, and Reclaimer of Lost Paths creating unhealthy pinch points?
- Cinder's Survivor learning curve: Are new drafters distinguishing dissolved-trigger from density Survivors?

---

## Overall Ember Health Score

| Archetype | Final Grade | V1 Grade | Change |
|-----------|-----------|----------|--------|
| Cinder | 8.4/10 (A) | A- | Improved |
| Tempest | 8.2/10 (A-) | A | Slight shift (homogeneity fix trades for Starcatcher loss) |
| Crucible | 7.6/10 (B+) | B+ | Stable (major internal improvement: rails 9->6) |
| Gale | 7.2/10 (B) | C+ | Significantly improved (but accessibility gap limits ceiling) |
| **Average** | **7.85/10** | **~7.1** | **+0.75 overall improvement** |

**Ember archetypes are healthy.** The v2 changes successfully addressed the major v1 problems (Tempest homogeneity, Crucible linearity, Gale signpost deficit) while maintaining the strengths of the existing designs. The single remaining critical issue -- Gale's accessibility gap -- is fixable with one card reassignment. Everything else is tag cleanup and playtesting validation.
