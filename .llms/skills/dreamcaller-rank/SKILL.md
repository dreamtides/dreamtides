---
name: dreamcaller-rank
description: Rank an anonymized card pool for how highly a drafter already committed to a given dreamcaller should pick each card. Uses staged subagent judgment, second-order synergy analysis, and deterministic UUID-based merge. Triggers on dreamcaller ranking, rank cards for a dreamcaller, dreamcaller pick order, card pick order for dreamcaller, or draft ranking for a dreamcaller.
---

# Dreamcaller Ranking

Rank every card in an anonymized pool for `pack 1 pick 1 after committing to this dreamcaller`.
The goal is a strict `best -> worst` ordering of the whole pool, not just a shortlist.

Use subagents when available. The task is intentionally staged: dreamcaller modeling,
broad first-pass scoring, anchor extraction, second-pass refinement, close-cluster
reconciliation, then deterministic merge into one final ranking.

Read `docs/battle_rules/battle_rules.md` before ranking. The agent and every ranking
subagent must ground its judgments in Dreamtides rules vocabulary and timing.

This skill is intentionally a fresh-perspective evaluation based only on the user-provided
dreamcaller text and anonymized card pool.

## Transition Context

This skill exists during an active transition where **tides are being removed from the game**
and **rarity is being removed from the game**.

Some exports may still contain legacy `tide` or `rarity` fields while the migration is in
progress. Treat those fields as migration residue, not as strategic truth.

- Do **not** reason from tide identity, tide balance, or tide lane assumptions.
- Do **not** reason from rarity, scarcity, or "this is rare so I can get filler later."
- If a legacy export includes tide or rarity, ignore those fields for card evaluation.
- Judge only from the dreamcaller text, the anonymized card text/stat line, and
  `docs/battle_rules/battle_rules.md`.

Do **not** read:
- `docs/tides/tides.md`
- `rules_engine/tabula/rendered-cards.toml`
- `rules_engine/tabula/cards.toml`
- `docs/resonance/resonance.md`
- any file whose purpose is to assign cards to archetypes, resonances, tides, rarity
  buckets, or existing deck identities

Do **not** use any prior archetype labels, tide associations, rarity heuristics, or
card-pool curation metadata as evidence.

## Objective

Interpret the question as:

`If I am already committed to this dreamcaller, which pick most increases the expected value
of the finished deck if I take it early?`

This is **not**:
- "Which cards are the most obviously on-theme?"
- "Which cards are best in a finished 40-card list after I already have all the pieces?"
- "Which cards have the prettiest direct text match?"

Raw power matters a lot. A premium generic removal spell usually outranks a merely decent
on-plan card. Close calls should break toward dreamcaller fit.

## Inputs

The user provides:

- Dreamcaller rules text as plain text
- Path to an anonymized card pool export

Assume the export already exists. Do not regenerate it from `rendered-cards.toml` unless the
user explicitly asks. Even if the repo contains richer source data, ignore it for this skill.

## Expected Card Pool Format

Use JSONL unless the user specifies another format. Each line should be one card with:

```json
{
  "uuid": "string",
  "card_type": "Character | Event",
  "cost": 0,
  "spark": 0,
  "subtype": "string or null",
  "is_fast": false,
  "rendered_text": "string",
  "tide": "string | null, optional legacy field - ignore",
  "rarity": "string | null, optional legacy field - ignore"
}
```

Notes:
- `uuid` is required and is the only identity field used in the final output.
- Ignore anonymized placeholder names if present. They are not reliable identifiers.
- If `tide` or `rarity` appear, they are legacy export fields and must not affect scoring.

## Output

Produce a strict ranking of the full pool sorted `best -> worst`.

Default output format:

```text
uuid,score,rendered_text
uuid,score,rendered_text
...
```

Score range: `0-100`.

Interpret scores as draft-value bands, not win-rate estimates:
- `95-100`: absurd bomb, nearly always first-pickable
- `85-94`: premium early pick
- `70-84`: strong pick
- `55-69`: solid role-player or narrower synergy piece
- `40-54`: replaceable support
- `0-39`: low-priority, weak, or meaningfully off-plan

The final output must contain only `uuid,score,rendered_text`.

## Core Ranking Lens

Every card should be judged on the combination of these questions:

1. **Raw power**
   How strong is this card if I ignore the dreamcaller?
2. **Direct dreamcaller fit**
   Does this card directly exploit what the dreamcaller rewards or enables?
3. **Infrastructure / second-order fit**
   Does this card make the direct-fit cards more reliable, more numerous, or more punishing?
4. **Dependency**
   Is this card good now, or only after I already have several specific pieces?
5. **Replaceability**
   If this card is omitted, how easy is it to replace its job with another plausible pick?
6. **Openness**
   How much should this dreamcaller actually bend early picks away from generic power?
7. **Anti-synergy**
   Does this card meaningfully push away from the dreamcaller's best path?

### Openness Rubric

Use one of these labels in the dreamcaller model and let it materially change the ranking:

- **Narrow**
  The dreamcaller sharply rewards a specific engine, trigger family, board state, or deck
  composition. Early picks should move substantially toward payoffs, enablers, and glue that
  make that engine real. A merely strong generic card can lose to a clearly on-plan card.
- **Medium**
  The dreamcaller creates a real direction, but many generically strong cards still fit well.
  Synergy should move cards by a meaningful tier, but not flatten obviously stronger generic
  cards without a concrete reason.
- **Open**
  The dreamcaller is mostly a nudge or tie-breaker. Generic power should dominate unless a
  card has an unusually clean synergy payoff. Do not force the ranking to look themed if the
  dreamcaller does not actually demand that.

The key question is not "does the dreamcaller mention this card?" It is "how much should this
dreamcaller distort a rational early-pick order?"

### Important Judgment Rules

- **Bomb override:** truly generic bombs stay near the top even if their synergy is modest.
- **No fake recursion loops:** do not keep boosting a card because it supports a card that
  supports another good card. Multi-hop synergy is real, but its weight decays quickly.
- **Stop at generic adjacency:** once the argument becomes "this is good with good cards,"
  the second-order chain has run out.
- **On-plan is good:** do not talk yourself out of a strong enabler or glue card just because
  similar-looking support effects may exist somewhere else.
- **Replaceability is real:** you may lower a card if its job is easy to fill later, but do so
  from the card text and pool context, not from legacy rarity assumptions.
- **Close ties break toward fit:** if two cards are close in expected-value terms, prefer the
  one that better matches the dreamcaller.

## Roles To Use Internally

Use these labels internally to keep subagents calibrated, even though the final output omits
them:

- `bomb`
- `generic_premium`
- `direct_payoff`
- `enabler`
- `infrastructure`
- `glue`
- `narrow_dependent`
- `anti_synergy`

Cards may have more than one internal role, but pick one dominant role when scoring.

## Workflow

### Phase 0: Create a Run-Isolated Workspace

Every run must use its own directory. Never reuse a previous run directory. Never write stage
artifacts into shared `/tmp/dreamcaller-rank` paths.

Example:

```bash
RUN_DIR="/tmp/dreamcaller-rank-$(date +%Y%m%d-%H%M%S)-$$"
mkdir -p \
  "$RUN_DIR/chunks" \
  "$RUN_DIR/stage1" \
  "$RUN_DIR/stage2" \
  "$RUN_DIR/windows/input" \
  "$RUN_DIR/windows/output"
```

Inside the run directory, keep:

- `cards.jsonl` or another one-time normalized export
- `dreamcaller.txt`
- `dreamcaller_model.md`
- `chunks/manifest.json`
- `stage1/*.jsonl`
- `stage1-merged.csv`
- `anchors.json`
- `anchors.md`
- `stage2/*.jsonl`
- `stage2-merged.csv`
- `windows/input/*.jsonl`
- `windows/input/*.meta.json`
- `windows/output/*.jsonl`
- `final.csv`

If the input format differs from JSONL, adapt it once up front and write the normalized result
to `$RUN_DIR/cards.jsonl`. Do not let every subagent invent its own parser.

Write the dreamcaller text to `$RUN_DIR/dreamcaller.txt` so every later stage can reference the
same exact text.

### Phase 1: Model the Dreamcaller

Before ranking any cards, write a short internal analysis to `$RUN_DIR/dreamcaller_model.md`
containing:

- the dreamcaller's direct hooks
- what the dreamcaller is trying to assemble
- what classes of cards it actively wants
- what classes of cards it merely tolerates
- what kinds of cards are generic premiums even if not especially synergistic
- whether the dreamcaller is **narrow**, **medium**, or **open**
- one short paragraph explaining *why* that openness label is correct

Do not overfit to exact words. Infer the deck's desired play pattern.
Re-read `docs/battle_rules/battle_rules.md` if the dreamcaller depends on timing, board state,
Judgment, void, reclaim, fast cards, or subtype interactions.
Do not consult tides, rarity, resonance, archetype, or source-pool files to answer these
questions.

Example:

`With 3 allied Survivors, cards in your void have reclaim equal to their cost.`

This is not only "Survivor tribal." It also wants:
- cards that help keep Survivors on the battlefield
- cards that stock the void
- cards that become much better when reclaim is live
- glue cards that connect the survivor-count requirement to the void engine

### Phase 2: Split the Pool

Use the helper script:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/split_jsonl.py \
  "$RUN_DIR/cards.jsonl" "$RUN_DIR/chunks" --chunk-size 60
```

This creates chunk files containing disjoint card sets and writes `manifest.json`.
Treat `"$RUN_DIR/chunks/manifest.json"` as the source of truth for:

- total expected record count
- chunk filenames
- chunk boundaries

Never give two subagents the same output file.

### Phase 3: First-Pass Chunk Scoring

Spawn parallel subagents over chunk files when available. Each subagent should score every
card in its chunk and write JSONL output with at least:

```json
{
  "uuid": "string",
  "score": 78,
  "tie_break": 2,
  "rendered_text": "string",
  "role": "direct_payoff",
  "note": "short internal note"
}
```

First-pass scoring should be broad and fast. The purpose is to cover the whole pool once and
to surface anchor candidates.

The `tie_break` field is internal. Use higher values for stronger dreamcaller fit when cards
are otherwise close.

Write each result to a run-local path such as:

- input: `$RUN_DIR/chunks/chunk-001.jsonl`
- output: `$RUN_DIR/stage1/chunk-001.jsonl`

After every chunk finishes, validate it before using it:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/chunks/chunk-001.jsonl" \
  "$RUN_DIR/stage1/chunk-001.jsonl" \
  --exact
```

Do not proceed past Phase 3 until every chunk output exists and validates against its source
chunk.

#### First-Pass Prompt Template

```text
You are ranking one chunk of an anonymized Dreamtides card pool for pack 1 pick 1 after
committing to this dreamcaller:

[dreamcaller text]

Before ranking, read docs/battle_rules/battle_rules.md.
This project is currently removing tides and rarity from the game. If any legacy export fields
mention tide or rarity, ignore them completely.
Do not read docs/tides/tides.md, rendered-cards.toml, cards.toml, resonance docs, or any
existing archetype-assignment material.

Also read:
- [dreamcaller model path]

Rank every card in [chunk path]. Use 0-100 draft-value scores.

Focus on:
- raw power
- direct dreamcaller fit
- infrastructure / second-order fit
- dependency
- replaceability
- openness

Rules:
- process every card in the chunk exactly once
- write output JSONL to [output path]
- include uuid, score, tie_break, rendered_text, role, and a short internal note
- preserve rendered_text exactly from the input chunk
- generic bombs stay high
- close calls break toward dreamcaller fit
- do not use placeholder card names as identifiers
```

### Phase 4: Merge Stage 1 and Extract Global Anchors

First produce a validated stage-1 merge:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  "$RUN_DIR"/stage1/*.jsonl \
  --output "$RUN_DIR/stage1-merged.csv" \
  --expected-jsonl "$RUN_DIR/cards.jsonl"
```

Then extract global anchors and write them to both:

- `$RUN_DIR/anchors.json`
- `$RUN_DIR/anchors.md`

Anchor extraction is the most important thinking step in the workflow. Do **not** treat it as
"copy the top-scoring cards into three buckets." Anchors are the cards that explain why other
cards should move.

Create exactly three anchor groups:

- `bombs_or_premiums`
- `direct_payoffs`
- `infrastructure`

Target roughly `8-20` cards in each group. Smaller is better than bloated.

Each anchor entry should include:

```json
{
  "uuid": "string",
  "score": 88,
  "rendered_text": "string",
  "anchor_type": "direct_payoffs",
  "reason": "why this card is an anchor rather than just a good card",
  "wants": "what kinds of neighboring cards it pulls upward",
  "confidence": "high | medium"
}
```

Use these rules when selecting anchors:

- **Bombs / premiums:** cards that stay near the top with only modest dreamcaller help.
  Their job is to stop the ranking from becoming fake-themes-only.
- **Direct payoffs:** cards the dreamcaller makes substantially better, more reliable, or more
  punishing. These are the cards the deck is happiest to end up with.
- **Infrastructure:** cards that make the payoff plan happen more often. They remove bottlenecks,
  increase trigger density, stabilize board conditions, feed the right zone, or connect two
  otherwise separate requirements.

Use these anti-rules:

- Do **not** include a card as an anchor merely because it scored well.
- Do **not** let direct payoffs crowd out infrastructure.
- Do **not** call a card infrastructure if the only reason is "good cards like good cards."
- Do **not** use legacy tide or rarity information to break ties.
- If an anchor would not plausibly cause any other card to move, it is probably not an anchor.

For a non-obvious card, ask:

`Does this card make one or more anchor plans materially better, more reliable, more numerous,
or easier to draft into?`

If yes, lift it. If the support claim is too indirect or too generic, do not.

Before moving on, sanity-check the anchor file:

- there is at least one real infrastructure anchor
- the anchor sets are not just the same cards copied three times
- each anchor has a concrete reason and a concrete "wants" clause
- the anchor file still makes sense if tides and rarity are ignored

### Phase 5: Second-Pass Refinement

Run a second pass over the pool, again by chunk, but now include the dreamcaller model and the
anchor artifact.

Use this pass to fix the common failure mode where only direct matches rise while enabling
cards stay too low.

Write each refined chunk to:

- input: `$RUN_DIR/chunks/chunk-001.jsonl`
- output: `$RUN_DIR/stage2/chunk-001.jsonl`

Validate every refined output before it is accepted:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/chunks/chunk-001.jsonl" \
  "$RUN_DIR/stage2/chunk-001.jsonl" \
  --exact
```

#### Second-Pass Prompt Template

```text
You are refining a chunk-level Dreamcaller draft ranking.

Dreamcaller:
[dreamcaller text]

Before refining, read docs/battle_rules/battle_rules.md.
This project is currently removing tides and rarity from the game. If any legacy export fields
mention tide or rarity, ignore them completely.
Do not read docs/tides/tides.md, rendered-cards.toml, cards.toml, resonance docs, or any
existing archetype-assignment material.

Also read:
- [dreamcaller model path]
- [anchors.md path]

For each card in [chunk path], write a refined JSONL row to [output path] with:
- uuid
- score
- tie_break
- rendered_text

Guidelines:
- raise cards that materially improve the anchor plans
- allow multi-hop support, but with fast decay
- do not create self-justifying loops
- keep truly generic bombs high
- let the openness label matter
- preserve rendered_text exactly from the input chunk
```

### Phase 6: Merge Stage 2 and Resolve Missing-Card Failures

Before any reconciliation work, produce a validated stage-2 merge:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  "$RUN_DIR"/stage2/*.jsonl \
  --output "$RUN_DIR/stage2-merged.csv" \
  --expected-jsonl "$RUN_DIR/cards.jsonl"
```

If this merge fails because cards are missing, duplicated, or unexpected:

1. Stop and identify which chunk outputs are invalid or absent.
2. Re-run only the missing or invalid chunks.
3. Re-validate those chunk outputs with `validate_rankings.py`.
4. Re-run the stage-2 merge.
5. Do not proceed until the merge succeeds.

A partial merge is not a valid intermediate result.

### Phase 7: Reconcile Only the Close Clusters

Do not spend agent budget globally re-ranking the whole pool again.

Instead:

1. Use `stage2-merged.csv` as the provisional global order.
2. Materialize reconciliation windows as explicit files in `"$RUN_DIR/windows/input"`.
3. For each window, also write a matching metadata file in
   `"$RUN_DIR/windows/input/window-XXX.meta.json"`.
4. Run reconciliation subagents only against those explicit window files.
5. Validate each window output against its own window input before accepting it.

Each window metadata file should include:

```json
{
  "window_id": "window-001",
  "reason": "top_of_pool | large_tie_band | boundary_cluster",
  "start_rank": 1,
  "end_rank": 25,
  "max_score_adjustment": 4
}
```

Window construction guidance:

- **Top of pool:** always make at least one window covering roughly the top `20-30` cards.
- **Large tie bands:** make windows where many cards sit within a very small score spread.
- **Boundary clusters:** make windows where adjacent cards are close enough that local order
  matters for draft decisions.
- **Overlap:** adjacent windows may overlap by `5-10` cards so local ordering can stabilize
  across boundaries.

Do **not** create windows for the whole tail of the pool. Reconciliation is for the high-value
parts of the ranking.

For each window:

- the input file must contain only the cards in that window
- the subagent may reorder only those cards
- the subagent may emit rows only for those UUIDs
- the subagent may adjust scores, but by no more than the metadata cap unless there is a very
  explicit reason
- the subagent must preserve `rendered_text` exactly
- the subagent must not introduce new archetype, tide, or rarity reasoning

Validate every window result before it is used:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/windows/input/window-001.jsonl" \
  "$RUN_DIR/windows/output/window-001.jsonl" \
  --exact
```

If a window output fails validation, discard it and rerun that window instead of letting a bad
file contaminate the final merge.

#### Reconciliation Prompt Template

```text
You are reconciling a small local ranking window inside a Dreamcaller draft order.

Dreamcaller:
[dreamcaller text]

Read:
- docs/battle_rules/battle_rules.md
- [dreamcaller model path]
- [anchors.md path]
- [window metadata path]

This project is currently removing tides and rarity from the game. Ignore any legacy export
fields about tide or rarity.

Your job:
- read only the cards in [window input path]
- reorder only those cards
- keep the comparison local
- preserve rendered_text exactly
- output one JSONL row per input card to [window output path]

Guidelines:
- stabilize close calls near the top and around score ties
- do not reinvent the whole ranking
- do not emit cards outside the window
- do not use generic "good with good cards" logic as a reason
- respect the window score-adjustment cap unless the current local order is clearly wrong
```

### Phase 8: Deterministic Final Merge

Use the merge helper script. Pass files in coarse-to-fine order so later stages override
earlier ones:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  "$RUN_DIR"/stage1/*.jsonl \
  "$RUN_DIR"/stage2/*.jsonl \
  "$RUN_DIR"/windows/output/*.jsonl \
  --output "$RUN_DIR/final.csv" \
  --expected-jsonl "$RUN_DIR/cards.jsonl"
```

The script:

- dedupes by UUID
- lets later files override earlier entries
- validates the final UUID set against the original normalized input
- sorts by `score desc`, then `tie_break desc`, then `uuid`
- emits final `uuid,score,rendered_text`

Do not return a final output file unless this final merge succeeds.

## Practical Notes

- Use a unique run directory under `/tmp` unless the user requests a repo path.
- If subagents are unavailable, run the same workflow locally in multiple passes.
- Do not waste effort on long written explanations. The product is the ranking.
- If the final merge reports missing cards, treat that as a blocking failure and fix coverage.
- If the final merge reports unexpected cards, treat that as a contamination failure and find
  the bad stage output before continuing.
- Preserve the exact `rendered_text` field from the card record in every intermediate ranking
  file so the final merge can emit it without re-reading the source pool.
- Treat any attempt to import prior archetype, tide, or rarity knowledge as contamination of
  the evaluation and avoid it.

## Failure Modes

- **Too literal:** only subtype matches rise; glue and infrastructure stay too low.
- **Too generic:** the dreamcaller barely affects the ranking.
- **Openness collapse:** every dreamcaller produces almost the same top-20 because the openness
  label never materially changes the weighting.
- **Recursive hand-waving:** "good with good cards" becomes an infinite reason generator.
- **No bomb override:** powerful generically great cards sink for no good reason.
- **Anchor drift:** the anchor file is just a top-cards list with no real causal value.
- **Shared temp contamination:** two runs reuse the same temp paths or stale files survive into
  a new run.
- **Missing-card merge:** a stage drops UUIDs and the workflow keeps going anyway.
- **Window contamination:** a reconciliation file emits cards outside its intended window.
- **Dropped rules text:** final rows lose the original `rendered_text`.
- **Legacy leakage:** tide or rarity fields sneak back into the reasoning.
- **Too much global re-ranking:** spending most of the budget adjudicating low-impact tail
  cards.

## Minimal Success Criteria

The run is successful only if all of these are true:

- every input UUID appears exactly once in the final output
- the final output is strictly ordered `best -> worst`
- every final row includes the card's `rendered_text`
- obvious direct-fit cards rise appropriately
- at least some non-obvious infrastructure cards rise for defensible second-order reasons
- generic bombs still appear near the top when warranted
- the final reasoning is not contaminated by tide or rarity heuristics
