# Hidden Tides Quest Prototype Notes

## Purpose

`scripts/quest_prototype` is now the hidden-tides quest baseline. The prototype
no longer uses the legacy single-visible-tide model, tide crystals, or a mid-
run Dreamcaller draft site. Run identity comes from the selected Dreamcaller
package, which is resolved once at quest start and then reused for the rest of
the run.

## Current Flow

1. The asset setup script normalizes `rendered-cards.toml` and
   `dreamcallers.toml` into browser-loadable JSON.
2. Quest start offers 3 Dreamcallers.
3. Selecting one resolves the fixed package:
   - mandatory tides
   - one validated optional subset
   - the draft multiset
   - the shared Dreamsign pool
4. The player enters the first dreamscape immediately.
5. Draft sites spend from the fixed multiset and always reveal 4 unique cards
   when possible.
6. Dreamsign-bearing surfaces spend from the shared Dreamsign pool when the
   sign is shown, not when it is accepted.
7. The run ends when battle 7 is won.

## Runtime Invariants

- `QuestState` now persists `resolvedPackage` and `remainingDreamsignPool`.
- The live flow does not use `chosenTide`, `excludedTides`, `tideCrystals`, or
  `DreamcallerDraft`.
- Player-facing card chrome is neutral and rarity-driven; hidden package
  membership is only exposed in debug surfaces.
- Shops, battle rare rewards, and other package-adjacent generators prefer
  overlapping content but fall back to the broader pool when needed.
- Draft offers are reveal-and-spend. If fewer than 4 unique cards remain, the
  draft site ends cleanly instead of partial-filling an offer.
- Dreamsign surfaces never duplicate a sign within the run. If the pool is
  exhausted, they omit the Dreamsign option or fall back to a non-Dreamsign
  surface.

## Package Validation

Dreamcaller packages are validated up front against the authored card pool.
The current runtime rules are:

- optional subsets are built from 3- and 4-tide combinations
- legal final pool size: 175-225 cards
- preferred final pool size: 190-210 cards
- mandatory-only pool size: 110-150 cards
- copies per card use `min(2, overlap_count)`
- no legal subset is a hard load-time failure

## Code Pointers

- Content normalization and package resolution:
  `scripts/quest_prototype/src/data/quest-content.ts`
- Live state and mutations:
  `scripts/quest_prototype/src/state/quest-context.tsx`
- Dreamsign templates and package membership:
  `scripts/quest_prototype/src/data/dreamsigns.ts`
- Hidden-package debug surfaces:
  `scripts/quest_prototype/src/screens/DebugScreen.tsx`
- Manual QA guidance:
  `docs/quest_prototype/qa_tooling.md`

## Verification

Use the package scripts from `scripts/quest_prototype/`:

- `npm run setup-assets`
- `npm run typecheck`
- `npm test`
- `npm run build`

`npm run dev` runs `setup-assets` automatically before starting Vite.
