---
name: dreamcaller-rank
description: Rank an anonymized card pool for how highly a drafter already committed to a given dreamcaller should pick each card. Uses staged subagent judgment, second-order synergy analysis, and deterministic UUID-based merge. Triggers on dreamcaller ranking, rank cards for a dreamcaller, dreamcaller pick order, card pick order for dreamcaller, or draft ranking for a dreamcaller.
---

Run this using GPT-5.4 high subagents.

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
- "Which cards are strongest in a generic vacuum if I ignore the committed dreamcaller?"

This is a committed-dreamcaller ranking, not a generic P1P1 ranking. Dreamcaller fit and
infrastructure should matter more than standalone card quality. Treat **raw power as a tie-breaker
at most** unless a card is a true bomb or patches a structural weakness of the dreamcaller shell.
Premium generic removal should not default into the top 20 unless it is truly exceptional or also
fits the dreamcaller cleanly.

## Archetype-First Ranking

Default to this question:

`Does this card increase the expected quality and reliability of the dreamcaller deck more than the alternatives?`

Not this question:

`Would I first-pick this card in a generic draft?`

This distinction is mandatory. A card should not rank highly just because it is generically
efficient, flexible, or rate-positive. To justify a high rank, the preferred argument should be
about dreamcaller fit, infrastructure, shell reliability, generated resources, or anti-synergy
avoidance. Raw power may break close ties, but it should rarely be the main reason a card climbs.
Assume your raw power evaluations are noisy and lower-confidence than your fit evaluations.

## Inputs

The user provides:

- Dreamcaller rules text as plain text
- Path to an anonymized card pool export. Default to
  `rules_engine/tabula/dreamcaller_rank_card_pool.jsonl`.

Assume the export already exists. Do not regenerate it from `rendered-cards.toml` unless the
user explicitly asks. If the user does not specify a path, use
`rules_engine/tabula/dreamcaller_rank_card_pool.jsonl`.
Even if the repo contains richer source data, ignore it for this skill.

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

Default delivery behavior:
- write the completed ranking to `$RUN_DIR/final.csv`
- copy `$RUN_DIR/final.csv` to `notes/dreamcaller_rank_<timestamp>.txt` by default
- if the user requested a specific destination path, write the final file there instead
- return only a short pointer or note by default
- do **not** print the full ranking inline in assistant output unless the user explicitly asks for inline output

Score range: `0-100`.

Interpret scores as draft-value bands, not win-rate estimates:
- `95-100`: true format-warping bomb; expect very few of these in a full pool
- `85-94`: premium early pick
- `70-84`: strong pick
- `55-69`: solid role-player or narrower synergy piece
- `40-54`: replaceable support
- `0-39`: low-priority, weak, or meaningfully off-plan

The final ranking file must contain only `uuid,score,rendered_text`.
The assistant's chat output should be a short pointer to the written file, not the ranking rows.

## Core Ranking Lens

Every card should be judged on the combination of these questions:

1. **Direct dreamcaller fit**
   Does this card directly exploit what the dreamcaller rewards or enables?
2. **Infrastructure / second-order fit**
   Does this card make the direct-fit cards more reliable, more numerous, or more punishing?
3. **Resource conversion**
   What resource, timing window, body, card-flow pattern, or board pressure does the dreamcaller
   naturally create, and how well does this card convert that into value?
4. **Dependency**
   Is this card good now, or only after I already have several specific pieces?
5. **Replaceability**
   If this card is omitted, how easy is it to replace its job with another plausible pick?
6. **Openness**
   How much should this dreamcaller actually bend early picks away from generic power?
7. **Anti-synergy**
   Does this card meaningfully push away from the dreamcaller's best path?
8. **Raw power**
   If the dreamcaller fit arguments are close, which card has the stronger generic floor or ceiling?

Raw power belongs at the end of this lens on purpose. Do not begin from generic strength and then
add a small synergy bonus. Begin from committed-shell value and use raw power only to resolve close
calls or protect truly exceptional bombs. If you are unsure whether your power-level read is sound,
default toward the card whose dreamcaller role is clearer.

### Openness Rubric

Use one of these labels in the dreamcaller model and let it materially change the ranking:

- **Narrow**
  The dreamcaller sharply rewards a specific engine, trigger family, board state, or deck
  composition. Early picks should move substantially toward payoffs, enablers, and glue that
  make that engine real. A merely strong generic card can lose to a clearly on-plan card.
- **Medium**
  The dreamcaller creates a real direction. Synergy should move cards by a meaningful tier and
  should often beat generically strong but more situational or less structurally reliable cards.
- **Open**
  The dreamcaller is mostly a nudge or tie-breaker. Generic power should dominate unless a
  card has an unusually clean synergy payoff. Do not force the ranking to look themed if the
  dreamcaller does not actually demand that.

The key question is not "does the dreamcaller mention this card?" It is "how much should this
dreamcaller distort a rational early-pick order?"

### Important Judgment Rules

- **Bomb override:** use this rarely. A card is not a bomb just because its ceiling is huge; it
  should be strong in most plausible decks and not depend heavily on already having the right setup.
- **Raw power is a tie-breaker:** do not promote a card mainly because it is generically strong,
  efficient, or flexible. Prefer dreamcaller-fit arguments unless the comparison is truly close.
- **Power-level humility:** assume your raw-rate judgments are error-prone. Treat them as weak
  evidence unless the card is obviously exceptional across many plausible decks.
- **No fake recursion loops:** do not keep boosting a card because it supports a card that
  supports another good card. Multi-hop synergy is real, but its weight decays quickly.
- **Stop at generic adjacency:** once the argument becomes "this is good with good cards,"
  the second-order chain has run out.
- **Rate is not enough:** a cheap card, high-spark body, modal effect, or efficient interaction
  should not rise on rate alone if it does not improve the committed dreamcaller shell.
- **Situational ceiling is not raw power:** downgrade cards whose average case depends on specific
  setup, hand composition, or support density.
- **Generic removal is replaceable:** strong generic removal should usually trail premium payoffs,
  enablers, and infrastructure once committed to the dreamcaller, unless it patches a clear shell weakness.
- **On-plan is good:** do not talk yourself out of a strong enabler or glue card just because
  similar-looking support effects may exist somewhere else.
- **Replaceability is real:** you may lower a card if its job is easy to fill later, but do so
  from the card text and pool context, not from legacy rarity assumptions.
- **Close ties break toward fit:** if two cards are close in expected-value terms, prefer the
  one that better matches the dreamcaller.
- **Explain the shell contribution:** if you cannot state what job the card performs in the
  committed shell, it probably should not rank highly.

## Roles To Use Internally

Use these labels internally to keep subagents calibrated, even though the final output omits
them:

- `bomb`
- `generic_premium`
- `direct_payoff`
- `enabler`
- `infrastructure`
- `resource_converter`
- `glue`
- `narrow_dependent`
- `anti_synergy`

Cards may have more than one internal role, but pick one dominant role when scoring.
Use `generic_premium` sparingly. Most strong cards should still be classified in terms of what
they do for the dreamcaller shell.

## Workflow

### Phase 0: Create a Run-Isolated Workspace

Every run must use a fresh directory. Never reuse shared `/tmp/dreamcaller-rank` paths.

```bash
RUN_DIR="/tmp/dreamcaller-rank-$(date +%Y%m%d-%H%M%S)-$$"
mkdir -p \
  "$RUN_DIR/chunks" \
  "$RUN_DIR/stage1" \
  "$RUN_DIR/stage2" \
  "$RUN_DIR/windows/input" \
  "$RUN_DIR/windows/output"
```

Keep these run-local artifacts:
- `cards.jsonl`
- `dreamcaller.txt`
- `dreamcaller_prior.md`
- `dreamcaller_pool.md`
- `reference_set.jsonl`
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
- `final_explanations.jsonl`

Normalize the user export once up front to `$RUN_DIR/cards.jsonl`, and write the exact
dreamcaller text to `$RUN_DIR/dreamcaller.txt`.

### Phase 1: Model the Dreamcaller Prior

Before looking at the whole pool, write `$RUN_DIR/dreamcaller_prior.md` with:
- a short **rules interpretation checkpoint** stating:
  - what game event or state the dreamcaller cares about
  - what object, resource, trigger, or permission it creates or modifies
  - what downstream rules consequences follow from that, including timing, zones, board pressure,
    targeting, triggerability, or once-per-turn constraints when relevant
- the dreamcaller's direct hooks
- what the deck is trying to assemble
- what recurring resource or pressure the dreamcaller naturally creates
- what classes of cards it actively wants
- what classes it merely tolerates
- what kinds of generic premiums should stay high anyway
- a **provisional** openness guess and why

Do **not** lock the final openness label yet. The final `narrow / medium / open` call must be
revisited after a first pass over the actual pool.

### Phase 2: Split the Pool and Build a Calibration Reference Set

Use the helper scripts:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/split_jsonl.py \
  "$RUN_DIR/cards.jsonl" "$RUN_DIR/chunks" --chunk-size 60

python3 .llms/skills/dreamcaller-rank/scripts/build_reference_set.py \
  "$RUN_DIR/cards.jsonl" "$RUN_DIR/reference_set.jsonl" --count 24
```

`manifest.json` is the source of truth for chunk coverage. `reference_set.jsonl` is a small,
deterministic calibration sample shared by every scoring subagent.

### Scoring Contract

All intermediate ranking files must use:
- `score`: finite `0-100`
- `tie_break`: integer `-2` to `2`
- `rendered_text`: exact copy from the source row
- `note`: short rationale when the phase requires it

Use `tie_break` only for close calls:
- `2`: unusually clean dreamcaller fit
- `1`: meaningful fit edge
- `0`: neutral
- `-1`: mild anti-synergy
- `-2`: strong anti-synergy

### Phase 3: First-Pass Chunk Scoring

Spawn subagents over disjoint chunk files. Every stage-1 subagent must read:
- `docs/battle_rules/battle_rules.md`
- `$RUN_DIR/dreamcaller_prior.md`
- `$RUN_DIR/reference_set.jsonl`
- its assigned chunk file

Dispatch policy:
- if the number of chunks exceeds available live subagent slots, run in waves
- keep a queue of remaining chunks and backfill newly free slots as workers finish
- close completed workers promptly once their outputs are validated so later chunks can start
- do not wait for every worker in a wave to finish before launching the next chunk if a slot is free

The reference set is calibration-only. Read it first, mentally place those cards on the
`0-100` scale, then score the chunk on that same scale. Do **not** copy reference-set rows into
chunk output.

Each row in `$RUN_DIR/stage1/chunk-XXX.jsonl` must include:

```json
{
  "uuid": "string",
  "score": 78,
  "tie_break": 1,
  "rendered_text": "string",
  "role": "direct_payoff",
  "note": "short internal note"
}
```

Validate every chunk before using it:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/chunks/chunk-001.jsonl" \
  "$RUN_DIR/stage1/chunk-001.jsonl" \
  --exact \
  --require-tie-break \
  --require-fields role note
```

Do not proceed until all stage-1 chunks validate.

Prompt requirements for stage-1 subagents:
- score only the assigned chunk
- preserve `rendered_text` exactly
- use raw power only as a tie-breaker, except for true bombs
- assume your generic power-level read may be wrong; prefer cards with clearer shell purpose
- prefer dreamcaller-fit, infrastructure, and resource-conversion arguments over rate arguments
- close calls break toward fit
- ignore legacy `tide` and `rarity`
- do not read tides, resonance, rarity, or archetype files

### Phase 4: Merge Stage 1 and Write the Pool-Aware Dreamcaller Model

- this phase begins only after every stage-1 chunk has validated successfully
- do not inspect or reason from `stage1-merged.csv` until the merge command has completed successfully

Merge stage 1:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  "$RUN_DIR"/stage1/*.jsonl \
  --output "$RUN_DIR/stage1-merged.csv" \
  --expected-jsonl "$RUN_DIR/cards.jsonl"
```

Then write `$RUN_DIR/dreamcaller_pool.md`. This is the **final** dreamcaller model and must
include:
- the direct hooks and plan
- what recurring resources, bodies, timing windows, or pressures the dreamcaller actually creates
- what the pool actually supports well
- what the pool supports weakly or sparsely
- whether generic premiums are dense enough to resist synergy pulls
- the final `narrow / medium / open` label
- one short paragraph explaining why that label is correct **for this pool**

The openness label must be based on both dreamcaller text and pool context, not dreamcaller
text alone.

Before moving to anchors, do a short self-audit against `stage1-merged.csv`:
- Are the top cards explainable mainly by dreamcaller fit rather than generic rate?
- Are there cards creating or converting the dreamcaller's recurring resource that seem too low?
- Are there generic rate cards floating high without a clear shell job?
- Are you leaning on a power-level guess where a fit-based explanation would be more reliable?
If the answer looks wrong, revisit the pool model before continuing.

### Phase 5: Extract Anchors

Write anchors to `$RUN_DIR/anchors.json` and `$RUN_DIR/anchors.md`.

Use up to three anchor groups:
- `bombs_or_premiums`
- `direct_payoffs`
- `infrastructure`
- `resource_converters`

Each group may be small or empty. Do **not** pad a bucket just to hit a quota.

Each anchor entry should include:

```json
{
  "uuid": "string",
  "score": 88,
  "rendered_text": "string",
  "anchor_type": "direct_payoffs",
  "reason": "why this card moves neighboring cards",
  "wants": "what kinds of cards it lifts",
  "confidence": "high | medium"
}
```

Anchor rules:
- a strong card is not automatically an anchor
- every anchor needs a concrete `reason` and concrete `wants`
- infrastructure must do real enabling work, not just generic adjacency
- resource converters must explain what dreamcaller-created resource they cash in
- if a card would not plausibly move any neighbor, it is not an anchor

### Phase 6: Second-Pass Refinement

Run a second pass by chunk. Every stage-2 subagent must read:
- `docs/battle_rules/battle_rules.md`
- `$RUN_DIR/dreamcaller_pool.md`
- `$RUN_DIR/anchors.md`
- `$RUN_DIR/reference_set.jsonl`
- its assigned chunk file

Dispatch policy:
- use the same wave-based queueing approach as stage 1 when chunk count exceeds available live subagent slots
- close completed workers promptly after validation so remaining chunks can start

Use this pass to fix under-ranked glue and infrastructure without talking yourself into
recursive fake synergy.

Validate every refined chunk:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/chunks/chunk-001.jsonl" \
  "$RUN_DIR/stage2/chunk-001.jsonl" \
  --exact \
  --require-tie-break \
  --require-fields note
```

Stage-2 prompt requirements:
- keep scores on the same calibrated scale as the reference set
- let the pool-aware openness label matter
- preserve `rendered_text` exactly
- emit only the cards in the assigned chunk
- use raw power only to break close calls
- assume your power-level reads are lower-confidence than your fit reads
- include `note`, a short explanation of the card's final-local case in this chunk

Each row in `$RUN_DIR/stage2/chunk-XXX.jsonl` should therefore include:

```json
{
  "uuid": "string",
  "score": 81,
  "tie_break": 1,
  "rendered_text": "string",
  "note": "anchor-backed figment infrastructure"
}
```

### Phase 7: Merge Stage 2

- this phase begins only after every stage-2 chunk has validated successfully
- do not inspect or reason from `stage2-merged.csv` until the merge command has completed successfully

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  "$RUN_DIR"/stage2/*.jsonl \
  --output "$RUN_DIR/stage2-merged.csv" \
  --expected-jsonl "$RUN_DIR/cards.jsonl"
```

If this merge fails because cards are missing, duplicated, or unexpected, stop, fix the bad
chunks, revalidate them, and rerun the merge. A partial merge is invalid.

### Phase 8: Reconcile High-Value Windows

Use `stage2-merged.csv` as the provisional global order. Reconcile only the windows where local
ordering matters most:
- top of pool
- large tie bands
- boundary clusters

Build windows deterministically in this order:

1. Always create one `top_of_pool` window covering ranks `1-25`.
2. Starting at rank `26`, scan downward through `stage2-merged.csv` by contiguous equal-score
   bands. For each untouched score band with at least `8` cards and whose starting rank is at or
   before `150`, create one `large_tie_band` window covering that entire band.
3. After steps 1-2, if fewer than `4` total windows exist, add `boundary_cluster` windows to fill
   the gap. Each boundary window should cover `18` cards: `9` cards above and `9` cards below the
   first not-yet-covered score drop of at least `1.0`, scanning downward from the top.
4. Windows must remain disjoint. If a candidate band or boundary overlaps an existing window,
   skip it and continue scanning.
5. Stop once you have created `4` total windows. Fewer is acceptable if no valid disjoint windows
   remain.

Window rules:
- windows must be **disjoint**
- no UUID may appear in more than one reconciliation window
- window input rows must carry the current `score` and `tie_break` from `stage2-merged.csv`
- the subagent may adjust score by at most the metadata cap
- the subagent may emit only the UUIDs in its own input file

Each metadata file should include:

```json
{
  "window_id": "window-001",
  "reason": "top_of_pool | large_tie_band | boundary_cluster",
  "start_rank": 1,
  "end_rank": 25,
  "max_score_adjustment": 4
}
```

Validate every window output:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/windows/input/window-001.jsonl" \
  "$RUN_DIR/windows/output/window-001.jsonl" \
  --exact \
  --require-tie-break \
  --require-fields note \
  --max-score-adjustment 4
```

Use the actual cap from that window's metadata, not a hard-coded number, when you run the
validator.

Before final merge, prove the windows are disjoint:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_unique_uuids.py \
  "$RUN_DIR"/windows/output/*.jsonl
```

Reconciliation prompt requirements:
- keep the comparison local
- do not reinvent the whole ranking
- preserve `rendered_text` exactly
- respect the score-adjustment cap
- treat raw power as a tie-breaker, not the main argument
- if unsure, trust the cleaner fit argument over the stronger generic-rate guess
- include `note`, a short local-order reason such as `raw power correction`, `cleaner abandon payoff`,
  or `more replaceable than neighbors`
- do not use legacy tide, rarity, resonance, or archetype reasoning

### Phase 9: Deterministic Final Merge

- do not run the final merge until every reconciliation window has validated successfully and
  `validate_unique_uuids.py` has passed

Pass files in coarse-to-fine order so later stages override earlier stages:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  "$RUN_DIR"/stage1/*.jsonl \
  "$RUN_DIR"/stage2/*.jsonl \
  "$RUN_DIR"/windows/output/*.jsonl \
  --output "$RUN_DIR/final.csv" \
  --expected-jsonl "$RUN_DIR/cards.jsonl"
```

The merge helper validates score shape, preserves exact `rendered_text`, sorts by
`score desc`, then `tie_break desc`, then `uuid`, and emits final `uuid,score,rendered_text`.
Do not return a final file unless this merge succeeds.

After the final merge, write `$RUN_DIR/final_explanations.jsonl`. One row per UUID:

```json
{
  "uuid": "string",
  "final_score": 82.4,
  "final_tie_break": 1,
  "stage1_note": "short note",
  "stage2_note": "short note",
  "window_note": "short note or null",
  "summary": "one-sentence final why"
}
```
`summary` should explain why the card landed where it did, usually in terms of raw power,
dreamcaller fit, infrastructure value, replaceability, or anti-synergy. For cards not
touched by a reconciliation window, `window_note` may be `null`.

Then write the user-facing ranking file under `notes/` unless the user asked for another
destination path:

```bash
NOTES_OUTPUT="notes/dreamcaller_rank_$(date +%Y%m%d-%H%M%S).txt"
cp "$RUN_DIR/final.csv" "$NOTES_OUTPUT"
```

Return only a short pointer to the written file unless the user explicitly asks for inline output.

## Practical Notes

- Use a unique run directory under `/tmp` unless the user requests a repo path.
- If subagents are unavailable, run the same phases locally in sequence.
- Prefer file-first delivery for large rankings. The default user-facing product is a `notes/dreamcaller_rank_<timestamp>.txt` file copied from `$RUN_DIR/final.csv`.
- If the user asks why a card ranked where it did, answer from `final_explanations.jsonl` first.
- Use nearby cards in `final.csv` for pairwise comparison.
- Wait for merge commands to succeed before running dependent inspection commands or downstream logic.
- Treat any use of prior tide, rarity, resonance, or archetype knowledge as contamination.
- If a validator fails, fix that stage before continuing; do not “eyeball past” it.

## Failure Modes

- **Too literal:** only direct subtype matches rise.
- **Too generic:** the dreamcaller barely changes the ranking.
- **Bad calibration:** different chunk subagents use different score temperatures.
- **Premature openness:** the openness label is chosen before the pool is understood.
- **Recursive hand-waving:** "good with good cards" becomes an infinite reason generator.
- **Anchor inflation:** the workflow invents anchors to satisfy a quota.
- **Shared temp contamination:** runs reuse stale files.
- **Missing-card merge:** a stage drops UUIDs and the workflow keeps going anyway.
- **Window overlap:** the same UUID appears in two reconciliation windows.
- **Score-cap drift:** a window rewrites scores beyond its allowed adjustment.
- **Dropped rules text:** final rows lose the original `rendered_text`.
- **Legacy leakage:** tide or rarity fields sneak back into the reasoning.

## Minimal Success Criteria

The run succeeds only if:
- every input UUID appears exactly once in the final output
- the final output is strictly ordered `best -> worst`
- every final row includes exact source `rendered_text`
- stage outputs obey the score and tie-break contract
- the run preserves enough explanation data to answer per-card and pairwise `why` questions later
- obvious direct-fit cards rise appropriately
- some non-obvious infrastructure cards rise for defensible reasons
- generic bombs still stay near the top when warranted
- the final reasoning is not contaminated by tide or rarity heuristics
