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

Normalize the user export once up front to `$RUN_DIR/cards.jsonl`, and write the exact
dreamcaller text to `$RUN_DIR/dreamcaller.txt`.

### Phase 1: Model the Dreamcaller Prior

Before looking at the whole pool, write `$RUN_DIR/dreamcaller_prior.md` with:
- the dreamcaller's direct hooks
- what the deck is trying to assemble
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
- generic bombs stay high
- close calls break toward fit
- ignore legacy `tide` and `rarity`
- do not read tides, resonance, rarity, or archetype files

### Phase 4: Merge Stage 1 and Write the Pool-Aware Dreamcaller Model

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
- what the pool actually supports well
- what the pool supports weakly or sparsely
- whether generic premiums are dense enough to resist synergy pulls
- the final `narrow / medium / open` label
- one short paragraph explaining why that label is correct **for this pool**

The openness label must be based on both dreamcaller text and pool context, not dreamcaller
text alone.

### Phase 5: Extract Anchors

Write anchors to `$RUN_DIR/anchors.json` and `$RUN_DIR/anchors.md`.

Use up to three anchor groups:
- `bombs_or_premiums`
- `direct_payoffs`
- `infrastructure`

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
- if a card would not plausibly move any neighbor, it is not an anchor

### Phase 6: Second-Pass Refinement

Run a second pass by chunk. Every stage-2 subagent must read:
- `docs/battle_rules/battle_rules.md`
- `$RUN_DIR/dreamcaller_pool.md`
- `$RUN_DIR/anchors.md`
- `$RUN_DIR/reference_set.jsonl`
- its assigned chunk file

Use this pass to fix under-ranked glue and infrastructure without talking yourself into
recursive fake synergy.

Validate every refined chunk:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/validate_rankings.py \
  "$RUN_DIR/chunks/chunk-001.jsonl" \
  "$RUN_DIR/stage2/chunk-001.jsonl" \
  --exact \
  --require-tie-break
```

Stage-2 prompt requirements:
- keep scores on the same calibrated scale as the reference set
- let the pool-aware openness label matter
- preserve `rendered_text` exactly
- emit only the cards in the assigned chunk

### Phase 7: Merge Stage 2

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
- do not use legacy tide, rarity, resonance, or archetype reasoning

### Phase 9: Deterministic Final Merge

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

## Practical Notes

- Use a unique run directory under `/tmp` unless the user requests a repo path.
- If subagents are unavailable, run the same phases locally in sequence.
- The product is the ranking, not a long essay.
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
- obvious direct-fit cards rise appropriately
- some non-obvious infrastructure cards rise for defensible reasons
- generic bombs still stay near the top when warranted
- the final reasoning is not contaminated by tide or rarity heuristics
