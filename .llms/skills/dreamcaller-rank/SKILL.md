---
name: dreamcaller-rank
description: Rank an anonymized card pool for how highly a drafter already committed to a given dreamcaller should pick each card. Uses staged subagent judgment, second-order synergy analysis, and deterministic UUID-based merge. Triggers on dreamcaller ranking, rank cards for a dreamcaller, dreamcaller pick order, card pick order for dreamcaller, or draft ranking for a dreamcaller.
---

# Dreamcaller Ranking

Rank every card in an anonymized pool for `pack 1 pick 1 after committing to this dreamcaller`.
The goal is a strict `best -> worst` ordering of the whole pool, not just a shortlist.

Use subagents when available. The task is intentionally staged: broad first-pass scoring,
anchor extraction, second-pass refinement, then deterministic merge into one final ranking.

Read `docs/battle_rules/battle_rules.md` before ranking. The agent and every ranking subagent
must ground its judgments in Dreamtides rules vocabulary and timing.

This skill is intentionally a fresh-perspective evaluation based only on the user-provided
dreamcaller text and anonymized card pool.

Do **not** read:
- `docs/tides/tides.md`
- `rules_engine/tabula/rendered-cards.toml`
- `rules_engine/tabula/cards.toml`
- `docs/resonance/resonance.md`
- any file whose purpose is to assign cards to archetypes, resonances, tides, or existing deck
  identities

Do **not** use any prior archetype labels, tide associations, or card-pool curation metadata as
evidence. Judge only from the given inputs plus `docs/battle_rules/battle_rules.md`.

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
  "rarity": "L | R | U | C | null",
  "is_fast": false,
  "rendered_text": "string"
}
```

Notes:
- `uuid` is required and is the only identity field used in the final output.
- Ignore anonymized placeholder names if present. They are not reliable identifiers.

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
5. **Openness**
   If the dreamcaller is broad or merely nudges the draft, lean harder on raw power.
6. **Anti-synergy**
   Does this card meaningfully push away from the dreamcaller's best path?

### Important judgment rules

- **Bomb override:** truly generic bombs stay near the top even if their synergy is modest.
- **No fake recursion loops:** do not keep boosting a card because it supports a card that
  supports another good card. Multi-hop synergy is real, but its weight decays quickly.
- **Stop at generic adjacency:** once the argument becomes "this is good with good cards,"
  the second-order chain has run out.
- **On-plan is good:** do not invent scarcity penalties just because a role is common.
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

### Phase 1: Model the Dreamcaller

Before ranking any cards, write a short internal analysis containing:

- the dreamcaller's direct hooks
- what the dreamcaller is trying to assemble
- what classes of cards it actively wants
- what classes of cards it merely tolerates
- what kinds of cards are generic premiums even if not especially synergistic
- whether the dreamcaller is **narrow**, **medium**, or **open**

Do not overfit to exact words. Infer the deck's desired play pattern.
Re-read `docs/battle_rules/battle_rules.md` if the dreamcaller depends on timing, board state,
Judgment, void, reclaim, fast cards, or subtype interactions.
Do not consult tides, resonance, archetype, or source-pool files to answer these questions.

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
  /path/to/cards.jsonl /tmp/dreamcaller-rank/chunks --chunk-size 60
```

This creates chunk files containing disjoint card sets. Never give two subagents the same
output file.

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
to surface anchor cards.

The `tie_break` field is internal. Use higher values for stronger dreamcaller fit when cards
are otherwise close.

#### First-pass prompt template

```text
You are ranking one chunk of an anonymized Dreamtides card pool for pack 1 pick 1 after
committing to this dreamcaller:

[dreamcaller text]

Before ranking, read docs/battle_rules/battle_rules.md.
Do not read docs/tides/tides.md, rendered-cards.toml, cards.toml, resonance docs, or any
existing archetype-assignment material.

Rank every card in [chunk path]. Use 0-100 draft-value scores.

Focus on:
- raw power
- direct dreamcaller fit
- infrastructure / second-order fit
- dependency

Rules:
- process every card exactly once
- output JSONL to [output path]
- include uuid, score, tie_break, rendered_text, role, and a short internal note
- generic bombs stay high
- close calls break toward dreamcaller fit
- do not use placeholder card names as identifiers
```

### Phase 4: Extract Global Anchors

Merge the first-pass outputs and identify small global anchor sets:

- top generic bombs / generic premiums
- top direct dreamcaller payoffs
- top infrastructure / enablers

Aim for roughly `8-20` cards in each anchor set.

This is the key move for non-obvious cards. For a card that is not an obvious fit, ask:

`Does this card make the anchor cards better, more reliable, more numerous, or easier to draft
into?`

If yes, lift it. If the support claim is too indirect or too generic, do not.

### Phase 5: Second-Pass Refinement

Run a second pass over the pool, again by chunk, but now include the anchor sets.

Use this pass to fix the common failure mode where only direct matches rise while enabling
cards stay too low.

#### Second-pass prompt template

```text
You are refining a chunk-level Dreamcaller draft ranking.

Dreamcaller:
[dreamcaller text]

Before refining, read docs/battle_rules/battle_rules.md.
Do not read docs/tides/tides.md, rendered-cards.toml, cards.toml, resonance docs, or any
existing archetype-assignment material.

Global anchors:
- bombs/premiums: [list]
- direct payoffs: [list]
- infrastructure: [list]

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
- if the dreamcaller is broad, fall back more toward raw power
```

### Phase 6: Reconcile Only the Close Clusters

Do not spend agent budget globally re-ranking the whole pool again.

Instead:
- merge the refined outputs
- sort provisionally
- identify close clusters near the top and around large score ties
- create overlapping windows from the provisional list

Window size guidance:
- top of pool: `20-30` cards
- middle/boundary clusters: `15-25` cards
- overlap adjacent windows by `5-10` cards

Ask subagents to reorder only inside the window and, if needed, slightly adjust scores or
`tie_break` values.

The point of the reconciliation pass is to stabilize local order, not to reinvent the whole
ranking.

### Phase 7: Deterministic Final Merge

Use the merge helper script. Pass files in coarse-to-fine order so later stages override
earlier ones:

```bash
python3 .llms/skills/dreamcaller-rank/scripts/merge_rankings.py \
  /tmp/dreamcaller-rank/stage1/*.jsonl \
  /tmp/dreamcaller-rank/stage2/*.jsonl \
  /tmp/dreamcaller-rank/windows/*.jsonl \
  --output /tmp/dreamcaller-rank/final.csv
```

The script:
- dedupes by UUID
- lets later files override earlier entries
- sorts by `score desc`, then `tie_break desc`, then `uuid`
- emits final `uuid,score,rendered_text`

## Practical Notes

- Keep intermediate artifacts in `/tmp/dreamcaller-rank-*` unless the user requests a repo
  path.
- If subagents are unavailable, run the same workflow locally in multiple passes.
- Do not waste effort on long written explanations. The product is the ranking.
- If the final ranking contains fewer cards than the input pool, treat that as a failure and
  fix it before returning.
- If the input format differs from JSONL, adapt once up front. Do not let every subagent
  invent its own parser.
- Preserve the exact `rendered_text` field from the card record in every intermediate ranking
  file so the final merge can emit it without re-reading the source pool.
- Treat any attempt to import prior archetype or tide knowledge as contamination of the
  evaluation and avoid it.

## Failure Modes

- **Too literal:** only subtype matches rise; glue and infrastructure stay too low.
- **Too generic:** the dreamcaller barely affects the ranking.
- **Recursive hand-waving:** "good with good cards" becomes an infinite reason generator.
- **No bomb override:** powerful generically great cards sink for no good reason.
- **No deterministic merge:** duplicate UUIDs or missing cards in the final output.
- **Dropped rules text:** final rows lose the original `rendered_text`.
- **Contaminated perspective:** reading tides, resonance, source TOML, or prior archetype
  assignments and then pretending the evaluation was fresh.
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
