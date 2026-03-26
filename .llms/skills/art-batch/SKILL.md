---
name: art-batch
description: Orchestrate batch art-to-card matching across ~1500 images. Runs all images against the full card pool, writing results to art-assigned.toml.
---

# Art Batch Matching Orchestrator

You are an orchestrator that matches art to cards. You launch **one subagent per image**,
running up to 5 images concurrently in each batch.

**All subagents MUST be launched with `model: "opus"`.**

## Setup

1. Build the image list:
   ```bash
   python3 .llms/skills/art-batch/build-image-list.py
   ```

2. Create the results directory:
   ```bash
   mkdir -p /tmp/art-batch-results
   ```

## Batch Loop

Repeat until done:

1. Get the next batch of unprocessed images:
   ```bash
   python3 .llms/skills/art-batch/next-batch.py 5
   ```
   This prints up to 5 image IDs (one per line), or `DONE` if none remain.
   If `DONE`, proceed to the Join step.

2. Launch **one background Agent per image ID** in a single message (all in parallel).
   Use `model: "opus"` and `run_in_background: true` for each. Give each agent the
   Image Agent Prompt below with its IMAGE_ID filled in.

3. As each agent completes, print a one-line status:
   - Match: `image {ID} -> {card name} ({tide}, {cost})`
   - Skip: `image {ID} -> SKIP ({reason})`
   - Fail: `image {ID} -> FAIL ({reason})`

4. Once all agents in the batch have completed, go back to step 1.

### Image Agent Prompt

```
You are matching one piece of art to a card from the Dreamtides card pool. Run with
ultrathink.

Your image ID is {IMAGE_ID}.

Read the art-match skill at `.llms/skills/art-match/SKILL.md` and follow it completely.

## Overused Name Words

During Phase 4 (naming), check your proposed name before finalizing:
```bash
python3 .llms/skills/art-batch/overused-words.py "Proposed Card Name"
```
If it prints FAIL, one or more words are overused (3+ prior uses) — pick a different name
and check again. Only finalize a name that prints PASS.

## Final Step: Write Result

After selecting a winner and assigning a name/subtype (Phase 4 of the skill), find the
real card matching your selected anonymized rules text:

```bash
grep -n 'rendered-text' rules_engine/tabula/rendered-cards.toml | grep '<EXACT RULES TEXT>'
```

Use the line number to read the surrounding [[cards]] block and extract the real card's
id, tide, tide-cost, energy-cost, card-type, rarity, is-fast, and spark.

Then write this TOML block to `/tmp/art-batch-results/{IMAGE_ID}.toml`:

```toml
[[cards]]
name = "<YOUR new card name from Phase 4>"
id = "<real card UUID>"
tide = "<tide from real card>"
tide-cost = <from real card>
rendered-text = "<rules text from real card>"
energy-cost = <from real card>
card-type = "<from real card>"
subtype = "<YOUR subtype from Phase 4>"
rarity = "<from real card>"
is-fast = <from real card>
spark = <from real card, or "" for events>
image-number = {IMAGE_ID}
art-owned = false
card-number = 0
```

If the art is landscape or abstract, do NOT write a result file. Instead, record the
skip by appending the image ID to `/tmp/art-batch-skips.txt`:
```bash
echo "{IMAGE_ID}" >> /tmp/art-batch-skips.txt
```
Then print "SKIP: [reason]" and stop.

Print ONLY the card name, tide, and cost as your final output.
```

## Join Results

After all batches are done:

```bash
python3 .llms/skills/art-batch/join-results.py
```

This appends all per-image result files from `/tmp/art-batch-results/` into
`rules_engine/tabula/art-assigned.toml`.

Print final stats:

```
=== Batch Complete ===
Total results joined: [count]
Skips: [count]
```

## Gravity Well Protection

Two layers prevent any single rules text from being overrepresented:

1. **Soft gate (check-match-count.py):** After selecting a winner, the subagent runs
   `check-match-count.py` with the exact rules text. WARN at 2 matches forces
   reconsideration; FAIL at 3+ forces picking a different card.

2. **Hard cap (pool-filter.py in art-match):** Cards matched 3+ times are excluded from
   pool output entirely. The pool also annotates cards with ⚠2× or 1× prefixes so the
   agent can see popularity at browse time.

Both scripts use `match_counts.py` to count assignments across **all** sources
(art-assigned.toml + /tmp/art-batch-results/*.toml) with proper TOML parsing.

## Context Preservation

1. **Keep output minimal.** One line per completed agent.
2. **Do not accumulate results.** Subagents write to individual files.
3. **Do not summarize or restate batch results.** Just the one-line status per image.
