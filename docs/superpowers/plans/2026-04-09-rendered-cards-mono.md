# Rendered Cards Mono Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create `rendered-cards-mono.toml` containing the original mono-tide
card assignments (matching `tides.md`), delete the stale
`rendered-cards-revised.toml`, and verify tide assignments are correct.

**Architecture:** Extract the `ff55c088` git version of `rendered-cards.toml`
(which has all 594 current cards with original mono-tide assignments and
`tide-cost` fields), save it as `rendered-cards-mono.toml`, delete the stale
revised file, and run sanity checks via subagents.

**Tech Stack:** Git, TOML files, bash

______________________________________________________________________

### Task 1: Create rendered-cards-mono.toml from git history

**Files:**

- Create: `rules_engine/tabula/rendered-cards-mono.toml` (via symlink at
  `client/Assets/StreamingAssets/Tabula/rendered-cards-mono.toml`)

- [ ] **Step 1: Extract the mono-tide version from git**

The commit `ff55c088` ("feat: add 10 starter-rarity cards to
rendered-cards.toml") is the last commit where `rendered-cards.toml` had the
original mono-tide assignments. It has all 594 cards that exist in the current
`rendered-cards.toml`, with `tide-cost` fields and the original
`tides.md`-aligned tide assignments. The only differences from the current file
are `tide` assignments (202 cards differ) and one `tide-cost` value.

```bash
git show ff55c088:client/Assets/StreamingAssets/Tabula/rendered-cards.toml > client/Assets/StreamingAssets/Tabula/rendered-cards-mono.toml
```

- [ ] **Step 2: Verify the file was created correctly**

```bash
# Should output 594
grep -c '^\[\[cards\]\]' rules_engine/tabula/rendered-cards-mono.toml
# Should output 596 (594 cards have tide-cost, 2 might not — but verify it's > 590)
grep -c 'tide-cost' rules_engine/tabula/rendered-cards-mono.toml
```

- [ ] **Step 3: Verify all current cards are present**

```bash
# Compare card names — should produce no output (identical sets)
grep '^name = ' rules_engine/tabula/rendered-cards.toml | sort > /tmp/current_names.txt
grep '^name = ' rules_engine/tabula/rendered-cards-mono.toml | sort > /tmp/mono_names.txt
diff /tmp/current_names.txt /tmp/mono_names.txt
```

Expected: no output (empty diff).

- [ ] **Step 4: Commit**

```bash
git add client/Assets/StreamingAssets/Tabula/rendered-cards-mono.toml
git commit -m "Add rendered-cards-mono.toml with original mono-tide assignments

Extracted from commit ff55c088, which has all 594 current cards
with the original tides.md-aligned tide assignments and tide-cost fields."
```

______________________________________________________________________

### Task 2: Delete rendered-cards-revised.toml

**Files:**

- Delete: `client/Assets/StreamingAssets/Tabula/rendered-cards-revised.toml`

- Delete:
  `client/Assets/StreamingAssets/Tabula/rendered-cards-revised.toml.meta` (Unity
  meta file)

- [ ] **Step 1: Delete the file and its meta**

```bash
rm client/Assets/StreamingAssets/Tabula/rendered-cards-revised.toml
rm client/Assets/StreamingAssets/Tabula/rendered-cards-revised.toml.meta
```

- [ ] **Step 2: Update doc references**

The files `docs/plans/quests/revised_quests.md` and
`docs/plans/quests/pack_quest_design.md` reference
`rendered-cards-revised.toml`. Update these references to point to
`rendered-cards.toml` (which now contains the revised tides) instead.

- [ ] **Step 3: Commit**

```bash
git add -u client/Assets/StreamingAssets/Tabula/rendered-cards-revised.toml client/Assets/StreamingAssets/Tabula/rendered-cards-revised.toml.meta
git add docs/plans/quests/revised_quests.md docs/plans/quests/pack_quest_design.md
git commit -m "Delete rendered-cards-revised.toml (stale), update doc references

rendered-cards.toml now contains the revised tide assignments.
rendered-cards-mono.toml contains the original mono-tide assignments.
The -revised file is no longer needed."
```

______________________________________________________________________

### Task 3: Sanity-check tide assignments via subagents

Launch 5-10 subagents in parallel. Each subagent receives a batch of ~60 cards
from `rendered-cards-mono.toml` and checks whether the tide assignments make
sense given `tides.md` philosophy. Each subagent should:

1. Read its batch of cards from `rendered-cards-mono.toml`
2. Read `docs/tides/tides.md` for the mono-tide philosophy
3. For each card, check whether the card's mechanics (rendered-text, subtype,
   card-type) align with its assigned tide's mechanical identity
4. Report any suspicious mismatches (e.g., a card with "abandon" mechanics in
   Bloom, or a card with "ramp" mechanics in Surge)

- [ ] **Step 1: Launch 8 parallel subagents, each checking ~75 cards**

Each subagent prompt should include:

- Line ranges to read from `rendered-cards-mono.toml` (divide the ~9800 lines
  into 8 roughly equal chunks by card-number ranges)

- Instructions to read `docs/tides/tides.md` for tide philosophy

- Instructions to flag cards where the tide assignment seems wrong

- Instructions to report in under 200 words: how many cards checked, how many
  look correct, list any suspicious ones with reasoning

- [ ] **Step 2: Review subagent results**

Compile the flagged cards from all subagents. A few mismatches are expected
(original assignments weren't perfect). The goal is to verify the file is
broadly correct — the original mono-tide assignments, not the revised ones.

- [ ] **Step 3: Final commit (if any fixes needed)**

If any obvious data corruption is found (e.g., cards with wrong IDs, missing
fields), fix and commit. Minor tide-assignment disagreements are expected and
should NOT be fixed — the goal is preserving the original assignments as-is.
