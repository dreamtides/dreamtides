---
name: card-design-batch
description: Batch card design from a directory of art images. Launches parallel card-design subagents (rolling window), each producing 5 candidate designs. Joins results into one TOML file for user selection. Triggers on batch card design, batch design, design cards from directory, card design batch.
---

# Card Design Batch Orchestrator

You are an orchestrator that runs the card-design workflow in parallel across a
directory of art images. You maintain a rolling window of concurrent subagents
(default 10), launching replacements as each completes.

## Inputs

The user must provide:
1. **Image directory** — path to a directory containing image files
2. **Card type** — the expected card type for all images (e.g. "Character",
   "Event", "Dreamwell"). Defaults to "Character" if not specified.
3. **Parallelism** (optional) — number of concurrent subagents (default 10)

## Setup

1. Build the image list:
   ```bash
   python3 .llms/skills/card-design-batch/build-image-list.py <IMAGES_DIR>
   ```

2. Create the results directory and clear stale inflight state:
   ```bash
   mkdir -p /tmp/card-design-batch-results
   rm -f /tmp/card-design-batch-inflight.txt
   ```

## Rolling Launch

### Initial Fill

Get the first N image IDs (where N is the parallelism target):
```bash
python3 .llms/skills/card-design-batch/next-batch.py <N>
```
This prints up to N image IDs (one per line), or `DONE` if none remain.

For each image ID, look up the full path from `/tmp/card-design-batch-paths.txt`
(format: `stem\tpath`).

Launch **one background Agent per image ID** in a single message (all in
parallel). Use `run_in_background: true` for each. Give each agent the
Subagent Prompt below with its variables filled in.

### Continuous Refill

After the initial fill, **wait for notifications**. Do NOT poll, sleep-loop, or
check on agents.

When notified that one or more agents have completed:

1. Print one short line per completed agent (image stem + card names summary).
2. Request replacement image IDs:
   ```bash
   python3 .llms/skills/card-design-batch/next-batch.py <COUNT>
   ```
   Output meanings:
   - **Image IDs** (one per line): launch new background agents for each.
   - **`WAITING`**: all remaining images are in-flight. Wait for next notification.
   - **`DONE`**: every image is processed. Let remaining in-flight agents drain.
3. If IDs returned, launch new background agents in a single message.
4. Continue waiting for the next notification.

### Drain & Finish

Once `next-batch.py` returns `DONE`, stop launching. Continue receiving
notifications until all in-flight agents complete. Then proceed to the Join step.

### Subagent Prompt

```
You are designing Dreamtides cards from art. Run with ultrathink.

Your image file is at: {IMAGE_PATH}
Your image stem (ID) is: {IMAGE_STEM}
The expected card type is: {CARD_TYPE}

Read the card-design skill at `.llms/skills/card-design/SKILL.md` and follow
Phases 1 through 6 (design generation only — no user interaction).

**Important overrides:**
- Skip Phase 1 (art classification) — the card type is {CARD_TYPE}.
- In Phase 5, generate exactly 5 complete card designs.
- In Phase 6, do NOT present designs to the user or wait for input. Instead,
  write all 5 designs to a TOML file as described below.
- Do NOT proceed to Phases 7-8.

## Overused Name Words

Before finalizing each card name, check it:
```bash
python3 .llms/skills/art-batch/overused-words.py "Proposed Card Name"
```
If it prints FAIL, pick a different name and check again.

## Output: Write 5 Designs to TOML

After generating 5 designs, generate a UUID for each:
```bash
for i in 1 2 3 4 5; do uuidgen | tr '[:upper:]' '[:lower:]'; done
```

Find the highest existing `card-number` in
`client/Assets/StreamingAssets/Tabula/rendered-cards.toml` and use the next
5 consecutive numbers.

Extract the Shutterstock image number from the filename (the numeric portion).

Write all 5 designs to `/tmp/card-design-batch-results/{IMAGE_STEM}.toml`
using this exact format:

```toml
[[cards]]
keep = false
name = "Design 1 Name"
id = "uuid-1"
tide = "Tide"
tide-cost = 1
rendered-text = "Rules text here."
energy-cost = 3
card-type = "{CARD_TYPE}"
subtype = "Warrior"
rarity = "Rare"
is-fast = false
spark = 2
art-owned = false
card-number = 667
image-number = 1234567890

[[cards]]
keep = false
name = "Design 2 Name"
...
```

Field notes:
- `keep = false` MUST be the first field in every entry.
- `rendered-text`: Use rendered symbols (●, ▸, ↯, ✪).
- `spark`: Integer for characters, `""` for events.
- `subtype`: Appropriate subtype for characters, `""` for events.
- `art-owned`: Always `false`.
- All 5 entries share the same `image-number`.

After writing, validate:
```bash
python3 .llms/skills/card-design-batch/validate-result.py /tmp/card-design-batch-results/{IMAGE_STEM}.toml
```
If FAIL, fix the file before finishing.

Print a one-line summary of your 5 designs: the card names and tides.
```

## Join Results

After all agents have drained:

```bash
python3 .llms/skills/card-design-batch/join-results.py
```

This writes all results to `/tmp/card-design-batch-output.toml`.

Tell the user the output file path and the total count. Instruct them to review
the file and set `keep = true` on their chosen design for each image (one per
image, or skip images they don't want).

```
=== Batch Complete ===
Total images processed: [count]
Total designs generated: [count * 5]
Output file: /tmp/card-design-batch-output.toml

Review the file and set keep = true on your chosen design for each image.
```

## Context Preservation — CRITICAL

You may process many images. Keep context lean:

1. **Never print more than one line per completed agent.**
2. **Never print summaries or progress updates between refills.**
3. **Do not accumulate results.** Subagents write to individual files.
4. **Do not restate or reflect on previous completions.**
5. **Do not poll or sleep-loop.** Wait for notifications.
6. **Immediately launch replacements** after each completion.
7. **Batch refill launches** when multiple agents complete together.
