# Zephyr Advocate Position Paper -- Design Council Round 10

## Executive Summary

Zephyr sits at the crossroads of four distinct archetypes: **Mirage** (Tide+Zephyr), **Gale** (Zephyr+Ember), **Basalt** (Zephyr+Stone), and **Eclipse** (Zephyr+Ruin). As the resonance with the second-highest mono count (43 cards), Zephyr provides deep creature-based infrastructure across all four of its archetypes. The v2 redesign has been largely successful, but three structural issues demand attention.

---

## 1. Archetype-by-Archetype Assessment

### Mirage (Tide+Zephyr) -- Grade: A- (8.2/10) -- HEALTHY

Mirage is Zephyr's flagship success story. The flicker engine is mechanically coherent (9/10), deeply supported (~25 core cards), and has two genuinely distinct builds (Tempo Flicker vs. Engine Flicker). Tideweaver Adept is a top-3 signpost in the format -- its modal design creates real decisions and signals the archetype clearly.

**Remaining concerns:**
- Removal dependency on neutral pool (no in-color hard removal)
- Pallid Arbiter vulnerability (shuts down the entire engine)
- Spark ceiling without dedicated finisher (relies on Nexus of Passing or Aurora Rider)

**Assessment: No changes needed.** Mirage is working as intended. The vulnerabilities are healthy counterplay dynamics, not design flaws.

---

### Gale (Zephyr+Ember) -- Grade: B (7.2/10) -- NEEDS HELP

Gale is the archetype I am most concerned about. Despite having the second-largest accessible pool (113 cards), its *effective* on-plan pool is significantly smaller due to Basalt Spirit Animal bloat in Zephyr (~12-14 cards Gale can't use) and Tempest/Crucible/Cinder cards in Ember that are off-plan. The on-plan count is closer to 70-76 cards.

**Critical structural problem:** Three cards explicitly designed for Gale are inaccessible:
- **Dreamtide Cartographer** (mono-Tide) -- Gale can't access Tide
- **Duskwatch Vigil** (mono-Tide) -- Gale can't access Tide
- **Voidthorn Protector** (mono-Ruin) -- Gale can't access Ruin

This means Gale received only **3 directly accessible new cards** (Windstride Runner, Ashen Threshold, Nexus of Passing), and only 1 of those (Windstride Runner) was specifically designed for Gale. This is the fewest of any archetype.

**Additional concerns:**
- Musician subtype concentration: only 4 cards deep (Sage of the Prelude, Intermezzo Balladeer, Melodist of the Finale, Minstrel of Falling Light). If 2+ are taken by other drafters, Gale loses its payoff infrastructure.
- Deckbuilding flexibility: 6/10 -- the archetype is linear ("is this card fast?" as the primary filter)
- No Prevent access without splash, despite Duskwatch Vigil being designed as a Gale Prevent-tempo bridge

**My primary proposal:** Move Dreamtide Cartographer from mono-Tide to mono-Zephyr. This is the highest-impact single change available:
- Gives Gale a hand-emptying draw engine that rewards its natural play pattern
- Also benefits Eclipse (Zephyr+Ruin) and Basalt (Zephyr+Stone)
- Tempest (Tide+Ember) loses a supplementary card but has 9/10 core depth already
- The hand-emptying draw mode ("3 or fewer in hand: draw 2") is more Zephyr-coded than Tide-coded

**Secondary proposal:** Clean up archetype tags -- remove "Gale" from Duskwatch Vigil and Voidthorn Protector since Gale cannot draft them.

---

### Basalt (Zephyr+Stone) -- Grade: B+ (7.8/10) -- MONITOR

Basalt is mechanically excellent (9/10 coherence, 9/10 core depth) with one significant design concern: auto-build convergence. With 33+ on-plan cards for ~25-card decks, the archetype risks feeling like it drafts itself. The tribal density is so high that marginal decisions are mostly "which Spirit Animal over which Spirit Animal" rather than genuinely different strategic choices.

**What's working:**
- Spirit Animal tribal is the cleanest tribal package in the format (17/19 SAs in Zephyr+Stone)
- The energy ramp -> activated ability -> spark conversion pipeline is tight
- Two builds exist (Spirit Engine vs. Judgment Storm), though they share ~80% of the same card pool
- Stoneveil Guardian is a functional signpost with genuine mode-switching

**What concerns me:**
- Deckbuilding flexibility: 6/10 -- tied for lowest with Gale and Eclipse
- The v2 non-SA Stone cards (Ironveil Watcher, Stoneheart Veteran, Vanguard of the Summit) partially address the auto-build by creating "dilute tribal for power" decisions, but the core tribal package is still overwhelmingly deep
- Mass removal fragility: SA bodies at 1 spark are trivially wiped by Burst of Obliteration paying 2 energy
- Lost 2 dual signposts from v1 (Conduit of Resonance and Blazing Emberwing moved to mono-Zephyr)

**Assessment:** Basalt is strong enough to be a healthy draft archetype. The auto-build risk is a replay-value concern, not a power-level problem. I would not make changes here unless other proposals create opportunities to address it incidentally.

---

### Eclipse (Zephyr+Ruin) -- Grade: B (7.2/10) -- IMPROVED BUT FRAGILE

Eclipse has substantially improved from v1's "thin payoff layer" to a genuine engine with 11 dedicated discard payoffs + Voidthread Weaver. The improvement is real: v1 Eclipse was a C-/D+ archetype that players fell into by accident. V2 Eclipse is a coherent B-tier strategy.

**What's working:**
- Voidthread Weaver is an excellent signpost (8/10) -- a complete engine on a single card
- Ridge Vortex Explorer (discard = free materialize) is a distinctive payoff unique to Eclipse
- Tidechannel Observer provides void-velocity scoring Eclipse can trigger trivially (3+ discards/turn)
- Clean draft signals: discard-payoff Ruin cards unambiguously signal Eclipse rather than Undertow

**Remaining concerns:**
- All payoffs concentrated in Ruin -- if another Ruin drafter takes them, Eclipse loses its engine
- Dreamtide Cartographer lists Eclipse as primary archetype but Eclipse (Zephyr+Ruin) cannot access mono-Tide cards
- Limited removal (no in-resonance unconditional removal)
- No discard-matters events -- all payoffs are on characters, making the engine vulnerable to dissolve effects
- Deckbuilding flexibility: 6/10 -- two builds (Void Weaver vs. Discard Aggro) but both on the same axis

**My proposal for Eclipse:** The Dreamtide Cartographer move to mono-Zephyr (proposed above for Gale) also directly benefits Eclipse. The hand-emptying draw mode rewards Eclipse's discard-heavy play pattern. This single change helps two of my archetypes simultaneously.

---

## 2. Cross-Cutting Issues

### The Dreamtide Cartographer Question

This is my #1 priority. Moving Dreamtide Cartographer from mono-Tide to mono-Zephyr:
- **Helps:** Gale, Eclipse, Mirage, Basalt (all Zephyr archetypes)
- **Hurts:** Tempest, Undertow, Depths (Tide archetypes lose access)
- **Tide's counter-argument:** Tempest has 9/10 core depth and Depths' health check scores 7/10. They can absorb the loss.
- **Zephyr's argument:** Gale's effective on-plan pool is already the smallest, and Dreamtide Cartographer's hand-emptying mode is mechanically Zephyr-coded.

I expect the Tide advocate to push back on this. My position is that the net benefit (helping 2 struggling archetypes -- Gale at B and Eclipse at B) outweighs the marginal loss to 2 healthy archetypes (Tempest at A- and Undertow at A-).

### Starcatcher Contradiction

Starcatcher is tagged "Tempest, Basalt" but is in Stone, so Tempest (Tide+Ember) can't access it. As the Zephyr advocate, I note that Starcatcher being in Stone *helps* Basalt (Zephyr+Stone can access it). I'm inclined to support Starcatcher staying in Stone and having Tempest removed from its archetype tag, unless the Tide/Ember advocates propose a compelling trade.

### Zephyr Pool Composition

Zephyr's 43 cards break down roughly as:
- ~12-14 Spirit Animal tribal (Basalt-specific)
- ~7-8 fast-matters cards (Gale-oriented)
- ~8-10 flicker cards (Mirage-oriented)
- ~6-7 discard cycling cards (Eclipse-oriented)
- ~5-6 generically useful cards

This spread means each Zephyr archetype has a reasonable "home" slice within the Zephyr pool, with minimal cannibalization between them. I consider this well-balanced and do not propose changes to existing Zephyr card assignments.

---

## 3. Specific Proposals

### Proposal 1: Move Dreamtide Cartographer to Mono-Zephyr (PRIORITY)
- Current: Mono-Tide, listed for Gale/Tempest/Eclipse
- Proposed: Mono-Zephyr, listed for Gale/Eclipse/Basalt
- Rationale: Hand-emptying draw is Zephyr-coded; fixes the accessibility gap for Gale and Eclipse

### Proposal 2: Clean Up Inaccessible Archetype Tags
- Remove "Gale" from Duskwatch Vigil's primary archetype list
- Remove "Gale" from Voidthorn Protector's primary archetype list
- Remove "Eclipse" from Dreamtide Cartographer's list if it stays Tide (or adjust if it moves)
- Remove "Bedrock" from Ashen Threshold's list (Bedrock can't access mono-Ember)

### Proposal 3: Support Starcatcher Staying in Stone
- Benefits Basalt (my archetype)
- Tempest can absorb the loss (9/10 core depth)
- Accept removing "Tempest" from Starcatcher's archetype tag

### Proposal 4: Consider a New Mono-Ember Card for Gale's Fast-Matters Plan
- Gale's Musician package is only 4 cards deep, all in Zephyr
- A 2-cost Ember card with "When you play a fast card, kindle 1" would give Gale cross-resonance payoff redundancy
- This is a lower-priority "nice to have" -- the Dreamtide Cartographer move is more impactful

### Proposal 5: Monitor Basalt Auto-Build in Playtesting
- No immediate changes needed
- Flag for post-playtest evaluation: if Basalt decks are too homogeneous, consider converting 1-2 Spirit Animals to non-SA subtypes to reduce tribal density
