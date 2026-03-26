---
name: art-batch
description: Orchestrate batch art-to-card matching across ~1500 images. Runs all images against the full card pool, writing results to art-assigned.toml.
---

# Art Batch Matching Orchestrator

You are an orchestrator that runs **parallel** art-matching across 5 lanes. Each lane
processes its share of images sequentially, but all 5 lanes run concurrently.

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

3. Partition images into 5 lanes:
   ```bash
   python3 .llms/skills/art-batch/partition-images.py 5
   ```
   This excludes already-processed images and distributes the rest across lanes.

## Launch Lanes

Launch **5 background Agent calls in a single message**, one per lane. Each lane agent
gets the Lane Agent Prompt below with its lane number filled in. Use `model: "opus"` and
`run_in_background: true` for all 5.

As each lane agent completes, print a summary of its results.

After all 5 lanes complete, run the join step.

### Lane Agent Prompt

```
You are lane {LANE_NUMBER} of a parallel art-batch matching run. You process images
sequentially from your lane file.

**All art-match subagents MUST be launched with `model: "opus"`.**

## Your Loop

Repeat until done:

1. Get your next image:
   ```bash
   python3 .llms/skills/art-batch/next-image.py --lane {LANE_NUMBER}
   ```
   This prints `INDEX TOTAL IMAGE_ID` or `DONE`. If `DONE`, print your final stats and stop.

2. Launch ONE foreground Agent with `model: "opus"` and this prompt (fill in IMAGE_ID):

   ---
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
   ---

3. Print a one-line status:
   - Match: `[Lane {LANE_NUMBER}] [{N}/{TOTAL}] image {ID} -> {card name} ({tide}, {cost})`
   - Skip: `[Lane {LANE_NUMBER}] [{N}/{TOTAL}] image {ID} -> SKIP ({reason})`
   - Fail: `[Lane {LANE_NUMBER}] [{N}/{TOTAL}] image {ID} -> FAIL ({reason})`

Do not retry failures. Every 25 images print:
`--- Lane {LANE_NUMBER} Checkpoint: {N}/{TOTAL}, {skips} skips, {failures} failures ---`

When done, print:
`=== Lane {LANE_NUMBER} Complete: {matched} matched, {skips} skips, {failures} failures ===`
```

## Join Results

After all 5 lanes finish:

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

2. **Hard cap (pool-filter.py):** Cards matched 3+ times are excluded from pool output
   entirely. The pool also annotates cards with ⚠2× or 1× prefixes so the agent can
   see popularity at browse time.

**Note:** With parallel lanes, gravity well checks only see results from art-assigned.toml
(prior runs), not from other lanes running concurrently. Some duplicate assignments are
expected and can be audited after joining.

## Context Preservation

Each lane runs for a long time across hundreds of agents. To preserve context:

1. **Keep output minimal.** One line per agent.
2. **Do not accumulate results.** Subagents write to individual files.
3. **Track only:** current index, total, skip count, failure count.
