# Agent 3 -- The Mechanic's Eye: v1 New Card Design Critique

## Objective

Evaluate all 18 v1 new card designs for mechanical subtlety, identify the design patterns that make them feel "on-rails," and propose counter-patterns that create non-obvious, multi-archetype synergy. Every card below is a candidate for deletion and replacement.

---

## Part 1: Card-by-Card Mechanical Subtlety Evaluation

### 1. Resonance Spark (1 cost, Event, Tide+Ember)

**Ability:** Gain 1 energy. Draw 1. If you have played 2+ events this turn, gain 1 additional energy.

**Mechanical Subtlety Score: 3/10**

**Why this score:** The card says "I am a storm chain link" in bold letters. The conditional ("2+ events this turn") is literally the Tempest archetype's mechanical identity restated as a trigger. There is zero ambiguity about where this card goes. The baseline (cantrip + 1 energy) is clean design, but the conditional bonus is a single-archetype neon sign.

**Archetype lock-in:** Tempest (primary), with marginal utility as generic cantrip in any deck. But no other archetype specifically *wants* the conditional bonus. Mirage does not play 2+ events a turn. Undertow does not chain events. The conditional is dead text outside Tempest. **Effectively 1 archetype.**

---

### 2. Storm Conduit (2 cost, Character -- Mage, Tide+Ember)

**Ability:** When you play an event, gain 1 energy. When you play your 3rd event in a turn, draw 2.

**Mechanical Subtlety Score: 2/10**

**Why this score:** The design notes literally say this card "screams 'I am the storm deck.'" It is difficult to score a card highly on subtlety when its own designer acknowledges it screams its archetype. The "when you play an event" trigger is Tempest-exclusive language. The "3rd event" threshold is achievable only in dedicated storm builds. This is a tribal lord for events-matter decks, and there is exactly one events-matter archetype.

**Archetype lock-in:** Tempest only. No other archetype plays 3 events in a single turn as a baseline game plan. The per-event energy gain has marginal value in any event-playing deck, but the draw-2 on 3rd event is Tempest-exclusive. **1 archetype.**

---

### 3. Volatile Inscription (0 cost, Event, Tide+Ember)

**Ability:** Draw 1. Opponent gains 1 point. If you have played 2+ events this turn, gain 1 energy.

**Mechanical Subtlety Score: 3/10**

**Why this score:** Another "2+ events this turn" conditional, another Tempest-only card. The drawback (opponent gains 1 point) is interesting design that creates a real cost, but the conditional energy means this card is only *good* in storm chains. Outside Tempest, it is "Draw 1, give opponent a free point" -- actively bad. The 0-cost makes it a required inclusion for storm and an unplayable card everywhere else.

**Archetype lock-in:** Tempest only. In any other deck, paying a point to draw 1 is terrible value. The conditional energy is dead outside storm turns. **1 archetype.**

---

### 4. Leyline Detonation (3 cost, Event, Tide+Ember)

**Ability:** Gain energy equal to the number of events in your void.

**Mechanical Subtlety Score: 4/10**

**Why this score:** This is slightly more subtle than the other Tempest cards because it counts events *in the void*, not events *played this turn*. This creates a bridge to any archetype that puts events into the void -- Undertow (mill) and Eclipse (discard-cycling) both fill voids. A late-game Eclipse deck with 6-7 events in the void could use this as a burst. However, the card costs 3 energy and only produces energy (no draw, no board impact), making it a pure combo enabler that only matters if you plan to spend that energy immediately on more events. That is Tempest.

**Archetype lock-in:** Tempest (primary), with marginal bridge to Undertow and Eclipse as a late-game energy burst. The bridge is real but weak -- Undertow and Eclipse do not need raw energy as much as they need specific effects. **1.5 archetypes.**

---

### 5. Arcane Refraction (1 cost, Event, Tide)

**Ability:** Foresee 2. If you have played 2+ events this turn, draw 1.

**Mechanical Subtlety Score: 5/10**

**Why this score:** This is the best of the Tempest batch because the baseline (Foresee 2 for 1 energy) is genuinely useful in *any* Tide deck. Depths wants it for card selection. Mirage wants it to find flicker targets. Undertow wants it to send cards to the void. The conditional draw is gravy in those decks rather than the reason to play it. This is mono-Tide (correct), and the Foresee baseline creates multi-archetype demand.

**Archetype lock-in:** Tempest (best home), Depths (card selection), Mirage (finding targets), Undertow (Foresee = void filling). **3-4 archetypes,** though the conditional is still Tempest-only language.

---

### 6. Shimmer Step (2 cost, Fast Event, Tide+Zephyr)

**Ability:** Banish an ally, then materialize it. Draw 1.

**Mechanical Subtlety Score: 6/10**

**Why this score:** Clean and modular. The flicker is Mirage's core mechanic, but the card has secondary applications: save a creature from removal (any deck with valuable bodies), re-trigger ANY Materialized ability (not just draw -- could re-trigger Materialized removal like Cosmic Puppeteer, or Materialized mill like Searcher in the Mists for Undertow), and the cantrip means it is never dead. Fast speed means defensive tricks. The subtlety comes from the fact that the *target* of the flicker determines which archetype benefits -- the card itself is a tool, not a directive.

However, the core pattern (flicker + draw) is still the Mirage blueprint stated plainly. A more subtle version might offer a choice or have conditional secondary effects.

**Archetype lock-in:** Mirage (primary), but useful in any deck with strong Materialized abilities. Basalt (re-trigger Spirit Animal ETBs), Tempest (re-trigger Ashlight Caller or Starcatcher), Depths (re-trigger Paradox Enforcer). **3-4 archetypes.**

---

### 7. Veil Dancer (3 cost, Character, Tide+Zephyr)

**Ability:** Once per turn, when an ally's Materialized ability triggers, Foresee 1.

**Mechanical Subtlety Score: 4/10**

**Why this score:** This is a passive value engine that only works in decks with frequent Materialized triggers. That means Mirage (flicker) and Basalt (Spirit Animal deployment) -- but in practice, it is a Mirage card because Basalt's Spirit Animals often have Judgment triggers, not Materialized triggers. The "once per turn" prevents combo loops but also limits the card to providing marginal value. Foresee 1 per turn is not exciting enough to build around; it is a small bonus for doing what Mirage already does.

The card is transparent: if you are flickering things, you get Foresee. If you are not, this is a 3-cost 1-spark vanilla. No hidden synergies, no multi-use modes.

**Archetype lock-in:** Mirage (primary), Basalt (marginal). **1.5 archetypes.**

---

### 8. Tidecaller of the Drowned (3 cost, Character -- Survivor, Tide+Ruin)

**Ability:** Materialized: Put the top 2 cards of your deck into your void. Draw 1 for each Survivor put into void this way.

**Mechanical Subtlety Score: 5/10**

**Why this score:** The Survivor-conditional draw creates genuine variance and deckbuilding tension -- you need high Survivor density for reliable draws, which limits your flexibility. This is good design. The self-mill is useful in Undertow (void volume), the Materialized trigger makes it a flicker target for Mirage, and the Survivor typing gives it tribal density. The subtlety is in the tension between wanting Survivor density (for draw reliability) and wanting diverse effects (for deck power).

However, the card is a transparent Undertow signpost. Mill + Survivor tribal + conditional draw = Undertow 101. No surprising interactions emerge.

**Archetype lock-in:** Undertow (primary), Mirage (flicker target for mill), Cinder (Survivor sacrifice fodder with ETB value). **2-3 archetypes.**

---

### 9. Tidestone Arbiter (4 cost, Character -- Ancient, Tide+Stone)

**Ability:** Judgment: Draw 1. Opponent's characters cost 1 more.

**Mechanical Subtlety Score: 5/10**

**Why this score:** Two cleanly separated halves -- Tide (draw) and Stone (tax) -- stapled together. The tax effect is interesting because it is proactive denial (Stone) rather than reactive denial (Tide), creating a philosophical bridge. The Judgment draw means the card gets better the longer it stays in play (Stone's permanence principle). However, both effects are pure control tools with no secondary applications. No aggressive deck wants this. No combo deck wants this. It does exactly one thing: slow the game down and give you cards. Clean, but obvious.

**Archetype lock-in:** Depths (primary), with marginal utility as a generic control finisher. Basalt does not want to pay 4 for a non-Spirit Animal. Crucible does not need the tax. **1.5 archetypes.**

---

### 10. Depths of Forgetting (3 cost, Event, Tide)

**Ability:** Banish an enemy. Its controller draws 1.

**Mechanical Subtlety Score: 6/10**

**Why this score:** The Swords-to-Plowshares design pattern (efficient removal with opponent compensation) is inherently interesting because it creates a decision: is the removal worth the card you give them? This decision is more meaningful against opponents with few cards in hand (compensation matters less) versus full hands (compensation matters more). The Banish (not Dissolve) respects the Ember boundary. Mono-Tide is correct.

The subtlety is in the meta-game implications. This card is better against opponents who are topdecking (they draw 1 but had nothing anyway) and worse against opponents with full hands (they draw into more options). This inverts the usual removal calculus where you want to remove big threats -- here, you want to remove things when the opponent is resource-poor.

**Archetype lock-in:** Depths (primary), but any Tide deck wants efficient removal. Mirage wants it to protect flicker targets. Tempest might take it as cheap interaction. **3+ archetypes,** though it competes with existing removal options.

---

### 11. Tempest Striker (3 cost, Fast Character, Zephyr+Ember)

**Ability:** When you play a fast card, this character gains +1 spark until end of turn.

**Mechanical Subtlety Score: 4/10**

**Why this score:** "When you play a fast card" is the Gale archetype trigger stated explicitly. The card is a fast-matters payoff for a fast-matters deck. It functions identically to the existing Musician trio (Sage of the Prelude draws on fast play, Intermezzo Balladeer gains spark on fast play) but with temporary spark instead of permanent spark. The design adds nothing mechanically new to the archetype -- it is a fourth copy of the same pattern.

The slight subtlety is that Moonlit Dancer can make ALL characters fast, which means this card technically grows when you play any character in a Moonlit Dancer deck. But that interaction requires a specific two-card combo and is obvious once you have both pieces.

**Archetype lock-in:** Gale only. No other archetype plays enough fast cards to make the trigger meaningful. **1 archetype.**

---

### 12. Voidweave Dancer (3 cost, Character, Zephyr+Ruin)

**Ability:** When you discard a card, this character gains +1 spark until end of turn.

**Mechanical Subtlety Score: 3/10**

**Why this score:** "When you discard a card" is the Eclipse archetype trigger stated explicitly. This is the discard-matters version of Tempest Striker -- same template, different trigger. The temporary spark is a safety valve but does not create interesting decisions. You discard things, it grows, you win Judgment. The card has exactly one axis of interaction.

The design is a linear payoff with no cap, no threshold, no choice. Every discard makes it bigger. There is nothing to optimize or discover -- just discard more.

**Archetype lock-in:** Eclipse only. Zephyr cycling decks without Ruin payoffs have no reason to draft this. **1 archetype.**

---

### 13. Memory Unraveler (2 cost, Character, Zephyr+Ruin)

**Ability:** Once per turn, discard a card: Return a card from your void to your hand.

**Mechanical Subtlety Score: 7/10**

**Why this score:** This is the best-designed new card in the batch. It creates a genuine engine with multiple uses:

- **Eclipse:** The core loop -- discard a bad card, retrieve a good one, triggering discard payoffs along the way.
- **Bedrock:** Discard an expensive reanimation target into the void, then retrieve it later (or retrieve Entomb to do it again).
- **Cinder:** Retrieve sacrifice fodder from the void to sacrifice again.
- **Undertow:** Retrieve milled Survivors that you actually wanted in hand.
- **Tempest:** Retrieve a key event from the void to replay.

The card is a universal tool because the discard-to-retrieve exchange is fundamentally flexible. What you discard, what you retrieve, and why you are doing it differs completely by archetype. The once-per-turn limit prevents degenerate loops while still enabling consistent value. The 2-cost, 1-spark body is appropriately modest for an engine piece.

**Archetype lock-in:** Eclipse (primary), Bedrock, Cinder, Undertow, Tempest (secondary). **4-5 archetypes.** This is the standard all the other new cards should aspire to.

---

### 14. Verdant Packmother (3 cost, Character -- Spirit Animal, Zephyr+Stone)

**Ability:** When you materialize an allied Spirit Animal, gain 1 energy. Judgment: Each allied Spirit Animal gains +1 spark until end of turn.

**Mechanical Subtlety Score: 3/10**

**Why this score:** This is a Spirit Animal tribal lord that only works in Spirit Animal decks. The Materialized trigger requires Spirit Animals. The Judgment boost requires Spirit Animals. Outside of Basalt, this card is a 3-cost 1-spark vanilla. The "until end of turn" on the Judgment boost prevents permanent snowballing but does not create interesting decisions -- you always want to have more Spirit Animals.

Both abilities are linear scaling: more Spirit Animals = more energy, more spark. No thresholds, no choices, no alternative uses.

**Archetype lock-in:** Basalt only. No other archetype runs enough Spirit Animals to make either ability relevant. **1 archetype** (acceptable for a tribal lord per the multi-archetype exception, but still a mechanical subtlety failure).

---

### 15. Warbond Sentinel (3 cost, Character -- Warrior, Stone+Ember)

**Ability:** Allied Warriors have +1 spark. When an allied Warrior is dissolved, gain 1 energy.

**Mechanical Subtlety Score: 5/10**

**Why this score:** The two abilities create a genuine tension for the opponent: removing Warriors gives you energy, but leaving them alive means they have +1 spark. This is a design success -- the card creates a lose-lose for the opponent that makes decisions interesting. The "dissolved" trigger (not "abandoned") means it fires on opponent removal, not your own sacrifice, which distinguishes it from Cinder's sacrifice payoffs.

However, the card is still a transparent Warrior tribal lord. Only Warrior decks want the +1 spark, and only Warrior-dense boards make the dissolved trigger relevant. The opponent-facing tension is good design, but the player using the card has no decisions to make -- you just deploy Warriors and let the card do its thing.

The dissolved trigger does create a marginal Cinder bridge (sacrifice Warriors for energy), giving it a secondary home.

**Archetype lock-in:** Crucible (primary), Cinder with Warrior splash (marginal). **1.5 archetypes.**

---

### 16. Battle Hymn (2 cost, Event, Stone+Ember)

**Ability:** Each allied Warrior gains +1 spark. Dissolve an enemy with spark 0.

**Mechanical Subtlety Score: 5/10**

**Why this score:** The two effects create a non-obvious interaction: the pump makes your Warriors bigger while the Dissolve targets spark-0 enemies. This means the card is a combat trick + conditional removal in one. The spark-0 condition is narrow but relevant -- it kills tokens, utility creatures (many Survivors and Spirit Animals are 1-spark, but some are 0-spark), and freshly materialized characters that have not been kindled yet.

The subtlety comes from the spark-0 Dissolve being relevant outside pure Warrior decks. Any deck can use "dissolve an enemy with spark 0" against specific threats. The Warrior pump is tribal-locked, but the removal half has wider applicability.

**Archetype lock-in:** Crucible (primary), with the Dissolve rider being useful in any aggressive Ember deck. **1.5-2 archetypes.**

---

### 17. Echoing Monolith (6 cost, Character, Stone+Ruin)

**Ability:** Reclaim 3. Materialized: Draw 1.

**Mechanical Subtlety Score: 5/10**

**Why this score:** The Reclaim 3 on a 6-cost body creates a genuine decision: do you hard-cast for 6 (Stone plan) or put it in the void and pay 3 (Ruin plan)? This duality is the Bedrock archetype's core tension expressed on a single card. The Materialized: Draw 1 means flicker decks (Mirage) can extract value from re-triggering it, and 4 spark at 6 cost is a reasonable finisher stat line.

The card is more subtle than most in this batch because the *method of deployment* varies by archetype. A Depths deck hard-casts it as a finisher. A Bedrock deck reanimates it for 3. A Mirage deck hard-casts it and flickers it for repeated draw. Three different play patterns from one card.

**Archetype lock-in:** Bedrock (primary), Depths (finisher), Mirage (flicker target). **3 archetypes.**

---

### 18. Entomb (2 cost, Event, Ruin)

**Ability:** Put a character from your hand into your void. Draw 2.

**Mechanical Subtlety Score: 7/10**

**Why this score:** This is deceptively flexible. On the surface, it is a Bedrock enabler (put an expensive target in the void for reanimation). But consider:

- **Eclipse:** It is a discard outlet (puts a card from hand to void) that draws 2, triggering discard payoffs. The "put into void" is mechanically a discard (hand to void), even though it targets a character specifically.
- **Cinder:** Put a creature into the void to set up recursion, draw into your sacrifice outlets.
- **Undertow:** Put a Survivor into the void to set up Dissolved-trigger chains, increase void count.
- **Tempest:** Trade a dead character for 2 cards to continue the chain (draw 2 for 2 energy is baseline acceptable even without the void setup).
- **Any deck:** "Discard a character, draw 2" is a playable rate -- you are trading a bad card for 2 new ones. Every deck has characters in hand that are worse than 2 random cards.

The card's subtlety is that it is a Bedrock signpost that secretly functions as a draw spell in half the archetypes in the game. The character-specific targeting is a genuine limitation (you cannot void events or put your last character in), but it is rarely a downside.

**Archetype lock-in:** Bedrock (primary), Eclipse, Cinder, Undertow, Tempest, and any deck that wants to trade a bad character for 2 cards. **4-5 archetypes.**

---

## Part 2: Score Summary

| Card | Score | Archetypes | Verdict |
|------|-------|------------|---------|
| Resonance Spark | 3 | 1 | On-rails Tempest card |
| Storm Conduit | 2 | 1 | Screams its archetype by designer admission |
| Volatile Inscription | 3 | 1 | Unplayable outside Tempest |
| Leyline Detonation | 4 | 1.5 | Marginal void bridge, still Tempest-locked |
| Arcane Refraction | 5 | 3-4 | Best Tempest card due to strong baseline |
| Shimmer Step | 6 | 3-4 | Clean tool, not directive |
| Veil Dancer | 4 | 1.5 | Transparent Mirage payoff |
| Tidecaller of the Drowned | 5 | 2-3 | Good variance, but obvious Undertow |
| Tidestone Arbiter | 5 | 1.5 | Clean control, no secondary uses |
| Depths of Forgetting | 6 | 3+ | Meta-game decision-making is subtle |
| Tempest Striker | 4 | 1 | Fourth copy of Musician pattern |
| Voidweave Dancer | 3 | 1 | Linear payoff, no decisions |
| Memory Unraveler | 7 | 4-5 | Genuinely multi-use engine |
| Verdant Packmother | 3 | 1 | Double tribal lock |
| Warbond Sentinel | 5 | 1.5 | Good opponent tension, still tribal |
| Battle Hymn | 5 | 1.5-2 | Narrow removal rider has wider use |
| Echoing Monolith | 5 | 3 | Deployment-method variance is real |
| Entomb | 7 | 4-5 | Secretly a universal draw spell |

**Average subtlety score: 4.6/10**

**Cards scoring 6+:** 4 out of 18 (22%)
**Cards scoring 3 or below:** 5 out of 18 (28%)
**Cards wanted by 3+ archetypes:** 5 out of 18 (28%)
**Cards wanted by only 1 archetype:** 8 out of 18 (44%)

---

## Part 3: BAD Design Patterns Identified

### Pattern 1: The Explicit Event-Count Trigger

**Cards using it:** Resonance Spark, Storm Conduit, Volatile Inscription, Arcane Refraction

**The problem:** "If you have played 2+ events this turn" is a Tempest-exclusive clause. No other archetype's baseline game plan involves playing multiple events in a single turn. This trigger literally defines the Tempest archetype, so putting it on a card is equivalent to printing "Tempest only" on the card. When 4 of 5 Tempest cards use this identical trigger template, the archetype has no mechanical variety either -- every card asks the same question ("did you play enough events?") and rewards you in the same way (more resources).

**Why it fails the multi-archetype test:** The trigger is *conditional* in non-Tempest decks and *trivial* in Tempest decks. This means it is dead text when it should be dead, and active text when it is obvious. No emergent behavior arises.

### Pattern 2: The Single-Trigger-Matters Linear Payoff

**Cards using it:** Storm Conduit ("when you play an event"), Tempest Striker ("when you play a fast card"), Voidweave Dancer ("when you discard a card"), Verdant Packmother ("when you materialize a Spirit Animal")

**The problem:** "When you do [archetype-defining action], gain [resource]" is a tribal lord template that only rewards density. There is no decision point. You are not choosing whether to trigger it -- you are simply doing what your archetype already does and getting paid for it. The card does not change your play pattern, create new options, or interact with anything outside its narrow trigger. It is a passive bonus for existing behavior.

**Why it fails:** These cards do not create deckbuilding tension or draft decisions. If you are in the archetype, you auto-include them. If you are not, they are unplayable. There is no interesting middle ground.

### Pattern 3: The Stapled-Signpost (Resonance A Effect + Resonance B Effect)

**Cards using it:** Tidestone Arbiter (Tide draw + Stone tax), Shimmer Step (Zephyr flicker + Tide draw), Veil Dancer (Zephyr materialize-trigger + Tide Foresee), Warbond Sentinel (Stone Warrior lord + Ember death-energy)

**The problem:** These cards take one mechanic from each resonance and staple them together. The result is a card that is obviously the intersection of two resonances, with no emergent third behavior. Tidestone Arbiter is "draw (Tide) + tax (Stone)" -- it does not create any interaction that is unique to *having both effects on one card*. Each half would be equally effective on separate cards.

**Why it fails:** Signpost cards should create play patterns that neither resonance produces alone. "Tide effect + Stone effect on one card" is efficient but not synergistic. The card should make you want both resonances *together* in a way that is more than additive.

### Pattern 4: The Tribal Lord with No Off-Tribe Application

**Cards using it:** Verdant Packmother (Spirit Animals only), Warbond Sentinel (Warriors only -- though the dissolved trigger has marginal wider use)

**The problem:** A card that references a specific subtype in both of its abilities has zero value outside that tribal deck. This is acceptable for tribal lords (the multi-archetype exception) but creates cards with no mechanical subtlety and no draft interest outside their narrow lane.

### Pattern 5: Overconcentration on One Archetype

**The Tempest problem:** 5 of 18 new cards (28%) are Tempest cards, and 4 of those 5 use the same "2+ events this turn" trigger. This means the new card batch is heavily biased toward one archetype's most obvious mechanical space. Tempest was thin and needed cards, but it needed *different kinds* of cards, not four variations on the same trigger. The result is that Tempest gets deeper but not more interesting.

### Pattern 6: Lack of Modal or Conditional Abilities

**The absence:** Of 18 new cards, zero have choose-one modality, zero have "you may" conditional branches that create meaningful choice, and zero have abilities that change function based on board state. Existing cards like Break the Sequence (choose: bounce or draw), Pattern Seeker (may discard to draw+point), and Duneveil Vanguard (may discard to dissolve) all create player agency. None of the new cards do.

### Pattern 7: Temporary Spark as Default Safety Valve

**Cards using it:** Tempest Striker, Voidweave Dancer, Verdant Packmother

**The problem:** "Until end of turn" is used on three cards as a safety valve against permanent scaling. While it prevents runaway, it also makes all three cards feel mechanically identical -- they are temporary pump effects triggered by different archetype actions. This creates homogeneity across archetypes: Gale's payoff works the same as Eclipse's payoff works the same as Basalt's payoff, just with different trigger conditions. The design space of "do X, get temporary spark" is one-dimensional.

---

## Part 4: COUNTER-PATTERNS -- Mechanical Templates for Multi-Archetype Synergy

### Counter-Pattern 1: The Named-Type Choice (Deployment Modality)

**Template:** "Materialized: Name a card type (Character or Event). Draw cards until you hit that type; put all other drawn cards into your void."

**Why it creates multi-archetype appeal:**
- **Tempest** names Event to dig for chain fuel, putting characters into the void as collateral.
- **Mirage** names Character to find flicker targets, putting events into the void.
- **Undertow** uses it as a self-mill engine that also draws, regardless of what is named.
- **Bedrock** might name Character to dig for a reanimation target while milling everything else.
- **Eclipse** profits from the void-filling regardless of the mode chosen.

The key is that the *choice* changes the card's function based on your archetype and board state. This is not a signpost for one archetype -- it is a tool that each archetype uses differently.

**Existing card reference:** No direct precedent exists in the pool. The closest is Seeker for the Way ("draw a Warrior from your deck"), but that is tribal-locked. This pattern generalizes the concept.

### Counter-Pattern 2: The Threshold-Gated Mode Switch

**Template:** "Judgment: If you have 3 or fewer cards in hand, draw 2. If you have 4 or more, each ally gains +1 spark until end of turn."

**Why it creates multi-archetype appeal:**
- **Tempest** depletes its hand during storm turns, then this draws refuel cards the next Judgment.
- **Gale** empties its hand through fast plays, then draws.
- **Crucible** maintains a full hand through tribal draw, getting the spark boost.
- **Basalt** uses the spark boost to push Spirit Animal Judgment scoring.
- **Cinder** depletes hand through sacrifice costs, getting the draw.
- **Eclipse** fluctuates between modes as it cycles hand contents.

The threshold creates a dynamic decision: do you play out your hand for the draw reward, or hold cards for the spark mode? This tension does not exist in any of the v1 designs.

**Existing card reference:** Pattern Seeker ("may discard to draw and gain points") has a similar opt-in structure but at a smaller scale. The Calling Night ("Judgment: Draw 1, opponent gains 2 points") is a Judgment draw engine but with a fixed cost rather than a mode switch.

### Counter-Pattern 3: The Cross-Zone Scaling Payoff

**Template:** "This character's spark is equal to the number of cards that have changed zones this turn."

**Why it creates multi-archetype appeal:**
- **Mirage** counts each flicker as 2 zone changes (out + back in).
- **Eclipse** counts each discard as a zone change (hand to void), each Reclaim as another (void to hand).
- **Cinder** counts each abandon (battlefield to void) and each recursion.
- **Tempest** counts each event played (hand to stack to void = 2 changes).
- **Undertow** counts each mill (deck to void).

This is a universal metric that every archetype's core action contributes to, but the *rate* of contribution varies. The card rewards high-activity turns regardless of *what kind* of activity. No single archetype owns "zone changes" as a mechanic.

**Existing card reference:** Tideborne Voyager ("When an ally is banished, +1 spark") counts a specific zone change. This pattern generalizes it to all zone changes, creating wider appeal.

### Counter-Pattern 4: The Conditional Commons Boost

**Template:** "Judgment: Each ally with spark 1 or less gains +1 spark until end of turn."

**Why it creates multi-archetype appeal:**
- **Crucible** has many 0-1 spark utility Warriors (Wolfbond Chieftain, Ethereal Trailblazer, Company Commander, Spirit Field Reclaimer) that this pumps.
- **Basalt** has many 1-spark Spirit Animals (Ebonwing, Dawnprowler Panther, Luminwings, Driftcaller Sovereign, Ghostlight Wolves) that this pumps.
- **Cinder** produces low-spark sacrifice fodder that this temporarily makes threatening.
- **Mirage** has low-spark utility bodies (flicker targets with 0-1 spark) that this makes relevant for Judgment.
- **Undertow** has many 1-spark Survivors that this pumps.

The key insight is that every tribal/engine archetype has many low-spark utility creatures that are not spark-relevant. This card makes them relevant, but only temporarily, creating a decision about when to deploy it for maximum Judgment value.

**Existing card reference:** Spiritbound Alpha ("pay 4: each Spirit Animal +2 spark") does this for Spirit Animals specifically. This pattern generalizes it to any archetype with cheap creatures.

### Counter-Pattern 5: The Opponent-Involving Choice

**Template:** "Choose one: Draw 2, or the opponent discards 1. If the opponent has fewer cards in hand than you, you may choose both."

**Why it creates multi-archetype appeal:**
- **Depths** uses it as a control tool -- the hand disruption is Tide-coded, the conditional "both" rewards having a full hand (which Tide maintains through draw).
- **Tempest** uses it as mid-chain draw.
- **Eclipse** might use the opponent-discard mode to trigger discard-adjacent synergies (e.g., if future cards care about opponent void size).
- **Any deck** has a meaningful decision based on the board state and opponent's hand.

The conditional "both" creates a mini-game: the card is most powerful when you are ahead on cards, which means it is strongest in Tide-aligned control decks but still useful as a draw spell in any deck.

**Existing card reference:** Wraith of Twisting Shadows ("Materialized: Discard a chosen card from opponent's hand. They draw 1") has the opponent-involving structure, but the choice is made for you. This pattern gives the player agency.

### Counter-Pattern 6: The Delayed Benefit with Optionality

**Template:** "Banish a card from your hand face down. At the start of your next turn, you may play it for free or return it to your hand and draw 1."

**Why it creates multi-archetype appeal:**
- **Tempest** exiles a big event to play for free next turn (essentially pre-loading energy).
- **Bedrock** exiles an expensive character, then plays it for free (a form of cheating on cost).
- **Depths** exiles a finisher, then deploys it safely next turn.
- **Eclipse** exiles something, decides next turn whether to play it or return it (the return + draw mode triggers discard-irrelevant but provides card selection).
- **Gale** exiles a fast threat, keeping the option to deploy it for free next turn.

The delayed benefit creates risk (you lose the card for a turn) and optionality (you choose the mode on resolution). The card rewards planning ahead, which is distinct from every v1 design's focus on immediate value.

**Existing card reference:** Passage Through Oblivion ("Banish an ally. Materialize it at end of turn.") has the delayed re-entry structure, but only for allies already in play. This pattern applies it to cards in hand.

### Counter-Pattern 7: The Scaling Cost with Variable Payoff

**Template:** "Pay X energy: Put the top X cards of your deck into your void. For each character put into the void this way, you may materialize a 1-cost figment. For each event, draw 1."

**Why it creates multi-archetype appeal:**
- **Undertow** pays high X to mass-mill, getting incidental figments and draw.
- **Basalt** pays moderate X hoping for characters to get figments that trigger materialize-matters.
- **Mirage** wants the figments for materialize-matters payoffs.
- **Tempest** wants the event draws to refuel.
- **Bedrock** wants specific expensive characters to land in the void.
- **Eclipse** profits from the void-filling.

The X-cost creates a decision: how much energy to invest. The split payoff (figments for characters, draw for events) means the output depends on your deck composition, creating emergent variance. Different decks build differently to optimize this card.

**Existing card reference:** Burst of Obliteration ("Pay 1 or more energy: Dissolve all characters with spark less than amount paid") has the X-cost template. Secrets of the Deep ("Pay 1+ energy: draw 1 per energy, discard 2") also uses variable cost. This pattern applies variable cost to a multi-output effect.

### Counter-Pattern 8: The Symmetric Effect with Asymmetric Reward

**Template:** "Each player puts the top 3 cards of their deck into their void. You gain 1 energy for each character put into either player's void this way."

**Why it creates multi-archetype appeal:**
- **Undertow** builds to profit from self-mill and does not care about opponent mill.
- **Bedrock** might hit an expensive reanimation target.
- **Depths** uses it as a disruption tool against opponent strategies that need their deck intact.
- **Cinder** gains energy to fuel sacrifice turns.
- Any deck gains 0-6 energy depending on variance, making it a flexible energy card with mill upside.

The symmetry (both players mill) creates interaction and counterplay. The asymmetric reward (you gain energy, they do not) ensures the card has a clear owner. The variance in energy output (depends on what is milled) creates interesting risk assessment.

**Existing card reference:** Nightmare Manifest ("Each player abandons a character") and Wasteland Arbitrator ("Each player discards 1") both use symmetric effects. This pattern adds an asymmetric payoff layer.

### Counter-Pattern 9: The Conditional Recursion with Archetype-Dependent Trigger

**Template:** "When a card enters your void from any zone, you may pay 1 energy. If you do, return a different card from your void to your hand."

**Why it creates multi-archetype appeal:**
- **Eclipse** triggers it on every discard, using it as a continuous loop engine.
- **Cinder** triggers it on every sacrifice, retrieving sacrifice fodder.
- **Undertow** triggers it on every mill, cherry-picking the best milled card.
- **Tempest** triggers it when events resolve to void, retrieving key events.
- **Mirage** triggers it less frequently but can use it when creatures are bounced through the void.

The "any zone" clause means every archetype's primary action triggers it. The 1-energy cost prevents infinite loops and creates resource tension. The "different card" requirement prevents self-loops. The result is a universal recursion engine where the rate of use depends on how frequently your archetype puts cards in the void.

**Existing card reference:** Memory Unraveler ("Once per turn, discard: return from void") is the closest existing design and is already the highest-subtlety new card. This pattern removes the once-per-turn restriction but adds an energy cost, making it more powerful but more energy-hungry.

### Counter-Pattern 10: The Board-State-Reading Dual Mode

**Template:** "Choose one -- Dissolve an ally: Draw 3. OR Dissolve an enemy: Gain 3 energy."

**Why it creates multi-archetype appeal:**
- **Cinder** uses mode 1 to sacrifice a creature for massive draw.
- **Gale/Ember** uses mode 2 as removal + energy burst.
- **Tempest** agonizes over mode 1 (draw 3 feeds the chain) vs. mode 2 (energy + removal).
- **Depths** prefers mode 2 for control.
- **Mirage** prefers mode 1 to sacrifice a creature it has already extracted value from.

Modal cards with genuinely different modes (one sacrificial, one aggressive) create deckbuilding and in-game decisions. The card is not "always mode A in archetype X" -- the correct mode depends on the board state, making it a skill-testing card.

**Existing card reference:** Break the Sequence ("2 energy: bounce enemy OR 3 energy: draw 2") is the closest modal card in the pool. This pattern pushes modality further by making the modes mechanically distinct rather than being different costs for similar effects.

---

## Part 5: Summary of Recommendations

### The Core Problem

The v1 new cards are designed as **archetype labels**, not as **mechanical tools**. They tell the drafter "you are in archetype X" but do not create interesting decisions within or across archetypes. The average card is wanted by 1.8 archetypes, well below the 2+ target. The average subtlety score of 4.6/10 reflects a batch of cards that are functional but uninspiring.

### What Makes Memory Unraveler and Entomb Work

These two cards score highest because they are **tools with flexible applications**, not **labels for specific archetypes**. Memory Unraveler's discard-to-retrieve is universally useful; the *context* of use varies by archetype. Entomb's "put a character in the void, draw 2" is a Bedrock enabler that secretly functions as a draw spell anywhere. Both cards have strong primary homes but genuine secondary demand.

### Design Principles for v2 Replacements

1. **Baseline playability:** Every card should be at least marginally playable in a deck that is not its primary archetype. "Draw 1, opponent gains 1 point" is not baseline playable.
2. **Emergent interactions:** The card's abilities should combine with *existing* cards in ways that are not immediately obvious from the card text alone. The v1 batch has zero emergent interactions.
3. **Modality or conditionality:** At least half the new cards should have choose-one modes, threshold-gated effects, or "you may" clauses that create real decisions.
4. **Cross-archetype triggers:** Prefer triggers that multiple archetypes satisfy (zone changes, card-entering-void, materializations) over triggers that only one archetype satisfies (playing events, playing fast cards, discarding).
5. **Avoid restating the archetype's definition as a trigger:** "When you play an event" in Tempest, "when you discard" in Eclipse, and "when you materialize a Spirit Animal" in Basalt are all equivalent to printing the archetype name on the card. Use triggers that the archetype satisfies *as a consequence of its game plan* but that other archetypes can also satisfy in different ways.

### The Tempest Overallocation

Five cards for one archetype using the same trigger template is a failure of variety, not a depth success. Tempest needs:
- 1 card with the "events this turn" trigger (it already has existing enablers like Starcatcher, Keeper of the Lightpath, Echoes of the Journey)
- 2-3 cards that Tempest wants but that other archetypes also want, creating genuine draft tension
- Cards that solve Tempest's actual problem (fizzling mid-chain) through different mechanical approaches, not four variations on "if you played 2+ events"

### Cards Worth Keeping (or closely adapting)

- **Memory Unraveler** (score 7) -- keep as-is or nearly so
- **Entomb** (score 7) -- keep as-is or nearly so
- **Shimmer Step** (score 6) -- keep, clean design
- **Depths of Forgetting** (score 6) -- keep, interesting opponent-compensation design
- **Echoing Monolith** (score 5) -- keep, Bedrock needs this specific card

### Cards That Need Full Redesign

- **All four "2+ events this turn" Tempest cards** -- replace with more flexible designs
- **Storm Conduit** -- replace with a Tempest engine that has cross-archetype appeal
- **Voidweave Dancer** -- replace with an Eclipse payoff that has threshold or modal design
- **Tempest Striker** -- replace with a Gale signpost that does not clone the Musician template
- **Verdant Packmother** -- replace with a Basalt signpost that bridges to non-Spirit-Animal decks
- **Veil Dancer** -- replace with a Mirage card that creates decisions, not passive bonuses
