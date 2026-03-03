# Research: AI Drafting in Games

## How Existing Games Implement AI Drafters and What Makes Them Feel Real

---

## MTG Arena: The Primary Reference

### How Quick Draft Bots Work

MTG Arena's Quick Draft (released two weeks after Premier Draft) uses software
bots that simulate human opponents during the draft phase. The delay exists
specifically to observe how real players pick in Premier Draft, since bot
pick orders are calibrated against actual human pick-order data.

Key structural facts about Arena bots:

- **Bots have assigned personalities.** Each bot has archetype/color
  preferences — a "red bot" picks aggressively-valued cards higher, a "blue
  bot" prizes card advantage. These personalities are consistent across the
  entire draft.
- **Bots do not play matches.** Because bots only draft and never play the
  resulting deck, they lack the feedback loop that shapes human pick
  decisions. This is the fundamental structural difference from human drafters.
- **Bots assign baseline pick order by rarity.** The baseline ranking for a
  given card is derived from how likely high-performing human drafters are to
  pick it, adjusted for rarity. Rares are universally taken higher by bots
  than by humans — a deliberate Wizards design to reduce rare-drafting in
  Premier Draft while allowing bot drafts to be a source of rare card
  acquisition.
- **Pick logic has three phases (per the Draftsim model, which approximates
  Arena behavior):** Speculation (take highest-rated card, small on-color
  bonus), Commitment (commit to two-color pair with strong preference), Deck
  Construction. This mirrors a simplified human drafting pattern.

### 17Lands Data on Bot Signal Quality

17Lands' analysis found that Arena bots do mirror human drafters in one key
respect: cards passed by bots correlate meaningfully with what is "open."
The slope of signal value is steeper in bot drafts than human drafts,
suggesting that lane reading is slightly *more* rewarding in bot pods than
human pods — precisely because bot behavior is more consistent and predictable.

However, bots make systematic valuation errors that persist across entire
formats. In one format, bots undervalued black and green for an entire season,
creating a persistent exploit: draft into those colors, collect cards the bots
consistently pass, build powerful decks. Players who discovered this exploit
used it until the next set arrived.

### What Players Complain About

The failure modes players consistently identify:

1. **Predictability / exploitability.** "Once you figure out what the bots
   undervalue, you just draft that every time." The bots don't update. The
   puzzle is solved, then boring.
2. **Consistency as a tell.** Bots never pivot. A human drafter in red might
   see a late bomb and switch to blue. Bots don't. This means experienced
   players can narrow down a bot's entire pick path from its first pick and
   predict what will wheel. Human drafts don't work this way.
3. **Rare drafting.** Every bot takes every rare, period. This removes the
   interesting signal from rare cards and feels mechanical. If all bots take
   rares, a late rare tells you nothing about what's open.
4. **Missing holistic deckbuilding.** Bots pick cards individually without
   tracking what they've already drafted. A bot might take its 8th removal
   spell when a human would know it already has enough and take a creature
   instead.

### What Players Appreciate

1. **Consistent signal generation.** Because bots are predictable within a
   format, skilled players can study which archetypes bots over- and
   under-value, and exploit that knowledge for consistent results. This is
   a skill axis — it rewards format study.
2. **Availability / queue time.** Bot drafts are always available. No waiting
   for 8 humans.
3. **Specific lanes are reliably open.** If bots undervalue blue this format,
   blue is reliably open. Players who know this can plan their draft before
   it starts.

---

## Other Digital Card Games

### Hearthstone Arena

Hearthstone Arena is not a draft against AI opponents — it is a single-player
draft with no other drafters at all. Players pick 1 of 3 cards, 30 times.
There is no lane reading, no signal reading, no sense of competition during
the draft itself. The draft is entirely about maximizing card quality
given what is offered.

The player-facing experience is different from a draft-against-others model:
the draft phase is purely individual optimization, and competition happens
entirely in the play phase.

Third-party tools (HearthArena) model the draft by assigning baseline scores
to each card, then making micro-adjustments for synergy based on deck data.
This reveals what single-player draft optimization actually looks like in
practice: a tier list with synergy bonuses. There is no strategic "reading"
of opponents — just card evaluation.

**Implication for Dreamtides:** Hearthstone Arena is the wrong reference.
It shows what happens when you remove the competitive draft entirely.
The result is a cleaner puzzle but a shallower one, with no lane reading
and no game-to-game variety driven by opponent behavior.

### Legends of Runeterra Expeditions and Eternal Draft

Both games offer a useful negative case. LoR Expeditions had no AI drafters
competing for cards — the pool was effectively infinite, with each player
drafting independently. The competitive element happened only in the play
phase. Eternal Draft uses asynchronous human picks (stored from past
drafters) to populate packs, creating human-shaped lane signals without
any AI modeling of draft behavior.

Neither provides a reference for AI-drafter dynamics. Both demonstrate that
competitive tension during the draft itself is a specific design choice, not
a default. Eternal's asynchronous approach creates authentic signals and is
worth noting as an alternative — but it requires a live player base and is
not feasible for a roguelike with offline play.

---

## What Makes an AI Drafter Feel Real

### The Key Attributes

Research on making draft bots feel like human opponents converges on several
factors:

**1. Personality consistency within a draft, variation across drafts.**
Bots that are consistent within one draft (so their lane is readable) but
vary between drafts (so the player can't memorize "bot 3 always takes
Warriors") hit the right balance. A bot should feel like a player who has a
strategy, not a random number generator.

**2. Unpredictability within a lane, not across lanes.**
Bots feel fake when you can predict their exact pick order. Humans within
a lane still make surprising choices — taking a generic card that is
exceptionally powerful, passing a good card because they already have
enough of that type. Bots that take exactly the top card every time
feel mechanical. Adding occasional "imperfect" picks (a generic power card
over a slightly weaker archetype card) makes bots feel more like people.

**3. Deckbuilding awareness.**
The most-cited reason bots feel fake: they don't track what they've drafted.
Human drafters know when they have "enough" of something. The Ryan Saxe
bot model found that tracking draft state and biasing toward archetype
completion (not just card quality) produced significantly more human-like
behavior (91% archetype-appropriate picks at Pack 3 vs. 80% for humans in
the test set). This emergent behavior — not hard-coded — felt more natural.

**4. Reactions that make sense.**
When players "fight" a bot for its archetype, they expect the bot to react
in a way that makes sense: escalate (draft its archetype cards more
urgently), or pivot (move to a backup strategy). Bots that ignore what
the player is doing throughout the draft feel like they're in a separate
universe, not at the same table.

**5. The never-fully-human ceiling.**
Even the best ML-trained bot (NNetBot in the Ward 2020 study) achieved
~40% accuracy in predicting human picks. Humans don't pick optimally or
consistently — they have moods, preferences, intuitions, mistakes. An AI
that picks "optimally" every time will feel like an AI. Some deliberate
suboptimality is necessary for verisimilitude.

---

## The Open Lane Dynamic: What Makes Signal Reading Skillful

### How Real Draft Signal Reading Works

In human booster draft, signal reading is an indirect inference from hidden
information. You never see what others are drafting — you only see what they
passed. The signal is in the absence: a strong card appearing late (as a 7th
or 8th pick) means nobody to your right took it, implying that color/archetype
is open in those seats.

The skill is in:
- **Knowing the format well enough** to recognize when a card appears "too
  late" to be coincidental. You must know the expected pick position to detect
  the deviation.
- **Committing early enough to benefit.** Staying open too long means you
  miss the window when your open lane had maximum card density. Committing
  too early means you might have read a false signal. Pick 4-6 is typically
  the sweet spot in 8-person MTG drafts.
- **Distinguishing signal from noise.** One late card could be a signal or
  could be that one drafter just didn't like that card. Multiple late cards
  from the same archetype is a stronger signal.

The forcing vs. reading debate reveals an important dynamic: if everyone reads
signals, nobody establishes lanes, and the signals become meaningless. Signal
reading only works if some drafters force (stick to a strategy regardless of
signals). This creates a meta-game: do you read or force? The answer depends
on your read of the other drafters' strategies.

**Implication for Dreamtides:** The V10 framing of "AI opponents drafting
alongside you" creates this dynamic naturally. AI drafters establish lanes
(they force their archetype). The player has to decide whether to read
signals or commit to their own strategy. This is the right framing.

### Why Lane Reading Feels Skillful Rather Than Random

Lane reading feels skillful when:
- Signals are consistent and accumulate (multiple late cards from the same
  archetype, not just one).
- Committing earlier produces better outcomes (the skill has consequences).
- Reading correctly is causally linked to the good outcome, not just correlated.
  If an archetype is open, committing to it must actually produce more of
  those cards, not just happen alongside better results by chance.

Lane reading feels random when:
- Signals contradict each other (one pack suggests green is open, the next
  suggests it's taken).
- Commitment doesn't change outcomes (the deck is equally good regardless of
  when you commit).
- The open lane shifts mid-draft in ways the player couldn't predict.

**Implication for Dreamtides:** For V10, AI drafters need to be consistent
enough within a game that their lane stays "their lane" for the duration.
An AI that pivots mid-draft breaks the signal-reading skill axis — the player
can't trust what the early signals meant.

---

## Failure Modes of AI Drafters

**Too predictable.** The central MTG Arena failure: once players identify
which colors bots undervalue each format, the correct strategy is always
to draft there. The draft becomes a solved puzzle. Root cause: no mechanism
to vary bot preferences between games.

**Too random.** Bots that pick incoherently across archetypes produce mixed
signals — the player can never identify what's open. Variety exists but
feels meaningless.

**No deckbuilding judgment.** Bots that take the top archetype card every
pick without tracking their pool overload on single card types. Cards that
would be useful to a human drafter appear late only because the bot already
has 8 of that type. Produces artifact signals.

**Too aggressive in the player's lane.** If a player wanders into an AI's
archetype and immediately receives nothing viable, the experience feels
punitive. Design constraint: fighting an AI for its lane should produce a
weaker deck, not an unplayable one.

**Too passive.** If AIs take too few cards or only irrelevant ones, the pool
stays open and no signals form. The AI framing becomes decorative — the pool
behaves as if no one else is drafting.

---

## Key Synthesis for V10

The clearest design lessons from existing implementations:

**1. Per-AI archetype assignment is mandatory.** Each AI must have a fixed
lane for the duration of a game. Without this, no signals form.

**2. Consistency within games, variety across games.** AI lane assignments
should vary by game (so the player can't memorize which lanes are always open),
but remain consistent within a game (so signal reading is meaningful).

**3. Some deliberate imperfection is necessary.** Pure optimal-pick bots feel
mechanical. Occasional off-archetype picks (a powerful generic card, or
passing something because the AI already has enough) make the AI feel like
a person with a strategy, not a programmed picker.

**4. Signal reading does not require reactivity.** The 17Lands data shows
that even fully predetermined bots create actionable signals. Reactivity is
not required. What is required is that the AI's lane preference is consistent
and strong enough that its picks cluster in one archetype.

**5. Power-based overrides are a failure mode.** Arena bots taking every rare
regardless of lane destroys the informational value of late rares. In
Dreamtides, if all AIs take the highest-power card regardless of archetype,
signals become noisy and the draft feels random.

---

## Sources

- [AI solutions for drafting in Magic: the Gathering — Ward 2020](https://arxiv.org/pdf/2009.00655)
- [Magic Arena Bot Drafting — Zvi Mowshowitz](https://thezvi.substack.com/p/magic-arena-bot-drafting)
- [Bot Battle: Which Draft Bot Is the Best? — Draftsim](https://draftsim.com/draftsim-bot-drafting-paper/)
- [Bot Drafting the Hard Way — Ryan Saxe / Draftsim](https://draftsim.com/ryan-saxe-bot-model/)
- [Do the Bots Send Signals? — 17Lands](https://blog.17lands.com/posts/bot-signals/)
- [Arena Exploits: Beating Bots Black, Green, and Blue — Cardmarket](https://www.cardmarket.com/en/Insight/Articles/Arena-Exploits-Beating-Bots-Black-Green-and-Blue)
- [Drafting 101: Understanding Signals — Wizards](https://magic.wizards.com/en/news/feature/drafting-101-understanding-signals-2016-04-12)
- [To Read or To Force — CoolStuffInc](https://www.coolstuffinc.com/a/radarudyak-gatecrash-limited-03212013-to-read-or-force-a-drafting-quandary)
- [Study Aims for MTG Draft Bots to Mimic Player Behavior — Esports Talk](https://www.esportstalk.com/news/study-aims-for-mtg-draft-bots-to-mimic-player-behavior-for-better-games/)
