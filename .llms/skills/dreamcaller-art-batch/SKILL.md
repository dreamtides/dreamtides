---
name: dreamcaller-art-batch
description: Orchestrate batch dreamcaller-identity assignment across all dreamcaller portrait images in ~/Documents/synty/dreamcallers. Runs one subagent per image, each producing a name + ability match via the dreamcaller-art-match skill, then joins results into notes/dreamcaller-assignments.md.
---

# Dreamcaller Art Batch Orchestrator

You are an orchestrator that assigns dreamcaller identities to every portrait
image in `~/Documents/synty/dreamcallers/` using a **rolling parallelism**
model. You maintain a target number of concurrent subagents (default 6),
launching replacements immediately as each agent completes — never waiting for
an entire batch to finish.

The candidate images are PNGs in `~/Documents/synty/dreamcallers/` (~118
portraits). The ability pool is `notes/dreamcallers.md`. Each subagent runs the
single-image `dreamcaller-art-match` skill end-to-end.

**All subagents MUST be launched with `model: "sonnet"`** and
`run_in_background: true`.

**IMPORTANT — name uniqueness is global.** The dreamcaller-art-match skill
enforces total naming-word uniqueness via
`/tmp/dreamcaller_art_match_registry.json`. Because that registry is the only
serialization point between concurrent subagents, **keep parallelism modest
(≤8)** so subagents don't race on the same fresh words. Each subagent already
re-checks and re-claims atomically (under `fcntl.flock`), so collisions
self-heal — but lower parallelism produces better naming variety, and naming
words become scarce late in a 118-image run.

**IMPORTANT — ability hard cap.** The registry caps each ability at 5 uses.
With 118 portraits and 32 abilities (~3.7 avg), several abilities WILL
saturate. When that happens `claim` fails with `ability has reached the hard
cap`. The subagent prompt below tells subagents to handle this by choosing a
different ability, not a different name.

## Setup

1. Build the image list:
   ```bash
   python3 .llms/skills/dreamcaller-art-batch/scripts/build-image-list.py
   ```

2. Create the results directory and clear stale inflight state:
   ```bash
   mkdir -p /tmp/dreamcaller-batch-results
   rm -f /tmp/dreamcaller-batch-inflight.txt /tmp/dreamcaller-batch-skips.txt
   ```

3. **Decide whether to reset the name registry.** If you want naming variety
   only against this batch, clear it:
   ```bash
   rm -f /tmp/dreamcaller_art_match_registry.json
   ```
   If you want to preserve names already claimed in prior runs, leave it.

## Rolling Launch

### Initial Fill

Get the first N image IDs (default 6):
```bash
python3 .llms/skills/dreamcaller-art-batch/scripts/next-batch.py 6
```
Prints up to 6 image IDs (one per line), `WAITING`, or `DONE`.

Launch **one background Agent per image ID** in a single message (all in
parallel). Use `model: "sonnet"` and `run_in_background: true` for each. Give
each agent the Image Agent Prompt below with its IMAGE_ID filled in.

### Continuous Refill

After the initial fill, **wait for notifications**. Do NOT poll, sleep-loop, or
check on agents. You will be automatically notified when each agent completes.

When notified that one or more agents have completed:

1. Print one short line per completed agent (the dreamcaller name, or SKIP
   reason). Nothing else.
2. Immediately request replacement IDs for however many agents just finished:
   ```bash
   python3 .llms/skills/dreamcaller-art-batch/scripts/next-batch.py <COUNT>
   ```
3. If this returns image IDs, launch new background agents for each in a single
   message. If it returns `WAITING`, just keep waiting. If `DONE`, stop
   launching.
4. Continue waiting for the next notification.

### Failed Subagents

If a notification reports an agent failed or returned with no result file
written and no entry appended to `/tmp/dreamcaller-batch-skips.txt`, its image
ID is wedged in the inflight file and `next-batch.py` will not re-issue it.
Release the slot manually:

```bash
sed -i '' "/^{IMAGE_ID}$/d" /tmp/dreamcaller-batch-inflight.txt
```

Then request a replacement via `next-batch.py 1` — the released ID will be
returned again and you can re-launch it. Print one line: `RETRY: {IMAGE_ID}`.

### Drain & Finish

Once `next-batch.py` returns `DONE`, stop launching new agents. Continue
receiving notifications for the remaining in-flight agents, printing one line
each, until all have completed. Then proceed to the Join step.

### Image Agent Prompt

```
You are assigning a dreamcaller identity to one portrait image. Run with
ultrathink.

Your image path is /Users/dthurn/Documents/synty/dreamcallers/{IMAGE_ID}.png.

Read the dreamcaller-art-match skill at
`.llms/skills/dreamcaller-art-match/SKILL.md` and follow it completely. Use
that single image as your input. Open the image with the Read tool — do not
match from the filename alone.

The skill will:
- read `notes/dreamcallers.md`
- check and claim a globally unique name via the
  `/tmp/dreamcaller_art_match_registry.json` registry script

Handle `claim` failures based on the conflict reason printed by the script:

- **`conflict: reused words: ...`** or **`conflict: full name already claimed`**
  — another subagent grabbed an overlapping word first. Generate a new name
  with different words and retry. The chosen ability does NOT need to change.
- **`conflict: ability has reached the hard cap and is banned`** — the chosen
  ability is saturated across the batch. Return to the skill's step 3, pick a
  *different ability* from `notes/dreamcallers.md`, then build a new identity
  around it. Do not try to rename around this — only a new ability resolves it.
- **`warning: ability has reached the soft cap`** is NOT a failure; claim
  succeeds. You may proceed, but prefer a less-used ability if one fits the
  art comparably well.

Never skip the registry — global uniqueness is the entire point.

## Final Step: Write Result

After the skill produces its final 5-section output, write it verbatim to
`/tmp/dreamcaller-batch-results/{IMAGE_ID}.md` in this format:

```markdown
# {IMAGE_ID}

![art](/Users/dthurn/Documents/synty/dreamcallers/{IMAGE_ID}.png)

## Chosen Ability
<exact ability text from notes/dreamcallers.md>

## Dreamcaller Name
<Proper Name, Title>

## Art Reading
<one paragraph>

## Narrative Match
<one paragraph>

## Title Justification
<one paragraph>
```

If the art fails the suitability check (landscape, abstract, crowd, non-person
creature that cannot read as a dream-person), do NOT write a result file.
Instead append the image ID to `/tmp/dreamcaller-batch-skips.txt`:
```bash
echo "{IMAGE_ID}" >> /tmp/dreamcaller-batch-skips.txt
```
Then print `SKIP: <reason>` and stop.

Print ONLY the final dreamcaller name (or SKIP line) as your output.
```

## Join Results

After all agents have drained:

```bash
python3 .llms/skills/dreamcaller-art-batch/scripts/join-results.py
```

This concatenates all per-image result files from
`/tmp/dreamcaller-batch-results/` into `notes/dreamcaller-assignments.md`,
sorted by image ID.

Print final stats:

```
=== Dreamcaller Batch Complete ===
Total assigned: [count]
Skips: [count]
```

## Context Preservation — CRITICAL

You will process ~118 images. Your context window may overflow if undisciplined.

1. **Never print more than one line per completed agent.** No commentary.
2. **Never print summaries, progress updates, or status messages between refills.**
3. **Do not accumulate results.** Subagents write to individual files.
4. **Do not restate or reflect on previous completions.**
5. **Do not poll or sleep-loop for agent completion.** Wait for notifications.
6. **Immediately launch replacements** after each completion notification.
7. **Batch your refill launches.** If multiple agents complete in quick
   succession before you respond, request all replacement IDs at once and
   launch them in a single message.
