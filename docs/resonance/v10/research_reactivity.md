# Research Results: AI Drafter Reactivity and Fairness

## Question

How should AI drafters respond to the player's actions, and at what point does
reactivity feel unfair? Specifically: does full independence feel static, does
full reactivity feel rigged, and is there a sweet spot that produces the
"drafting against real opponents" feeling?

---

## Core Finding: The Reactivity Paradox

The fundamental tension is this: the mechanisms that make reactivity feel
*authentic* are the same mechanisms that make it feel *unfair*.

In a real booster draft, other players are independent — they cannot see your
picks and do not react to them. Signal reading works precisely because other
drafters are *not* reacting to you; they are drafting their own preferences, and
the cards they pass reflect those preferences. When you see strong Warriors cards
flowing late, it means the players on your right aren't in Warriors — not that
Warriors has been opened up for you.

If AI drafters are reactive (Level 3-4), they are doing something no real
drafter does: adjusting their strategy based on hidden information about the
player. This violates the mental model V10 is trying to create. However, fully
static AIs (Level 0) may produce a draft that feels like solving a fixed puzzle
rather than a competitive table.

The resolution: **the sweet spot is not in the middle of the reactivity
spectrum, but rather in making Level 0 (static) behavior feel sufficiently
varied and competitive that it doesn't feel predetermined.** The puzzle vs.
competition distinction comes from variety and unpredictability, not from
reactivity.

---

## Findings

### 1. Real Draft Signal Reading Requires Independent Drafters

MTG's booster draft signal reading is built entirely on drafter independence.
Signals arise from what other players *don't* pick — a strong card passing late
means everyone to your right declined it. This inference is only meaningful if
those players were acting on their own preferences, not reacting to you.

As Reid Duke explains in the official MTG signal reading guide: signals come
from "what cards you pass, receive, do not pass, and do not receive from your
neighbors." Each signal is a data point about an independent agent's
preferences. The accumulated picture reveals the table's archetype distribution.

**Implication for V10:** If AIs are reactive (e.g., Level 3 — they back off
Warriors when the player commits to Warriors), then Warriors cards flowing late
no longer signals "nobody else is in Warriors." It signals "the AI moved away
from Warriors because of me." These are completely different inferences. A
reactive AI corrupts the signal reading skill axis — the player cannot trust
what late cards mean because they don't know how much AI behavior has been
influenced by their own picks.

Signal reading requires at minimum Level 0 or Level 1 AIs. Levels 3-4 destroy
the inferential foundation of signal reading entirely.

Sources:
- [Signals in Booster Draft | MTG](https://magic.wizards.com/en/news/feature/signals-booster-draft-2015-01-19)
- [Drafting 101: Understanding Signals](https://magic.wizards.com/en/news/feature/drafting-101-understanding-signals-2016-04-12)

---

### 2. MTG Arena Quick Draft: Static Bots Do Send Signals

MTG Arena's Quick Draft mode uses bots with fixed personalities and largely
predetermined pick orders — the closest real-world analog to Level 0 AIs.
Research by 17Lands found that these static bots do send meaningful signals:
players who identify the open colors perform better.

Key findings from 17Lands bot signal analysis:
- Bots have strict pick orders within each color; they don't adjust mid-draft
- 18 rares/mythics were never passed by bots across hundreds of recorded drafts
- Signal reading value with bots is comparable to (or slightly stronger than)
  human drafts — precisely because bot behavior is consistent and learnable
- Bot "personalities" (archetype preferences) are consistent throughout the
  draft, not reactive

The criticism of Arena bots is not that they are static — it is that they are
*too* static and learnable. Experienced players memorize bot tendencies and
exploit them. The solution is not reactivity; it is varied AI personalities and
occasional "imperfect" picks.

Sources:
- [Do the Bots Send Signals? | 17Lands](https://blog.17lands.com/posts/bot-signals/)
- [Magic Arena Bot Drafting | LessWrong](https://www.lesswrong.com/posts/mLzA746mERFsZEq8j/magic-arena-bot-drafting)

---

### 3. The Rubber-Banding Problem: When Reactivity Feels Unfair

Game design research on dynamic difficulty adjustment (DDA) and rubber-banding
reveals why reactive opponents feel unfair:

**Core failure mode:** When the game adjusts to your actions, the player loses
confidence that outcomes reflect their own skill. This is the DDA paradox:
systems that help struggling players are resented by skilled players, and
systems that punish leading players feel like cheating.

The rubber-banding critique from Game Wisdom:
> "Rubber-banding is a design option that, while popular, is also a big example
> of poor game design. Rubber-banding is often used as a fix-all for games with
> poor AI."

The research consensus:
- Obvious reactivity feels like cheating (Mario Kart blue shells, Civilization
  difficulty scaling via resource boosts)
- Hidden reactivity, when discovered, creates deeper betrayal ("the game was
  lying to me")
- The fairness test: "no outside force affected how things turned out" — players
  want to believe outcomes are earned

**For draft specifically:** If the player commits to Warriors and AIs suddenly
stop drafting Warriors (Level 3 — lane avoidant), the player may feel rewarded
in the short term but learn over many runs that commitment always opens the lane.
This removes the puzzle: the player isn't reading the table, they are
*triggering* the lane to open. The skill disappears.

Sources:
- [Explaining Rubber-Banding AI in Game Design | Game Wisdom](https://game-wisdom.com/critical/rubber-banding-ai-game-design)
- [More Than Meets the Eye: Secrets of DDA | Gamedeveloper](https://www.gamedeveloper.com/design/more-than-meets-the-eye-the-secrets-of-dynamic-difficulty-adjustment)
- [Dynamic game difficulty balancing | Wikipedia](https://en.wikipedia.org/wiki/Dynamic_game_difficulty_balancing)

---

### 4. When Static AIs Feel Like a Fixed Puzzle (Level 0 Failure Mode)

The risk with fully predetermined AIs (Level 0) is not that signal reading
breaks — it is that the draft feels solvable rather than competitive. A fixed
puzzle has a right answer the player can memorize. Each game with the same seed
would be identical; across games, pattern recognition replaces genuine reading.

Research on AI opponent design (board game AI specifically) identifies this
failure mode: bots that are too predictable teach players to exploit tendencies
rather than develop transferable skills. The problem is not that bots are
predetermined but that their behavior is *learnable to the point of trivial
exploitation*.

The distinction between "solvable puzzle" and "competitive table" comes from:
1. **Variety of AI composition per game** — which AIs are at the table varies,
   so there is no single "right answer" to learn
2. **AI behavior contains some noise** — occasional off-archetype picks prevent
   perfect prediction
3. **Multiple viable lanes** — predetermined AIs should never leave only one
   acceptable path

If Level 0 achieves these three properties, it will not feel static. The
"puzzle" becomes: "what is the table composition this game?" — a question the
player must answer through signal reading rather than memorization.

Sources:
- [Magic Arena Bot Drafting | LessWrong](https://www.lesswrong.com/posts/mLzA746mERFsZEq8j/magic-arena-bot-drafting)
- [Do the Bots Send Signals? | 17Lands](https://blog.17lands.com/posts/bot-signals/)

---

### 5. The Transparency Principle

Research on DDA and AI fairness converges on a key principle: hidden reactivity
that players can *detect* creates deeper betrayal than visible reactivity.

A player who discovers that AIs are backing off their archetype will feel
manipulated, even if the experience was enjoyable before discovery. The fairness
narrative collapses: "the game was secretly helping me" is more damaging than
"I made good picks."

V10's "AI opponents" framing provides a natural transparency: players expect
other drafters to follow their own preferences, not to be reading your picks.
This means Level 3-4 reactivity (lane avoidant, fully dynamic) specifically
*violates* the framing that makes V10 defensible. If other players were
somehow watching your picks and adjusting accordingly, that would feel like
cheating at a real draft table — and players will recognize this dynamic when
they discover it.

The transparency design principle, applied to V10: **whatever the AIs do should
be explainable as "they were drafting their own deck."** Lane avoidance fails
this test. Static preferences, varied per-game, pass this test.

Sources:
- [Dynamic Difficulty Adjustment in Games | IntechOpen](https://www.intechopen.com/chapters/1228576)
- [The Corrosive Comfort: Why DDA is Ruining Games | Wayline](https://www.wayline.io/blog/dynamic-difficulty-adjustment-ruining-games)
- [Game design "illusion of challenge" | Wayline](https://www.wayline.io/blog/illusion-of-challenge-fake-difficulty-game-design)

---

### 6. The Automa Precedent: Independent Simulation Works

Board game automa systems (solo mode AI opponents for games like Terraforming
Mars) demonstrate that independent, non-reactive AI simulation can create
genuinely competitive experiences. The Terraforming Mars Automa is explicitly
designed to compete for the same resources as the player — without knowing what
the player will do.

The Automa philosophy: "each action performed by the Automa is the same as what
a human would perform." The bot follows its own logic, does not react to the
player's specific choices, and creates competition through simultaneous
resource-claiming. Players experience this as genuine competition.

The key design principle from automa design: **simulate the pressure of
competition, not the knowledge of the opponent.** An automa that takes resources
the player wants creates competitive tension without information asymmetry.

For V10: AIs that draft cards in their archetype create competitive pressure
without needing to watch what the player is doing. The player feels competition
from scarcity, not from AIs responding to them.

Sources:
- [Terraforming Mars: Automa — How Does It Enhance Solo Mode?](https://meepleit.substack.com/p/terraforming-mars-automa-a-solo-mode)
- [Playing solo — thoughts on automas | Zatu Games](https://zatu.com/en-us/blogs/features/playing-solo-thoughts-on-automas)

---

### 7. Player Agency: The Illusion vs. Real Choice Problem

Research on player agency in games identifies a specific failure mode relevant
to V10: players resent false choices — situations where their picks appear to
matter but outcomes are actually predetermined.

The Wayline analysis: "My biggest annoyance is giving me choices that very
obviously don't matter and you are just giving me a choice to give me a choice."

This failure mode applies to Level 3-4 AIs in an unexpected way: if AIs always
open the player's committed lane (lane avoidant), then early picks feel
consequential but actually aren't — the outcome (the lane opens) is
predetermined by the algorithm, not by the quality of the player's reading.

Conversely, Level 0 AIs with varied composition create *real* agency: the
player's early pick signal-reading matters because AIs do not adjust. Reading
wrong costs the player cards; reading right rewards them. This is authentic
agency.

The implication: **real agency requires that the outcome of the player's choices
depends on external conditions they don't control.** Static AIs provide this
because they have their own fixed preferences. Reactive AIs remove this because
the environment adjusts to the player's choices.

Sources:
- [The Illusion of Choice: How Modern Games Stifle Player Agency | Wayline](https://www.wayline.io/blog/illusion-of-choice-modern-games-stifle-player-agency)
- [Player Agency: How Game Design Affects Narrative | Gamedeveloper](https://www.gamedeveloper.com/business/player-agency-how-game-design-affects-narrative)

---

## The Sweet Spot: Recommendation

Based on the research, the sweet spot is not a specific level on the reactivity
spectrum but a combination of properties:

**Primary recommendation: Level 0 with high per-game variety.**

Static AIs feel like real players when:
- Which AIs are present varies game-to-game (the "table composition" changes)
- AIs have distinct personalities with non-trivial pick behavior (not just
  "take the best card for your archetype")
- AIs occasionally make imperfect picks (preventing perfect prediction)
- Multiple lanes are genuinely available in most games (not all lanes contested)

**Secondary: Level 1 (delayed reaction) as the maximum acceptable reactivity.**

If AIs establish their lanes in picks 1-5 (predetermined) and then increase
pick urgency based on overall pool pressure (not player-specific picks) from
pick 6 onward, this would be explainable as "AIs adjusting as the pool thins
out" rather than "AIs watching what you're doing." This preserves the signal
reading skill axis while allowing some mid-game dynamism.

**Levels 3-4 are incompatible with the V10 framing** and should not be tested.
Lane avoidance specifically violates the "other players at the table"
mental model and corrupts signal reading.

---

## Connections

- **Research Agent A (AI Drafting in Games)** may have found concrete data on
  how Arena's static bots compare in player satisfaction vs. Premier Draft (human
  opponents). These findings would complement the reactivity analysis.
- **Research Agent C (V9 Translation)** should note that V9's pool contraction
  was effectively a Level 4 system (fully dynamic, invisible). V10's Level 0
  framing is a significant philosophical shift, not just a narrative change.
- **Algorithm Design Agent 3 (Lane-Avoidant AIs)** should be aware that Level 3
  reactivity directly conflicts with the signal reading goal and the fairness
  narrative. The research suggests testing it anyway (to confirm the problem), but
  the design case against it is strong.
- **Algorithm Design Agent 2 (Delayed Reaction)** may want to frame late-game
  reactivity as "pool pressure" rather than "player tracking" — a subtle but
  important distinction for the fairness narrative.

---

## Open Questions

1. Does Level 1 reactivity (AIs become more aggressive in their lane after
   pick 5, responding to pool thinning) actually feel different to the player
   from Level 0? If the increased urgency tracks pool state rather than player
   picks specifically, it may be indistinguishable — but simulation would need
   to confirm this.

2. How many runs before a player "solves" a Level 0 system with fixed AI
   personalities? The Arena bot evidence suggests players discover bot tendencies
   over time. How does Dreamtides' roguelike context (each run is relatively
   infrequent vs. grinding Arena) affect the discovery rate?

3. Does the "puzzle vs. competition" distinction matter in a roguelike where
   the draft is a means to an end (building a deck for combat) rather than the
   primary competitive venue? Players who care primarily about the battle may
   not notice or care whether AIs feel like real opponents.

4. Can AIs behave "imperfectly" (occasional off-archetype picks) in a way that
   creates the right unpredictability without creating the wrong patterns
   (players discovering and exploiting the imperfection logic)?
