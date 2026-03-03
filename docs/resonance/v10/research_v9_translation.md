# Research Results: V9 Mechanism Translation to AI Drafters

## Question

How do V9's proven mechanisms (pool contraction, pair-affinity, floor slots) map
onto AI drafter behavior? What are the mathematical constraints on AI drafter
designs that must respect V9-level M3 and M11?

---

## 1. Pool Contraction as AI Picks: The Equivalence

V9 Hybrid B removes 12% of the pool per pick starting at pick 4. With 360
cards and a pool minimum of 17, this contracts the pool to roughly:

| Pick | Pool size |
|:----:|:---------:|
| 4    | 317       |
| 8    | 204       |
| 12   | 131       |
| 18   | 72        |
| 24   | 40        |
| 28   | 23        |

Each pick removes 0.12 * current_pool cards by culling the lowest-relevance
cards. An AI drafter produces the same pool reduction by taking one card per
pick from a pool of the same size.

**The key mapping:** If one AI drafter takes 1 card per pick, and the player
takes 1 card per pick, the pool shrinks by 2 cards per round. That is a
percentage removal of 2 / pool_size, which starts at 0.56% and rises slowly
as the pool shrinks. This is far weaker than V9's 12% contraction.

To match V9's 12% contraction rate at pick 4 (pool ~317), an AI-drafter system
needs to remove approximately 38 cards per pick (0.12 * 317). With 4-card
packs, this means the equivalent of 9-10 AI drafters each taking 1 card per
pick, or fewer AIs taking multiple cards per pick.

**Concrete equivalence table:**

| AI config | Cards removed per round | Equivalent contraction at pick 4 (pool=317) |
|-----------|:-----------------------:|:-------------------------------------------:|
| 1 AI, 1 card/pick | 1 | 0.3% |
| 3 AIs, 1 card/pick | 3 | 0.9% |
| 7 AIs, 1 card/pick | 7 | 2.2% |
| 9 AIs, 4 cards/pick (full pack) | 36 | 11.4% |
| 10 AIs, 4 cards/pick | 40 | 12.6% |

**Conclusion:** To match V9's 12% contraction rate, the AI drafter system needs
approximately 9-10 AIs each drafting a full 4-card pack per round. This is more
AIs than the number of archetypes (8), which means some archetypes would need
multiple AIs or each AI would need to take more than 1 card per pick.

The more natural design — 3-5 AIs each taking 1-2 cards per pick — produces
contraction rates of 1-3%, far below V9's threshold. **This is the central
mathematical constraint V10 must solve.** AI drafters as normally conceived
(each taking 1 card per pick from their archetype) cannot replicate V9's
contraction intensity unless either (a) many AIs are present, or (b) the
contraction is supplemented by a second mechanism (explicit relevance culling
applied on top of AI picks).

**Important nuance:** V9's contraction is weighted. The algorithm removes the
bottom 12% by relevance score, not 12% at random. This is qualitatively
different from AI picks, which remove cards that are attractive to a particular
archetype. V9 culls cards that are irrelevant to the player's archetype; AI
drafters cull cards that are relevant to the AI's archetype. The two mechanisms
pull in different directions — V9 removes low-relevance cards from the player's
perspective; AI drafters remove high-relevance cards from the AI's own
perspective. For V10, the net effect on the player's pool is that cards
attractive to other archetypes disappear, which is the correct framing
narratively, but the per-archetype targeting of what survives is different from
V9's relevance-weighted culling.

---

## 2. Pair-Affinity in AI Drafters: Same Distinction, Different Mechanism

V9's pair-affinity scores let the contraction distinguish Warriors from Sacrifice
within Tide-primary cards. A Tide card with (warriors_affinity=0.85,
sacrifice_affinity=0.15) survives the Warriors player's contraction; a card
with (warriors_affinity=0.15, sacrifice_affinity=0.85) is culled earlier.

**The AI drafter analog:** An AI drafter assigned to Warriors would prefer the
same Warriors-affinity cards over Sacrifice-affinity cards. It would take the
(0.85, 0.15) Warriors card before the (0.15, 0.85) Sacrifice card. This is
conceptually natural — the Warriors AI drafts Warriors cards.

**Does the AI need hidden affinity data?**

Three possible preference models:

**Model A — Visible resonance only:** AI takes cards matching its primary and
secondary visible resonance symbols. A Warriors AI (Tide/Zephyr) takes any
(Tide) or (Zephyr) card. This replicates V8-level contraction — adequate for
M3 but insufficient for M11. The AI cannot distinguish Warriors from Sacrifice
within Tide-primary cards because it lacks pair-affinity data. This is
mathematically equivalent to a visible-only contraction, which V9 showed caps
M11 at approximately 2.1 at 10% visible dual-resonance.

**Model B — Archetype tag (3-bit equivalent):** AI knows which cards are tagged
for its archetype and prioritizes them. This is equivalent to V9's 3-bit
archetype tag approach, which achieved M3 = 2.37-2.62 but M11 only 2.40-2.83.
The AI doesn't need continuous affinity floats — a binary home/sibling
classification is sufficient. This is the simplest AI model that enables
within-sibling discrimination.

**Model C — Pair-affinity scores (8-bit equivalent, Hybrid B class):** AI uses
the same 4-bit pair-affinity floats as V9's contraction algorithm. The Warriors
AI selects cards by descending warriors_affinity score. This replicates Hybrid
B's discriminative power. The player cannot directly see which cards the AI is
prioritizing, but the AI's behavior is derivable from honest card mechanics
(V3 = 9/10).

**Key finding:** Model B (archetype tag) is sufficient for M3 >= 2.0 but not
M11 >= 3.0. Model C (pair affinity) is needed for M11 >= 3.0. This mirrors V9's
exact finding. The AI drafter version of this constraint is: **an AI that
drafts by archetype tag produces M11 ~ 2.8; an AI that drafts by pair-affinity
scores produces M11 ~ 3.2.**

The AI drafter framing does not escape the fundamental information requirement.
To achieve M11 >= 3.0, the AI must use information that distinguishes
same-resonance siblings — either archetype tags or pair-affinity scores. Whether
that information is called "hidden metadata" or "AI archetype knowledge" is
framing, not mathematics.

**Bridge cards:** V9's pair-affinity encoding keeps bridge cards (high affinity
for both archetypes) in the pool longer because they are not strongly culled by
either archetype's contraction pass. In the AI drafter translation, bridge cards
are cards that multiple AIs want to some degree but that no AI prioritizes
above home-affinity cards. This produces similar bridge-card persistence
naturally — bridge cards are passed over by the Warriors AI in favor of stronger
Warriors cards, and similarly by the Sacrifice AI. They survive in the pool
longer as a result.

---

## 3. Floor Slot: AI Drafter Equivalents

V9's floor slot guarantees 1 top-quartile card from the player's committed
archetype in each pack starting at pick 3. This directly addresses M10
(consecutive bad packs) by ensuring that no pack after pick 3 contains zero
high-quality options for the committed player.

**The floor slot's role in V9's failures:** Hybrid B still fails M10 (3.8
consecutive bad packs) and M5 (convergence at pick 9.6). The floor slot
mitigates but does not solve M10. Design 4's M10 = 2.13 (marginally better)
suggests the floor slot alone cannot fix M10 — it reduces the worst streaks
but does not eliminate them because the transition zone (picks 6-10) has
inherently noisy archetype inference.

**AI drafter equivalents:**

**Option A — No floor slot:** Simply not modeled. AI drafters take cards they
want; the player sees what remains. This produces no quality guarantee. If an
aggressive Warriors AI takes all top Warriors cards early, the player who
drafts Warriors gets mediocre packs until the AI moves on to other cards. This
would likely worsen M10 relative to V9.

**Option B — Passive AI (floor slot proxy):** A "weaker" AI that only takes
cards above a power threshold leaves lower-quality cards for the player. The
floor slot effect is reversed: a passive AI takes good generic cards but leaves
the best archetype-specific cards. This is the opposite of what a floor slot
does — the floor slot ensures good archetype cards appear, but a passive AI
would ensure they remain by not taking them. A passive Warriors AI that avoids
the top-quartile Warriors cards would organically leave strong Warriors cards
available for the player.

This is mathematically equivalent to the floor slot if the passive AI's
"avoidance threshold" matches the top-quartile boundary, but the player
experience framing is different: the AI is depicted as somewhat inept rather
than the game guaranteeing quality.

**Option C — Coach mechanic:** A guaranteed high-quality archetype card appears
in each pack independently of AI behavior. This is the floor slot re-labeled.
It cannot be framed as "AI opponents drafting" without awkwardness — it would
require explaining why the AI didn't take this card. One natural framing: the
coach card was already reserved for the player before the AIs drafted.

**Option D — Reactivity-based floor slot:** A Level 3 (lane-avoidant) AI that
actively avoids the player's lane once the player commits naturally leaves
high-quality archetype cards available. If the Warriors AI pivots away from
Warriors cards when it detects the player is drafting Warriors, the net effect
is that top-quality Warriors cards remain in the pool. This is the most
narratively coherent floor slot equivalent: "the Warriors AI noticed you were
competing for Warriors cards and pivoted to a different strategy."

**Recommendation for V10:** Option D (lane-avoidant AI as floor slot) is the
most natural translation. It solves M10 not by guaranteeing a specific card
appears but by ensuring the competing AI doesn't strip the pool of the player's
archetype. This requires at least Level 2-3 reactivity for the relevant AI.
The trade-off: this removes signal reading value (the player always gets their
archetype's cards once committed), potentially making the draft feel too easy.

---

## 4. V9's M5 and M10 Failures: Can AI Drafters Help?

**M5 failure (convergence at pick 9.6, target 5-8):** V9 converges late because
archetype inference from early picks is unreliable at 10% visible dual-resonance.
The algorithm doesn't confidently know which sibling the player wants until pick
5-9, delaying the contraction's precision.

AI drafters could potentially help M5 through observable lane signals. If the
player can see that the Warriors AI is drafting heavily from Tide cards and the
Sacrifice AI is also taking Tide cards, the player receives earlier signal that
Tide-primary archetypes are contested — not specifically Warriors vs. Sacrifice,
but at least the resonance layer. If the player sees many (Tide) cards being
taken by AIs, they can infer which resonance is "crowded" and which is "open"
as early as picks 1-3.

However, for V9's specific M5 failure — distinguishing Warriors from Sacrifice
within Tide — visible AI behavior (both AIs take Tide cards) does not resolve
the within-resonance ambiguity. AI drafters signal at the resonance level; the
sibling level requires pair-affinity discrimination. M5 would improve only if
the player can observe which specific cards the AIs are taking, not just which
resonance they're in.

**AI drafters create earlier convergence signals if the player can see AI picks
or the resulting thinning of specific card types.** The orchestration plan notes
the player observing "which archetypes have more cards available" as a skill
axis. This visible lane signal is V10's mechanism for improving M5. The
mathematical relationship: if the player commits earlier because lane signals
are stronger, the contraction (or AI-driven pool reduction) starts earlier,
improving M5. This is a genuine structural improvement over V9's abstract
contraction, which gave no visible signal for why the player should commit.

**M10 failure (3.8 consecutive bad packs, target <= 2):** V9's M10 failure is
structural: the transition zone (picks 6-10) is where archetype inference
stabilizes, and before inference stabilizes, the contraction removes the wrong
cards. AI drafters produce a different but related problem: if the player's
archetype AI is very aggressive early, the player's archetype cards thin out
early. If the player's archetype AI is slow (or absent), quality is adequate.
The M10 outcome depends heavily on AI timing.

AI drafters could improve M10 through lane-avoidant reactivity: once the player
signals commitment (picks 5-6), the competing AIs pivot away from the player's
archetype, guaranteeing that subsequent packs have adequate archetype card
supply. This is mechanically analogous to a delayed floor slot that activates
on commitment detection.

The critical constraint: lane avoidance can be too effective (always guarantees
good packs, removes the challenge of M10 streaks entirely) or too slow (AIs
don't react quickly enough). V9 showed the transition zone persists across all
6 simulated algorithms — it is structural, not parameter-sensitive. AI drafter
reactivity is V10's main hypothesis for escaping this structure.

---

## 5. Mathematical Constraints for V10 AI Drafters

To maintain V9-level M3 >= 2.0 and M11 >= 3.0, V10 AI drafter designs must
respect:

**Constraint 1: Total pool reduction must reach V9 contraction levels by mid-draft.**

V9 contracts the pool to approximately 131 cards by pick 12 (from 360). AI
drafters must collectively remove enough cards to achieve comparable pool
concentration by the same pick. With 9-10 AIs each taking one card per pick,
the pool shrinks by 9-10 cards per round (the player takes 1 more). After 12
picks, this removes 120-132 cards, matching V9's trajectory.

If AI count is lower (3-5 AIs), supplemental visible-based culling or higher
per-AI pick rates are required to maintain equivalent concentration.

**Constraint 2: AI picks must be archetype-discriminating, not random.**

Random AI picks (each AI takes any card at random) would produce pool reduction
but no archetype targeting. The player would see a random 20% reduction in the
pool, not a reduction of the cards that compete with their archetype. For M3
and M11 to benefit, AI picks must remove cards from specific archetypes — the
same targeting that V9's relevance-weighted contraction achieves. AIs using
visible resonance only achieve V8-level discrimination; AIs using pair-affinity
data achieve Hybrid B-level discrimination.

**Constraint 3: Floor quality cannot drop below 1 top-quartile card per pack from pick 6.**

V9's floor slot provides this guarantee. Without it, M10 rises. The AI drafter
equivalent must either include a passive AI (avoidance-based floor slot), a
lane-avoidant reaction, or an explicit floor slot mechanism. Removing the floor
slot entirely would likely push M10 above 5, based on Design 4's result (M10 =
2.13 with floor slot) vs. the no-floor-slot projection.

**Constraint 4: Same-resonance sibling discrimination requires 8 bits equivalent.**

V9 proved a 3-bit archetype tag is insufficient for M11 >= 3.0. AIs using only
binary archetype knowledge (home vs. sibling) replicate the 3-bit regime and
cap M11 at approximately 2.8. AIs that use continuous pair-affinity scores
replicate the 8-bit regime and can reach M11 >= 3.0. This constraint cannot be
escaped through architectural choices — it is a consequence of the pool
composition (10% visible dual-resonance) and fitness model (Graduated Realistic).

**Constraint 5: Archetype inference must stabilize by pick 5-7.**

V9's M5 failure (convergence at 9.6) occurs because archetype inference is
noisy for the first 5-9 picks. AI drafters can help by providing visible lane
signals earlier (the player observes AI behavior, infers which lanes are open,
and commits faster). But the algorithm must still infer which sibling the player
wants to target correctly. If AI drafters are used to simplify inference
(e.g., the player is definitionally "competing" with whichever AI's archetype
they've taken the most cards from), this could stabilize inference earlier.

---

## Connections

**Relevant to all V10 design agents:** The contraction equivalence calculation
(Section 1) is the central constraint. Any design that uses fewer than ~9 AIs
with 1 card/pick needs supplemental culling to match V9's pool reduction.

**Relevant to Agent 1 (Static AIs):** A fully predetermined AI system with 7-8
AIs each drafting 4 cards per pack could approach V9's contraction intensity,
but the directionality of picks (AIs remove high-affinity cards, not low-affinity
cards) means the player's pool is depleted of other-archetype cards rather than
enriched with own-archetype cards. This is the same net effect but via a
different mechanism.

**Relevant to Agent 3 (Lane-Avoidant):** Lane avoidance is the most natural
floor slot equivalent. Section 3 Option D details how this works and its
trade-offs.

**Relevant to Agent 4 (Aggression Spectrum):** AI aggression directly controls
effective contraction rate. A fully aggressive AI removes ~12 cards per pick
from its archetype pool; a passive AI removes far fewer. The aggression level
is a tuning knob for matching V9's contraction intensity.

**Relevant to all agents:** The mathematical ceiling from V9's research still
applies. At 10% visible dual-resonance under Graduated Realistic fitness, M11
>= 3.0 requires same-resonance sibling discrimination equivalent to 8 bits per
card. No AI drafter architecture escapes this; the question is how that
information is encoded in the AI's preference function.

---

## Open Questions

1. **Can the AI pick direction substitute for V9's contraction direction?** V9
   removes low-relevance cards from the player's perspective; AI drafters remove
   high-relevance cards from the AI's perspective. Are these mathematically
   equivalent for M3/M11, or does the directionality matter?

2. **How many AIs is the right count given pack size 4?** The calculation shows
   9-10 AIs for V9-equivalent contraction, but 9-10 AIs means 36-40 AI picks
   per round vs. 4 player picks. What does this look like as a game and does
   it feel like a real draft table?

3. **Does lane-avoidant reactivity replace the floor slot entirely, or does it
   need to be combined with an explicit floor mechanism?** V9's M10 = 3.8 with
   the floor slot suggests the floor slot alone is insufficient; lane avoidance
   may be the more powerful mechanism but needs simulation to confirm.

4. **Can player-visible AI behavior provide the earlier convergence signal that
   would fix M5?** If the player can observe which cards are being taken by AIs,
   do they commit earlier (improving M5), and if so, by how much?

5. **Is supplemental relevance culling acceptable under the "AI opponents"
   framing?** If AI picks alone cannot match V9's contraction intensity, the
   system may need to combine AI picks with explicit low-relevance culling.
   Can this additional culling be framed as AI behavior, or does it break the
   narrative?
