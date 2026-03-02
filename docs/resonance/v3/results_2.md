# Results: Balanced Pack with Majority Bonus (Agent 2, Corrected)

## One-Sentence Algorithm

"Each pack has one card per resonance type, but if you have a clear majority resonance (strictly more weighted symbols than any other, counting primary=2), it replaces one random non-majority slot, giving you 2 of your majority resonance."

## Target Scorecard

Archetype-committed strategy, 1000 drafts, **archetype-level metrics**:

| Metric | Target | Actual | Pass? |
|--------|--------|--------|-------|
| Picks 1-5: unique archetypes w/ S/A card per pack | >= 3 | 7.24 | PASS |
| Picks 1-5: S/A cards for emerging archetype per pack | <= 2 | N/A* | PASS |
| Picks 6+: S/A cards for committed archetype per pack | >= 2 | 2.07 | PASS |
| Picks 6+: C/F-tier cards per pack | >= 0.5 | 0.67 | PASS |
| Convergence pick (first 2+ S/A for archetype) | 5-8 | 6.3 | PASS |
| Deck archetype concentration | 60-80% | 93.8% | FAIL |
| Run-to-run overlap | < 40% | 5.5% | PASS |
| Max archetype frequency | <= 20% | 23.8% | FAIL |
| Min archetype frequency | >= 5% | 8.2% | PASS |

**7/9 passed.** Two failures: (1) Concentration 93.8% -- each resonance is primary for 2 adjacent archetypes that are mutually S/A-tier, so the resonance-to-archetype funnel loses very little. Nearly every picked card is S/A. (2) Flash at 23.8% -- the power-based early picks create a slight Zephyr/Flash bias.

*Early S/A reads N/A because the committed player has no target during picks 1-5.

**Key correction insight:** The original measured "unique resonances per pack" at 3.72. The archetype-level metric reads **7.24** because each card is S/A for ~3 archetypes (home + adjacent). A 4-card balanced pack trivially covers 7-8 archetypes. This metric no longer meaningfully constrains.

## Symbol Distribution

30% 1-symbol (96), 55% 2-symbol (180), 15% 3-symbol (48), 36 generic. Total 360. Minimal impact: mostly-1-sym to mostly-3-sym shifts late S/A from 2.01 to 2.18 and concentration from 93.4% to 95.0%. The structural guarantee dominates.

## Parameter Sensitivity

**Majority threshold (0/3/5):** Higher thresholds delay convergence (6.3 to 6.9) and marginally reduce concentration (94.2% to 93.3%). Late S/A drops from 2.09 to 2.02. Little leverage because even the balanced 1/1/1/1 pack delivers ~1.5 S/A cards for a committed archetype.

**Variant A (dual override 2/2/0/0):** Late S/A jumps to 2.60 but C/F crashes to 0.19, failing splash badly. Not recommended.

**Variant B (threshold gate, override at 5+):** Nearly identical to standard (late S/A 2.06, C/F 0.67). Delays majority by ~1 pick. Marginal.

## Draft Traces

**Early Committer (Flash):** Picks power through 5, accumulates Zephyr. Commits pick 6 with Zephyr=6. Majority activates, 2 Zephyr slots per pack. Final: 27/30 S/A (90%), 15 Flash + 8 Ramp + 4 Blink. Confirms over-concentration.

**Flexible Player (Power-Chaser):** Always picks highest power. Stone accumulates by chance; by pick 5, Stone=5 triggers majority. Ends with 13 Self-Discard + 6 Self-Mill despite no strategy. Majority bonus is a strong gravitational force even for unfocused players.

**Signal Reader (Blink):** Reads Ember early, commits Blink pick 6 with Ember=12. Double Ember slots feed Blink/Storm S/A cards every pack. Final: 29/30 S/A (97%). Signal-reader strategy shows extreme archetype skew: Blink 58.5% of all runs, revealing a feedback loop.

## Self-Assessment

| Goal | Score | Justification |
|------|-------|---------------|
| 1. Simple | **9** | Two pack states (1/1/1/1 or 2/1/1/0), predictable from one count. |
| 2. Not on rails | **6** | 2.07 S/A of 4 means ~2 real choices per pack. |
| 3. No forced decks | **6** | 93.8% concentration means decks are homogeneous within archetype. |
| 4. Flexible archetypes | **5** | Majority operates at resonance, not archetype; can't support specific hybrids. |
| 5. Convergent | **7** | 2.07 S/A per pack, convergence pick 6.3 -- meets targets but barely. |
| 6. Splashable | **7** | 0.67 C/F passes, but "splash" cards are usually still S/A for an adjacent archetype. |
| 7. Open early | **5** | 7.24 archetypes trivially high; metric doesn't constrain because cards are S/A for ~3 archetypes each. |
| 8. Signal reading | **3** | No pool mechanism; reads only player's own picks. |

**Overall:** The archetype-level correction reveals this algorithm over-converges. "Matching your resonance" almost always means "S/A for your archetype" because adjacent archetypes share resonances. The original resonance-level metrics hid this. The core mechanism works but concentration must come down.
