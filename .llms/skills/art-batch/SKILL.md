---
name: art-batch
description: Orchestrate batch art-to-card matching across 2200 images. Runs all images against the full card pool, writing results to art-assigned.toml.
---

# Art Batch Matching Orchestrator

You are an orchestrator that runs sequential subagent art-matching sessions. Each subagent
receives ONE image and matches it to the best card from the pool.

**All subagents MUST be launched with `model: "opus"`.**

## Setup

Run the image list builder:
```bash
python3 .llms/skills/art-batch/build-image-list.py
```

## Execution Loop

Each iteration of the loop:

1. Get the next unprocessed image:
   ```bash
   python3 .llms/skills/art-batch/next-image.py
   ```
   This prints `INDEX TOTAL IMAGE_ID` or `DONE`. If `DONE`, exit the loop.

2. Launch ONE foreground Agent with `model: "opus"` and the prompt below. Wait for it to
   complete before launching the next. Do NOT run in parallel.

3. Print a one-line status:
   - Match: `[N/TOTAL] image XXXXXXX -> [card name] ([tide], [cost])`
   - Skip: `[N/TOTAL] image XXXXXXX -> SKIP ([reason])`
   - Fail: `[N/TOTAL] image XXXXXXX -> FAIL ([reason])`

Do not retry failures. Every 50 images print:
`--- Checkpoint: {N}/{TOTAL}, {skips} skips, {failures} failures ---`

### Subagent Prompt

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

## Final Step: Write to art-assigned.toml

After selecting a winner and assigning a name/subtype (Phase 4 of the skill), find the
real card matching your selected anonymized rules text:

```bash
grep -n 'rendered-text' rules_engine/tabula/rendered-cards.toml | grep '<EXACT RULES TEXT>'
```

Use the line number to read the surrounding [[cards]] block and extract the real card's
id, tide, tide-cost, energy-cost, card-type, rarity, is-fast, and spark.

Then append this TOML block to `rules_engine/tabula/art-assigned.toml`:

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

If the art is landscape or abstract, do NOT write anything. Print "SKIP: [reason]" and stop.

Print ONLY the card name, tide, and cost as your final output.
```

## Oversaturation Protection

The pool filter automatically excludes cards that have been assigned to 5 or more
images. This prevents any single rules text from being overrepresented. If you see
"N cards hidden" in the pool filter output, those cards have hit the saturation limit
and are no longer available for matching.

## Context Preservation

This runs for a very long time across ~2200 agents. To preserve context:

1. **Keep output minimal.** One line per agent.
2. **Do not accumulate results.** Subagents write to the file.
3. **Track only:** current index, total, skip count, failure count.

## Completion

After all images, print:

```
=== Batch Complete ===
Images processed: [count]
Cards matched: [count]
Skips: [count]
Failures: [count]
```
