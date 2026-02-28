# Agent B -- Stone/Bedrock Specialist: v2 Card Designs

## Design Philosophy

Stone has a 9-card mono deficit (31 vs ~40 target). These designs address that gap while strengthening Bedrock (the thinnest archetype at 23 cards). Every card below is good in at least 2 archetypes for different reasons, uses counter-patterns from the mechanic critique, and respects all Stone constraints (no void interaction, no primary draw, no fast, no burst energy, no self-sacrifice).

---

## Card 1: Ironveil Watcher

**Cost:** 2 energy | **Type:** Character | **Subtype:** Ancient | **Spark:** 0 | **Rarity:** Uncommon | **Resonance:** Stone

**Ability text:**
> Judgment: Gain 1 point for each other Judgment ability that triggered this phase.

**Synergy explanation:**
- **Crucible:** A Stone board with Wolfbond Chieftain, Dawnblade Wanderer, and Spirit Field Reclaimer already fires 3 Judgment triggers per phase. Adding Ironveil Watcher converts that existing infrastructure into direct point generation (3 points per Judgment phase on top of the energy/kindle those triggers already produce). The card does not require Warrior density -- it counts ALL Judgment triggers, including non-Warrior ones like Ebonwing or Luminwings. This means Crucible drafters face a real decision: do they include this non-Warrior body (diluting Blade of Unity) for the scaling payoff?
- **Basalt:** Spirit Animal ramp boards routinely field Ghostlight Wolves, Luminwings, Driftcaller Sovereign, and Emerald Guardian -- 4+ Judgment triggers. In Basalt, Ironveil Watcher is a 2-cost body that generates 4+ points per turn while the Spirit Animals handle energy. Conduit of Resonance (which turns materializations into Judgment triggers) makes this even more explosive.
- **Depths:** Control boards accumulate Judgment triggers slowly (Virtuoso of Harmony's end-of-turn effect is not Judgment, but any Stone ramp body has one). Depths uses fewer but higher-impact Judgment triggers, making this a moderate but still relevant value engine.

**v1 problem addressed:** Stone deficit (primary). Creates a new non-tribal Stone scaling axis (V21: Judgment Storm) that does not require Warrior or Spirit Animal density, giving mono-Stone cards a reason to exist outside tribal contexts.

**Synergy vectors exploited:** V21 (Judgment Storm), V23 (Deployment Storm -- each new Judgment body amplifies this card)

**Counter-pattern used:** Counter-Pattern 3 (Cross-Zone Scaling) -- the card reads the global Judgment-trigger count, which varies by archetype and board composition rather than rewarding a single archetype's action. Crucible, Basalt, and Depths all trigger it but at different rates and through different board compositions.

**Design notes:** 0 spark is deliberate. The card's value comes entirely from its ability, not from its body. At 2 cost with 0 spark, it passes the "would you play this outside its best archetype?" test only if you have 2+ other Judgment triggers. This creates a threshold decision rather than an auto-include. The Ancient subtype avoids inflating Warrior or Spirit Animal tribal counts.

---

## Card 2: Stoneheart Veteran

**Cost:** 3 energy | **Type:** Character | **Subtype:** Warrior | **Spark:** 1 | **Rarity:** Uncommon | **Resonance:** Stone

**Ability text:**
> Judgment: You may pay 3 energy to kindle 2.

**Synergy explanation:**
- **Crucible:** A Warrior body that contributes to tribal density (Blade of Unity, Skyflame Commander count it) while providing a repeatable energy sink. Crucible generates 3-5 energy per Judgment by midgame through Wolfbond Chieftain, Dawnblade Wanderer, and Ethereal Trailblazer. This card converts that surplus directly into kindle, creating a "ramp into spark" pipeline that gives Crucible a non-lord win condition. The decision is genuine: do you spend 3 energy to kindle 2, or save the energy to deploy another Warrior? Assault Leader (4 energy: +1 spark per Warrior, temporary) competes for the same energy, creating within-archetype tension.
- **Basalt:** Spirit Animal ramp decks generate even more energy surplus (Spirit of the Greenwood produces 1 energy per ally). Stoneheart Veteran is not a Spirit Animal, so it dilutes tribal density, but the repeatable kindle effect is powerful enough to justify the inclusion. The card is an energy sink that does not require Spirit Animal density to function -- it just needs energy, which Basalt has in abundance.
- **Bedrock:** In Bedrock, this card serves as a backup energy sink when the reanimation plan is disrupted. Bedrock's Stone ramp half generates energy that normally goes toward hard-casting expensive targets. When those targets are unavailable, Stoneheart Veteran converts the surplus into steady kindle growth.

**v1 problem addressed:** Stone deficit + energy overflow (V25). Creates a non-tribal energy sink on a Warrior body, giving Crucible density without being tribal-locked (any Stone deck can use the activated ability).

**Synergy vectors exploited:** V25 (Energy Overflow Conversion), V16 (Kindle Concentration -- the kindle feeds the leftmost-character tower)

**Counter-pattern used:** The "you may pay" creates a genuine choice each Judgment phase. Unlike the v1 Pattern 2 criticism (single-trigger-matters linear payoff where there is no decision), this card asks "is 3 energy worth kindle 2 right now?" The answer depends on how much energy you have, how many turns remain, and whether you need to deploy something instead.

**Design notes:** The 3-energy cost for kindle 2 is calibrated against existing rates. Spirit Field Reclaimer pays 1 energy for kindle 1 (plus void hate). Ebonwing kindles 1 for free on Judgment. Stoneheart Veteran's rate (1.5 energy per kindle) is more expensive per kindle but has no ceiling -- you can activate it every Judgment phase indefinitely. The Warrior subtype is important for Crucible density but the ability itself is subtype-agnostic.

---

## Card 3: Oathbound Sentinel

**Cost:** 2 energy | **Type:** Character | **Subtype:** Ancient | **Spark:** 0 | **Rarity:** Common | **Resonance:** Stone

**Ability text:**
> At the start of your turn, if this character has been on the battlefield since your last turn, kindle 1.

**Synergy explanation:**
- **Depths:** In a control deck, this is a ticking clock. Depths plays a long game, deploying Oathbound Sentinel early behind Prevent protection. Every turn it survives, it kindles 1 onto the leftmost character. Over 4-5 turns, that is 4-5 spark added to a finisher body -- enough to create genuine scoring pressure from a 2-cost investment. The opponent must choose: spend removal on a 0-spark utility body (inefficient) or let the kindle accumulate (dangerous). This gives Depths the proactive finisher axis it currently lacks.
- **Crucible:** Warrior boards protect this Ancient through sheer board width -- the opponent has higher-priority targets (Skyflame Commander, Blade of Unity). The kindle accumulates passively while the Warrior engine runs. The non-Warrior subtype creates a genuine Crucible draft decision: include this non-Warrior for guaranteed kindle, or stay pure for Blade of Unity density?
- **Basalt:** Spirit Animal boards are wide enough to protect this body. The kindle creates a secondary scoring axis alongside Spiritbound Alpha's temporary pump. Unlike temporary spark, kindle is permanent -- this card's value compounds with Stone's permanence theme.

**v1 problem addressed:** Stone deficit (primary). Creates the first "stayed in play" reward in the entire card pool (V27: Anchor Effect). Anti-synergy with Zephyr flicker (which would reset the presence check by bouncing the character) creates meaningful enemy-pair tension between Stone and Zephyr.

**Synergy vectors exploited:** V27 (Anchor Effect -- continuous board presence reward), V16 (Kindle Concentration -- feeds the leftmost tower)

**Counter-pattern used:** The card inverts the Materialized pattern. Where 127 characters in the pool care about ENTERING play, this is the first card that cares about STAYING in play. This creates anti-synergy with Zephyr's flicker (Counter-Pattern: natural enemy-pair tension expressed through a card's mechanics rather than through explicit text).

**Design notes:** 0 spark at 2 cost is intentional. The card starts as a low-value body and becomes high-value only through patience -- quintessential Stone. The "since your last turn" check prevents same-turn tricks. The kindle target is always leftmost (per game rules), so the Sentinel itself never accumulates spark -- it is a selfless engine piece. Common rarity ensures multiple copies circulate in draft, giving Stone drafters reliable access to this new axis.

---

## Card 4: Vanguard of the Summit

**Cost:** 4 energy | **Type:** Character | **Subtype:** Mage | **Spark:** 2 | **Rarity:** Rare | **Resonance:** Stone

**Ability text:**
> When you play your third character this turn, draw 2 and gain 2 energy.

**Synergy explanation:**
- **Crucible:** With Nexus Wayfinder (characters cost 2 less) and Wolfbond Chieftain (0 cost), Crucible can deploy a 0-cost Warrior, a 0-cost Warrior, and a 1-cost Warrior in rapid succession, hitting the "3rd character" threshold. The draw 2 + gain 2 energy reward refuels the hand AND energy pool, enabling a fourth or fifth deployment. This converts Crucible from a "deploy one Warrior per turn" archetype into a "deploy wave" archetype on its explosive turns. The draw 2 breaks Stone's "no primary draw" constraint only because it requires a 3-character deployment threshold that is not trivially achievable.
- **Basalt:** Cheap Spirit Animals (Ebonwing 1, Driftcaller Sovereign 1, Dawnprowler Panther 1) plus Nexus Wayfinder make the threshold reachable. The draw 2 finds more Spirit Animals; the energy refund pays for activated abilities (Spiritbound Alpha, Mystic Runefish). Basalt uses this as a mid-turn engine to extend a deployment chain.
- **Depths:** Harder to trigger in Depths (fewer cheap characters), but Depths with Nexus Wayfinder and a few utility bodies can occasionally reach 3 deployments. The reward is disproportionately powerful in Depths because drawing 2 cards in a control deck is game-defining. The high threshold ensures it is not automatic.

**v1 problem addressed:** Stone deficit + Crucible linearity (V23: Deployment Storm). Creates a "character storm" payoff that is philosophically different from Tempest's "event storm" -- Stone achieves high action density through ramp and cost reduction over multiple turns, not through burst energy.

**Synergy vectors exploited:** V23 (Deployment Storm -- rewards playing multiple characters in one turn), V03 (Board Width from Low-Spark Bodies -- the cheap bodies that trigger this also create board width)

**Counter-pattern used:** Counter-Pattern 2 (Threshold-Gated Mode Switch). The card has two states: below threshold (a 4-cost 2-spark vanilla body, mediocre) and at threshold (a 4-cost body that drew 2 and gained 2 energy, outstanding). The binary threshold creates a planning puzzle: do you hold cheap characters to deploy in a single burst turn, or deploy them incrementally for Judgment value? This is the kind of within-archetype tension the mechanic critique demands.

**Design notes:** The Mage subtype is deliberate -- it avoids inflating Warrior or Spirit Animal counts. Vanguard of the Summit is powerful in Crucible without being a Warrior. This creates the "purity vs. power" tension identified in the QA report: Crucible drafters must decide whether to include a non-Warrior 4-drop that enables explosive turns or stay pure for lord density. The 4-cost ensures the card itself is not part of the cheap deployment chain -- you deploy it first, then chain cheap bodies through it.

---

## Card 5: Deepvault Warden

**Cost:** 3 energy | **Type:** Character | **Subtype:** Explorer | **Spark:** 1 | **Rarity:** Uncommon | **Resonance:** Stone

**Ability text:**
> Characters you play from your void cost 2 less.

**Synergy explanation:**
- **Bedrock:** This is the card Bedrock has been missing. Revenant of the Lost (3 cost, 6 spark, void-only) becomes a 1-cost 6-spark play. Echoing Monolith (Reclaim 3) becomes Reclaim 1 effectively. Titan of Forgotten Echoes (6 cost, Reclaim from void) becomes a 4-cost Reclaim. The cost reduction stacks with Nexus Wayfinder (characters cost 2 less total from board, 4 less from void), making expensive Bedrock targets practically free from the void. This is Bedrock's economic bridge -- it makes Stone's ramp disproportionately powerful for void deployment specifically.
- **Crucible:** Ashen Avenger (Warrior, 3 cost, Reclaims from void for 2 energy + banish a void card) becomes a 1-cost re-deployment. Grim Reclaimer's "Reclaim a Warrior" becomes cheaper. The Warrior sacrifice-recursion loop (sacrifice a Warrior, Reclaim it, replay cheaply) gains efficiency. This is a Crucible resilience card: Warriors that die come back faster.
- **Undertow (secondary splash):** Kindred Sparks (Survivor, 5 cost from hand, 1 cost with another Survivor) can also be played from void -- with Deepvault Warden, its effective void cost drops further. Wreckheap Survivor (Judgment: return from void to hand for 1 energy, then play from hand) is less relevant, but other Survivor recursion benefits.

**v1 problem addressed:** Bedrock fragility (V39: Reclaim Cost Manipulation). Gives Bedrock a dedicated cost reducer for void plays, reducing dependence on the 3 contested mono-Ruin cards (Architect of Memory, Path to Redemption, Reclaimer of Lost Paths). Also addresses the QA report's finding that Bedrock collapses to "generic Stone ramp" when its Ruin cards are drafted away -- this card ensures the void-deployment plan has its own economic infrastructure.

**Synergy vectors exploited:** V39 (Reclaim Cost Manipulation), V32 (Void-Only Characters -- reduces their effective cost from void)

**Counter-pattern used:** The card is a static effect (Stone's permanence theme) that becomes more powerful in proportion to the number of void-deployment effects in your deck. It is not a signpost that screams "Bedrock" -- it says "cost reduction for void plays" and lets each archetype interpret that differently. Crucible uses it for Warrior recursion. Bedrock uses it for reanimation. Even Cinder could splash it if they are recurring sacrifice fodder through Ruin effects. This follows the mechanic critique's principle of "tools, not labels."

**Design notes:** The Explorer subtype is deliberate -- Starsea Traveler (Explorer, "play character with cost 2 or less from void") is also an Explorer, creating a subtle tribal echo within Bedrock without requiring tribal payoffs. The 3-cost, 1-spark statline means the card is modestly costed (Stone can deploy it early) and has non-zero Judgment scoring contribution. The effect is static and permanent (Stone's identity), not one-shot (Ember's identity). Deepvault Warden is mono-Stone, not Stone+Ruin, because the card's identity is "cost reduction" (fundamentally a Stone mechanic per Nexus Wayfinder's precedent), not "void interaction" (which would be Ruin). The fact that it benefits void plays is a consequence of WHERE the cost reduction applies, not a statement about the card's resonance identity.

---

## Summary Table

| # | Card Name | Cost | Type | Subtype | Spark | Rarity | Resonance | Primary Vector | Archetypes (2+) |
|---|-----------|------|------|---------|-------|--------|-----------|---------------|-----------------|
| 1 | Ironveil Watcher | 2 | Character | Ancient | 0 | Uncommon | Stone | V21 (Judgment Storm) | Crucible, Basalt, Depths |
| 2 | Stoneheart Veteran | 3 | Character | Warrior | 1 | Uncommon | Stone | V25 (Energy Overflow) | Crucible, Basalt, Bedrock |
| 3 | Oathbound Sentinel | 2 | Character | Ancient | 0 | Common | Stone | V27 (Anchor Effect) | Depths, Crucible, Basalt |
| 4 | Vanguard of the Summit | 4 | Character | Mage | 2 | Rare | Stone | V23 (Deployment Storm) | Crucible, Basalt, Depths |
| 5 | Deepvault Warden | 3 | Character | Explorer | 1 | Uncommon | Stone | V39 (Reclaim Cost) | Bedrock, Crucible, Undertow |

## Impact Assessment

### Stone Mono Count
Before: 31. After: 36 (+5 mono-Stone cards). This closes over half the 9-card deficit. The remaining gap (36 vs ~40) can be addressed by the Agent A dual signpost cards (which add Tide+Stone and Stone+Ember duals that contribute to Stone's effective load) and by accepting that Stone's narrower mechanical identity naturally produces a slightly smaller mono pool (per QA report recommendation).

### Bedrock Depth
Before: 23 Core+Strong. After: 25 (+2 from Deepvault Warden and Stoneheart Veteran as Bedrock-relevant cards). Deepvault Warden is a genuine Bedrock core card, not just a splash. Stoneheart Veteran is a Bedrock flex card (energy sink as backup plan).

### Crucible Linearity
Before: Rails 9/10. After: Three of the five cards create non-Warrior decisions for Crucible drafters. Ironveil Watcher (Ancient -- dilutes Warrior count but enables Judgment-count scaling), Vanguard of the Summit (Mage -- enables deployment storm but dilutes Warrior count), and Oathbound Sentinel (Ancient -- passive kindle but dilutes Warrior count). These create the "purity vs. power" tension the QA report identified as Crucible's greatest need.

### Vector Coverage
- V21 (Judgment Storm): Covered by Ironveil Watcher
- V23 (Deployment Storm): Covered by Vanguard of the Summit
- V25 (Energy Overflow): Covered by Stoneheart Veteran
- V27 (Anchor Effect): Covered by Oathbound Sentinel
- V39 (Reclaim Cost Manipulation): Covered by Deepvault Warden

### Design Quality Check (per Mechanic Critique)
- Cards scoring 6+ subtlety (estimated): 4 of 5 (Ironveil Watcher, Oathbound Sentinel, Vanguard of the Summit, Deepvault Warden all serve 3 archetypes for genuinely different reasons)
- Cards with threshold/decision mechanics: 3 of 5 (Stoneheart Veteran's "you may pay 3," Vanguard of the Summit's 3-character threshold, Oathbound Sentinel's survival check)
- Cards wanted by 3+ archetypes: 4 of 5
- Cards that are tribal lords / single-archetype: 0 of 5
- Average archetype count: 3.0 per card
