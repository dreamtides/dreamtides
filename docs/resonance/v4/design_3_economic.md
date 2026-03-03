# Domain 3: Economic & Resource Mechanisms

## Key Takeaways

- **Player agency over *when* to influence packs is the defining feature of economic mechanisms.** Unlike probabilistic or structural approaches, the player explicitly chooses to intervene or not on each pick, creating a natural rhythm of enhanced and random packs that produces variance by design rather than by tuning.
- **The simplest economic mechanisms are also the most effective.** Algorithms that auto-accumulate a single currency and offer a single spend action (e.g., "widen the pack") are easy to explain and produce the right convergence curve. Dual-currency or multi-action systems add complexity without proportional benefit.
- **Economic mechanisms naturally solve the "on rails" problem.** Because spending is optional, the player always has the choice to save resources and draft from random packs. A player who misreads signals early can bank tokens and course-correct, unlike threshold systems where commitment is permanent.
- **The convergence challenge is real but solvable.** A purely optional spend mechanism risks players who never spend (power-chasers) or always spend (forcing). The key is calibrating earn rate and spend cost so that spending is clearly beneficial but not overwhelming -- the player should want to spend roughly half the time.
- **Symbol distribution matters more for economic mechanisms than for slot-assignment systems.** Because the player earns tokens from symbols on drafted cards, 2-symbol cards are the sweet spot: they provide enough tokens to fund regular spending without flooding the economy after a few picks.
- **Economic mechanisms enable genuine skill expression.** Knowing *when* to spend (e.g., saving tokens before a critical pivot pick, or spending heavily when you have identified the open archetype) creates a decision layer that other domains lack.
- **The biggest risk is cognitive overhead.** Even simple economic mechanisms ask the player to track a resource and make a decision before each pack. This must be weighed against the agency it provides.

---

## Proposal 1: Pack Widening

**Player-facing description:** "Each symbol you draft earns 1 matching token (primary earns 2); before seeing a pack, you may spend 3 tokens of one resonance to add a 5th card of that resonance to the pack."

**Technical description:** The player accumulates resonance tokens from every drafted card (primary symbol = 2 tokens, secondary/tertiary = 1 each). Before each pack is generated, the player may spend 3 tokens of a single resonance to widen the pack from 4 cards to 5, where the 5th card is drawn randomly from cards with that resonance as primary. The player still picks 1 card. Unspent tokens persist across picks.

**Assessment:** Excellent on simplicity (one sentence, concrete operations), flexibility (spending is optional, pivoting means switching which resonance you spend), and natural variance (non-spend packs are fully random). Convergence depends on earn/spend rate -- a committed player earning ~3 tokens per pick can spend every other pick, adding one on-resonance card to half their packs. Splash is maintained because the base 4 cards are always random. Signal reading is weak unless combined with pool asymmetry.

**Best symbol distribution:** Mostly 2-symbol cards (55%). This gives ~3 tokens per pick, funding a spend every 2 picks.

---

## Proposal 2: Resonance Auction

**Player-facing description:** "You start each quest with 10 influence points; before any pack, you may spend 1-3 influence to replace that many random pack cards with cards drawn from a specific resonance; you earn 1 influence whenever you draft a generic card."

**Technical description:** The player begins with a fixed budget of influence points. Before pack generation, they may declare a resonance and a spend amount (1-3). That many of the 4 pack cards are replaced with random cards of the declared resonance. Influence regenerates slowly (1 point per generic card drafted). The budget creates a natural spend/save tension across the 30-pick draft.

**Assessment:** Strong on agency and variance (the player chooses exactly when to concentrate packs). Weak on simplicity (two levers: which resonance and how much to spend) and convergence (the fixed budget means late-draft influence is scarce). Generics as the regeneration source is thematic but creates a perverse incentive to draft mediocre cards. The replacement mechanic is dangerously close to mechanical slot assignment.

**Best symbol distribution:** Less sensitive to distribution since the mechanic operates on a fixed budget rather than symbol-derived tokens.

---

## Proposal 3: Surplus Investment

**Player-facing description:** "Each pick, the 3 cards you don't draft are converted into 1 token matching the most common resonance among them; spend 4 tokens of one resonance to make your next pack draw from only that resonance's cards."

**Technical description:** After each pick, examine the 3 passed cards. The resonance with the most symbols among the passed cards generates 1 token (ties broken randomly). When the player accumulates 4 tokens of a single resonance, they may spend them to make the next pack draw all 4 cards exclusively from cards whose primary resonance matches. This creates an interesting dynamic: passing on a resonance cluster builds tokens for that resonance, but you might want tokens in a *different* resonance.

**Assessment:** Creative blend of rejection and economic mechanics. However, the token accumulation is confusing -- the player earns tokens from what they *don't* pick, which is counterintuitive. The "all 4 cards from one resonance" spend is too powerful and too mechanical. Convergence would be strong but at the cost of making spend-packs feel like vending machines. Signal reading is inherently supported because the passed cards carry information.

**Best symbol distribution:** Higher symbol counts (2-3) so that passed cards reliably produce tokens.

---

## Proposal 4: Tempo Banking

**Player-facing description:** "You earn 1 tempo each pick; spend 2 tempo before a pack to see 6 cards instead of 4, or spend 4 tempo to see 8 cards -- you always pick 1."

**Technical description:** The player gains 1 tempo automatically every pick. Before pack generation, they may spend tempo to widen their pack: 2 tempo for a 6-card pack, 4 tempo for an 8-card pack, or spend nothing for the default 4. All cards are drawn randomly from the full pool with no resonance bias. The wider pack increases the *probability* of seeing on-archetype cards purely through combinatorics -- an 8-card random draw from a pool where ~12.5% of cards are S/A for your archetype yields ~1.0 expected S/A cards vs. ~0.5 in a 4-card pack.

**Assessment:** Maximally simple (one sentence, one resource, one action). Produces perfect natural variance because there is zero resonance manipulation -- wider packs are just more random draws. However, convergence is weak because even 8 random cards only yield ~1.0 archetype S/A cards on average. This mechanism alone cannot hit the 2.0 S/A target without either very large pack sizes (which changes the game feel) or a complementary mechanism. Flexibility and pivot-friendliness are excellent because tempo has no resonance commitment. Splash is naturally high.

**Best symbol distribution:** Irrelevant -- this mechanism does not use symbols at all.

---

## Proposal 5: Resonance Futures

**Player-facing description:** "At any point during the draft, you may 'invest' in a resonance by paying 2 of that resonance's tokens; each active investment adds one extra card of that resonance to every future pack."

**Technical description:** The player accumulates tokens as in Proposal 1 (primary = 2, secondary/tertiary = 1). At any point, they may spend 2 tokens of a resonance to purchase a permanent "investment" in that resonance. Each investment adds one additional card of that resonance to every subsequent pack (drawn randomly from cards with that resonance as primary). Investments are permanent and cumulative -- 2 investments in Tide means every future pack has 2 extra Tide cards (making it a 6-card pack). The player still picks 1.

**Assessment:** This creates an escalating commitment curve: early investments compound over many picks, making timing a genuine strategic decision. Convergence is strong because permanent extra cards accumulate. However, the permanent nature means pivoting is costly (invested tokens are gone), and multiple investments rapidly narrow the draft. Two Tide investments by pick 8 means 22 extra Tide cards across the remaining 22 picks -- very powerful. Simplicity is moderate: the concept of "permanent pack-widening investments" is intuitive but the interaction between multiple investments and pack size needs careful explanation. Risk of degeneracy: a player who invests early and aggressively gets a strong advantage.

**Best symbol distribution:** 2-symbol cards (55%), same reasoning as Proposal 1.

---

## Champion Selection: Proposal 1 -- Pack Widening

Pack Widening is the champion because it best balances the competing demands of V4:

1. **Simplicity:** One sentence captures the entire algorithm. "Earn tokens from symbols, spend tokens to add a card." A programmer can implement it from the description alone.

2. **Natural variance:** Non-spend packs are 100% random. Spend packs add exactly one resonance-biased card to an otherwise random pack. This creates a gentle slope of influence rather than a binary on/off switch.

3. **Agency without rails:** The player chooses when to spend, creating a strategic layer. But the spend is small enough (one extra card) that it does not feel mechanical. A spend-pack with 5 cards (4 random + 1 resonance) still has 4 fully random cards.

4. **Pivot-friendly:** Tokens accumulate in all drafted resonances. A player who starts Ember-heavy can spend Tide tokens later if they pivot. The cost is only the unspent Ember tokens, not a permanent commitment.

5. **Convergence path:** A committed player earning ~3 tokens per pick in their primary resonance can spend every other pick. This adds 1 on-resonance card to roughly 50% of packs. Combined with the natural base rate of seeing archetype cards, this should approach the 2.0 S/A target -- though simulation is needed to confirm.

Proposals 4 (Tempo Banking) was close but fails convergence. Proposal 5 (Resonance Futures) is interesting but the permanent investments create an "on rails" dynamic that V4 is trying to avoid. Proposals 2 and 3 have complexity or perverse incentive problems.

---

## Champion Deep-Dive: Pack Widening

### Example Draft Sequences

**Early Committer (Warriors/Tide-Zephyr):**
- Picks 1-3: Drafts Tide-primary cards. Earns ~6 Tide tokens, ~3 Zephyr tokens.
- Pick 4: Spends 3 Tide tokens. Pack is 5 cards: 4 random + 1 Tide-primary card. The Tide card might be Warriors, Sacrifice, Self-Mill, or Ramp -- not guaranteed Warriors, but ~50% chance of S/A.
- Picks 5-6: Earns more Tide tokens. Spends again at pick 6.
- Picks 7-30: Alternates between spend and save. Roughly 12 of 24 remaining packs have a 5th Tide card. On average, this adds ~6 additional S/A archetype cards seen across the draft.

**Flexible Player:**
- Picks 1-5: Drafts strong cards across resonances. Accumulates a spread: ~4 Tide, ~3 Ember, ~2 Stone, ~2 Zephyr.
- Picks 6-8: Still exploring. Spends 3 Ember tokens on pick 6 to explore Storm/Blink. Spends 3 Tide tokens on pick 8 to explore Warriors/Sacrifice.
- Pick 9: Commits to Warriors. Has ~5 Tide tokens remaining. Begins focused spending.
- Picks 10-30: Has fewer tokens than the early committer (spent some exploring) but still funds ~8-10 Tide spend-packs across the remaining draft.

**Pivot Attempt:**
- Picks 1-5: Drafts Ember-primary cards. Accumulates ~8 Ember tokens.
- Pick 6: Realizes Storm is being competed for (few Storm cards appearing). Decides to pivot to Warriors (Tide/Zephyr).
- Picks 6-10: Starts drafting Tide cards. The 8 Ember tokens are stranded (usable only for Ember packs, which the player no longer wants). But new Tide tokens accumulate at ~3 per pick.
- Pick 9: Has ~9 Tide tokens after 3 Tide-focused picks. Can already fund 3 Tide spend-packs.
- Picks 10-30: Catch-up is possible but slower -- the early committer had a ~4 pick head start on Tide tokens. The pivot player sees slightly fewer enhanced packs (roughly 8-9 vs. 12 for the early committer).

### Failure Modes

1. **Convergence shortfall.** The extra 5th card is one card drawn from a resonance shared by 4 archetypes. Only ~50% of the time will it be S/A for the target archetype. If the base 4 random cards average ~0.5 S/A, and the 5th card adds ~0.5 S/A on spend picks, the average across all picks (spending ~50% of the time) is roughly 0.5 + 0.25 = 0.75 S/A per pack. This is below the 2.0 target. **Mitigation:** Increase token earn rate or decrease spend cost to enable spending on more picks. Or increase the spend bonus (add 2 cards instead of 1).

2. **Cognitive load.** Tracking token counts in 4 resonances and deciding when to spend adds mental overhead. **Mitigation:** UI shows token counts prominently. Alternatively, simplify to a single "draft influence" currency that is not resonance-specific (but this loses the resonance-targeting benefit).

3. **Power-chaser exploitation.** A player who never spends tokens gets perfectly random packs, which is fine -- but a player who always spends on the same resonance from pick 1 might force an archetype. **Mitigation:** The cost of 3 tokens per spend and the fact that only ~50% of resonance cards are S/A for any specific archetype limits forcing. The player still needs to find the right cards within the resonance.

4. **Token inflation.** Late-draft players may have more tokens than they can spend, making the spend decision trivial (always spend). **Mitigation:** Increase spend cost over time, or cap token accumulation.

### Parameter Variants Worth Testing

1. **Spend cost: 2 vs. 3 vs. 4 tokens.** Lower cost enables more frequent spending (more convergence, less variance). Higher cost creates more save/spend tension (more variance, less convergence). The sweet spot is where a committed player spends on roughly 50-60% of packs.

2. **Bonus card count: 1 vs. 2 extra cards per spend.** Adding 2 cards per spend doubles the convergence effect but makes spend-packs significantly different from non-spend packs. Adding 1 card is subtler -- the pack goes from 4 to 5 cards, a noticeable but not dramatic change.

3. **Token earn rate multiplier: primary = 2/1/1 vs. 3/1/1 vs. 2/2/1.** Higher primary weight concentrates tokens in the player's most-drafted resonance, speeding convergence. Flatter weights spread tokens across resonances, supporting flexibility but slowing commitment.

### Proposed Symbol Distribution

| Symbol Count | % of non-generic | Cards |
|---|---|---|
| 0 (generic) | -- | 36 |
| 1 symbol | 20% | 65 |
| 2 symbols | 55% | 178 |
| 3 symbols | 25% | 81 |

**Rationale:** 2-symbol cards dominate to provide a reliable ~3 tokens per pick. The 25% at 3 symbols creates occasional high-earn picks (4 tokens) that let a committed player occasionally double-spend. The 20% at 1 symbol provides low-earn picks that create natural token income variance. This distribution yields an average of ~3.1 tokens per pick for a committed player, funding a spend every pick at cost 3 -- but since not every card drafted will be on-resonance, the effective on-resonance earn rate is lower, creating the desired save/spend rhythm.
