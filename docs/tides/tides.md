# Dreamtides Tides

## Core Idea

A tide is a named gameplay package. It describes a real battle pattern: the kind
of board a deck wants, the resources it converts, the timing it rewards, and the
way it closes games.

Tides are not factions, flavor-only labels, or tiny mechanic tags. A tide exists
when it changes how cards play together on the battlefield. The question is
always what a card is doing in game terms: what board state it creates, what
resource loop it feeds, what timing window it exploits, and what payoff it is
helping a deck reach.

Every tide should have both:

- a narrative identity: what the deck feels like it is trying to do
- a mechanical identity: the concrete game actions and payoffs that define it

## Tide Layers

Dreamtides uses three tide layers.

- Structural tides are full shells. They own the main payoff cards, the central
  engines, and the finishers that define a deck's primary plan.
- Support tides are splashable technique packages. They own setup, smoothing,
  bridges, low-commitment role players, and enablers that reinforce a shell
  without being that shell's main payoff.
- Utility tides are broad role packages. They own generally useful curve,
  interaction, selection, refuel, and closing tools that do one clean job
  without asking for a dedicated density or threshold.

In practice:

- Structural tides own "do the thing, get paid" cards.
- Support tides own "help the deck do the thing more often" cards.
- Utility tides own "this is broadly good regardless of shell" cards.

## Applying Tides To Cards

Apply tides by battlefield function, not by surface wording.

- A card belongs to a structural tide when it is a real payoff, engine, or
  closer for that plan.
- A card belongs to a support tide when it is setup, smoothing, or a bridge that
  another shell can use without needing the full payoff package.
- A card belongs to a utility tide when its job is broadly useful and remains
  clear outside any dedicated shell.
- Mentioning a mechanic is not enough. A card should only carry a tide if it
  actively advances that tide's actual game plan.

Several boundaries matter because they separate similar-looking cards into
different play patterns:

- `materialize_value` owns repeatable ETB and materialized value.
  `materialize_tempo` owns return, temporary banish, replay timing, and pressure
  sequencing.
- `ally_formation` owns true multi-ally scaling and pair-based battlefield
  formation. `ally_wide` owns generic flood plans and non-scaling wide-board
  pressure.
- `character_chain` owns explicit second-character and deploy-chain reward
  cards. `character_curve` owns cheap character density and quantity without
  chain triggers.
- `void_recursion` owns reclaim, replay, and recursive threats. `void_threshold`
  owns count-the-void scaling and threshold finishers.
- `abandon_value` owns abandon as resource conversion and loop fuel.
  `abandon_ladder` owns abandon into upgrade, deck-cheat, and escalation chains.
- `prevent_control` owns reactive denial and payoff for stopping cards.
  `tax_pressure` and `hand_disruption` own proactive taxes and hand attack.

## Structural Tides

Structural tides are the dream's main doctrines. Each one supplies a complete
way to win games.

### `warrior_pressure`

Narrative identity: Relentless martial momentum.

Mechanical identity: Low-curve Warriors, direct spark buffs, point racing, and
aggressive battlefield snowball.

Apply to cards that build early Warrior boards, push damage immediately, or turn
Warrior density into pressure.

### `warrior_bastion`

Narrative identity: Disciplined defense and superior trading.

Mechanical identity: Sticky Warriors, favorable trades, attrition tools, and
defensive board control.

Apply to cards that make Warrior boards hard to clear, reward holding ground, or
grind through combat.

### `spirit_growth`

Narrative identity: Living abundance that compounds over time.

Mechanical identity: Spirit Animals, ramp, top-of-deck play, board snowball, and
Judgment energy and spark chaining.

Apply to cards that ramp into larger Spirit turns, reward Spirit density, or
convert Judgment into more board.

### `materialize_value`

Narrative identity: Repeated arrival as a source of advantage.

Mechanical identity: ETB value, materialized triggers, copy effects tied to
entry, and steady resource gain.

Apply to cards that are premium materialize targets or directly reward repeated
entries.

### `materialize_tempo`

Narrative identity: Reality in motion, never letting the board settle.

Mechanical identity: Bounce, temporary banish, replay timing, fast pressure, and
tempo swings.

Apply to cards that remove blockers for a turn, reset allies for timing value,
or use replay to create pressure windows.

### `ally_formation`

Narrative identity: Coordinated companies moving in deliberate formation.

Mechanical identity: Multi-ally payoffs, pair scoring, formation spark scaling,
and disciplined board geometry.

Apply to cards that reward exact ally counts, adjacent support patterns, or
pair-based scaling.

### `ally_wide`

Narrative identity: Overwhelming presence through sheer board spread.

Mechanical identity: Generic go-wide battlefield shells, token-flood finishes,
and non-scaling wide pressure.

Apply to cards that flood the board with bodies and turn width itself into the
win condition.

### `fast_tempo`

Narrative identity: Winning through timing and initiative.

Mechanical identity: Dense fast cards, hand-fast enablers, opponent-turn plays,
and explicit fast-payoff bodies.

Apply to cards that care that a card was played fast or create reactive pressure
without giving up tempo.

### `event_chain`

Narrative identity: Spell turns that build into a decisive burst.

Mechanical identity: Event density, cost reduction, copies, burst sequencing,
and cards-played-this-turn finishers.

Apply to cards that reward long event turns, sequencing density, or storm-style
chaining.

### `prevent_control`

Narrative identity: Command of the game through refusal and interruption.

Mechanical identity: Hard prevents, counterspell pressure, reactive events, and
denial-as-engine value.

Apply to cards that stop opposing plays on the stack or convert prevented cards
into advantage.

### `discard_velocity`

Narrative identity: Burning through the hand to gain speed.

Mechanical identity: Self-discard, hand churn, burst draw, and discard-fueled
tempo.

Apply to cards that want cards discarded as cost or payoff and turn churn into
speed.

### `void_recursion`

Narrative identity: The void as a second hand and a place of return.

Mechanical identity: Reclaim, replay from void, recursive threats, and
void-as-hand patterns.

Apply to cards that come back from the void, cast from it, or reward repeated
rebuy loops.

### `void_threshold`

Narrative identity: Power that grows as the void deepens.

Mechanical identity: Void count scaling, threshold bodies, and count-based
finishers.

Apply to cards that care how many cards are in the void and become threatening
at specific void sizes.

### `abandon_value`

Narrative identity: Sacrifice as fuel for resource conversion.

Mechanical identity: Abandon for energy, cards, attrition value, and sacrifice
loops.

Apply to cards that turn leaving play into cards, energy, or repeatable
conversion.

### `abandon_ladder`

Narrative identity: Feeding lesser pieces upward into greater ones.

Mechanical identity: Abandon to upgrade, abandon into deck cheat,
materialize-bigger chains, and deck-as-resource finishing.

Apply to cards that cash in expendable bodies to pull larger cards or step up a
board state.

### `figment_swarm`

Narrative identity: A dream made real through multiplying fragments.

Mechanical identity: Figment generation, figment multiplication, figment-tribal
payoffs, and token-board finishes.

Apply to cards that create figments, reward figment count, or turn token mass
into lethal pressure.

### `survivor_dissolve`

Narrative identity: Endurance through death, recovery, and attrition.

Mechanical identity: Survivors, Dissolved triggers, death loops, void rebuys,
and sticky attrition.

Apply to cards that reward allied cards dissolving or repeatedly recycle
dissolved pieces.

### `judgment_engines`

Narrative identity: Winning by bending the scoring phase itself.

Mechanical identity: Extra Judgment phases, repeated Judgment triggers,
phase-scaling bodies, and Judgment payoff turns.

Apply to cards that care specifically about Judgment happening, happening again,
or happening bigger.

### `character_chain`

Narrative identity: Momentum built from body after body entering play.

Mechanical identity: Second-character payoffs, deploy rebates, replay-on-deploy
chains, and character-play cost reduction.

Apply to cards that pay off multiple character plays in a turn or convert
character sequencing into tempo.

### `spark_tall`

Narrative identity: Concentrating power into a single dominant threat.

Mechanical identity: Kindle, focused spark growth, board compression, and
one-threat or two-threat pressure.

Apply to cards that stack spark on a small number of bodies and make that
concentration the plan.

## Support Tides

Support tides are techniques, bridges, and setup packages. They skew how a
structural shell plays without replacing that shell's main payoff cards.

### `big_energy`

Narrative identity: Power spikes and overcharged turns.

Mechanical identity: Temporary and permanent energy bursts plus flexible sinks.

Apply to cards that produce surplus energy or give decks a place to spend it.

### `fast_setup`

Narrative identity: Reactive posture without needing dedicated fast payoffs.

Mechanical identity: Cheap fast cards, timing tools, and opponent-turn texture.

Apply to cards that make fast play easier or more frequent without being a fast
reward card.

### `hand_cycling`

Narrative identity: Constant hand sculpting and motion.

Mechanical identity: Looting, rummaging, hand filtering, and redraw.

Apply to cards that turn weak cards into better ones and smooth sequencing.

### `reclaim_characters`

Narrative identity: Recovering bodies for another deployment.

Mechanical identity: Character rebuy and replay setup.

Apply to cards that return characters from the void or hand them back for reuse.

### `reclaim_events`

Narrative identity: Recasting key spells.

Mechanical identity: Event rebuy and replay setup.

Apply to cards that let a deck get more uses out of events without being an
event-count payoff.

### `spark_growth`

Narrative identity: Quiet preparation for a tall payoff.

Mechanical identity: Direct spark buffs, kindle tools, and tall-board setup.

Apply to cards that add spark efficiently without being the main tall reward.

### `go_wide_enablers`

Narrative identity: Building width before rewards arrive.

Mechanical identity: Cheap extra bodies, token makers, deployment smoothing, and
board support.

Apply to cards that create wide boards without being an ally-count or tribal
payoff.

### `leave_play_enablers`

Narrative identity: Making departure matter.

Mechanical identity: Sacrifice, bounce, banish, and dissolve bridges.

Apply to cards that cause allied cards to leave play in ways other shells can
exploit.

### `bounce_blink_tools`

Narrative identity: Reusing entries through movement.

Mechanical identity: Ally return, temporary banish, replay setup, and blink
infrastructure.

Apply to cards that reset allies or create low-commitment replay patterns.

### `topdeck_setup`

Narrative identity: Preparing the next draw before it matters.

Mechanical identity: Top-of-deck access, deck smoothing, deck stocking, and
light deck-cheat setup.

Apply to cards that arrange, reveal, or preload the top of the deck for later
turns.

### `void_setup`

Narrative identity: Stocking the void for future use.

Mechanical identity: Self-mill, discard-to-void, threshold setup, and void
loading.

Apply to cards that place cards into the void efficiently without being the main
void payoff.

### `judgment_repeaters`

Narrative identity: Extending the scoring rhythm.

Mechanical identity: Extra phases, trigger copying, and generic Judgment setup.

Apply to cards that cause Judgment abilities to happen again without being the
payoff for it.

### `event_setup`

Narrative identity: Frictionless spell sequencing.

Mechanical identity: Cheap events, cantrips, cost smoothing, and sequencing
tools.

Apply to cards that make event-heavy turns more reliable without being the event
finisher.

### `copy_effects`

Narrative identity: Borrowing the same action twice.

Mechanical identity: Event copies, trigger copies, and one-shot duplication
tools.

Apply to cards that duplicate effects in a generally splashable way.

### `abandon_fodder`

Narrative identity: Supplying expendable material.

Mechanical identity: Cheap bodies and generic abandon outlets.

Apply to cards that give sacrifice decks things they are happy to cash in.

### `cost_reduction`

Narrative identity: Making turns larger than the resource system expects.

Mechanical identity: Generic rebates, one-off discounts, and flexible cost
smoothing.

Apply to cards that reduce costs broadly without tying that reduction to one
shell's payoff.

### `trigger_reuse`

Narrative identity: Squeezing extra value from text already on board.

Mechanical identity: Tools that re-fire Materialized, Dissolved, or Judgment
abilities.

Apply to cards that explicitly make another card's triggered text happen again.

### `character_tutors`

Narrative identity: Finding the right body at the right time.

Mechanical identity: Search, reveal, and deck-to-hand fetch for characters.

Apply to cards that directly locate characters from the deck.

### `character_curve`

Narrative identity: Reliable pressure through body count alone.

Mechanical identity: Low-curve character density and cheap quantity.

Apply to cards that are just strong cheap characters for character-heavy decks
without chain text.

### `discard_outlets`

Narrative identity: Turning cards in hand into movement and setup.

Mechanical identity: Cheap self-discard tools, discard-as-cost activations, and
hand-to-void bridges.

Apply to cards that ask the player to discard and give useful compensation for
doing so.

### `recursion_fuel`

Narrative identity: Feeding graveyard-style engines.

Mechanical identity: Deep self-mill and void stocking without threshold or
reclaim payoffs.

Apply to cards that fill the void aggressively for decks that later exploit it.

### `attrition_trade`

Narrative identity: Trading resources cleanly and often.

Mechanical identity: Efficient one-for-one trades and small attrition payoffs.

Apply to cards that generate modest advantage from repeated exchanges and
removal.

### `tax_pressure`

Narrative identity: Winning by making every opposing action awkward.

Mechanical identity: Proactive taxes and cost-increase pressure.

Apply to cards that make the opponent pay more or sequence less efficiently.

## Utility Tides

Utility tides are common tools. They stay broadly playable and do one clear job.

### `cheap_curve`

Narrative identity: Reliable early footing.

Mechanical identity: Generically good 0-2 cost starters.

Apply to cards that are strong opening plays without synergy gating.

### `defensive_curve`

Narrative identity: Surviving the early game cleanly.

Mechanical identity: Blockers, reserve-friendly bodies, and stabilizers.

Apply to cards that buy time, absorb pressure, or make early races harder.

### `card_flow`

Narrative identity: Keeping resources moving.

Mechanical identity: Generic draw and hand refuel.

Apply to cards that provide cards without requiring a dedicated shell.

### `foresee_selection`

Narrative identity: Seeing the next turns before they happen.

Mechanical identity: Smoothing, selection, and setup.

Apply to cards that improve draw quality through foresee or adjacent selection
tools.

### `resource_burst`

Narrative identity: Short-term acceleration available to anyone.

Mechanical identity: Broad energy gain, rebates, and flexible sinks.

Apply to cards that give generic energy tempo without tying it to a
shell-specific engine.

### `cheap_removal`

Narrative identity: Efficient answers with conditions attached.

Mechanical identity: Fast or inexpensive but conditional removal.

Apply to cards that answer threats cheaply when some board or rules condition is
met.

### `premium_removal`

Narrative identity: Clean answers that trade rate for certainty.

Mechanical identity: Slower or unconditional removal.

Apply to cards that remove nearly anything without needing synergy support.

### `fast_interaction`

Narrative identity: Flexible disruption at the moment it matters.

Mechanical identity: Prevents, bounce, and combat-speed disruption.

Apply to cards that interrupt combat or stack play at fast speed in a generally
useful way.

### `hand_disruption`

Narrative identity: Attacking plans before they hit the board.

Mechanical identity: Discard, hand attack, and denial pressure.

Apply to cards that strip options from hand rather than countering them on the
stack.

### `finishers`

Narrative identity: Ending games once the setup is done.

Mechanical identity: Top-end threats, extra turns, and closing tools.

Apply to cards that are broadly playable closers instead of shell-specific
payoffs.

### `void_denial`

Narrative identity: Refusing the opponent a second life from the void.

Mechanical identity: Banish, void hate, and anti-recursion tools.

Apply to cards that shut off reclaim loops or punish void-centric decks.

### `discover_toolbox`

Narrative identity: Access over raw efficiency.

Mechanical identity: Discover, flexible search, narrow tutoring, and access
tools.

Apply to cards that trade rate for finding the right answer or role player.

### `judgment_bodies`

Narrative identity: Low-cost bodies that cash in on scoring.

Mechanical identity: Cheap characters with small Judgment triggers.

Apply to cards that mainly exist to provide a modest Judgment payoff on a
playable body.

### `materialized_staples`

Narrative identity: Good entry effects that need no shell.

Mechanical identity: Generically playable Materialized characters with one-shot
value.

Apply to cards that have respectable materialized text even when never blinked.

### `tempo_resets`

Narrative identity: Buying time by undoing the opponent's turn.

Mechanical identity: Bounce, top-of-deck resets, and full-hand resets.

Apply to cards that reset opposing development without permanently answering it.

### `point_pressure`

Narrative identity: Quietly forcing races to end sooner.

Mechanical identity: Small, low-commitment point gain and race tools.

Apply to cards that help almost any deck close games or convert a lead into
points.
