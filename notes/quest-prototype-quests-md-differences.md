# Quest Prototype vs. `quests.md` Differences

This note synthesizes 10 read-only subagent passes comparing
`docs/plans/quests/quests.md` with the current behavior in
`scripts/quest_prototype`.

## High-Confidence Matches

- Dreamcaller package resolution broadly matches the plan: mandatory-only
  validation, optional subset selection, legal/preferred pool ranges, overlap
  copy weighting capped at two, and Dreamsign pool seeding are implemented in
  `src/data/quest-content.ts`.
- Draft offer behavior broadly matches the plan: each offer shows four unique
  cards when possible, cards are sampled weighted by remaining copies, shown
  cards are spent immediately, the finite draft pool persists across sites, and
  each Draft site provides five picks.
- Draft site counts by completion level match the table in `quests.md`: two at
  completion levels 0-1, one at 2-3, and zero at 4+.

## Economy and Limits

- Omens are not implemented. `quests.md` defines essence and omens, with omens
  used for shop Dreamsign purchases, shop rerolls, and 1-3 omen battle victory
  rewards. `QuestState` only stores `essence`.
- Essence is uncapped. The plan specifies a default cap of 500 and loss of
  overflow; `changeEssence()` applies raw deltas without a cap field or clamp.
- Shop Dreamsigns and rerolls currently spend essence, not omens.
- Economy values are hardcoded in TypeScript rather than configured data:
  starter essence, shop card prices, Dreamsign prices, reroll costs, essence
  site ranges, reward-site essence ranges, and battle reward formulas.
- Battle rewards grant essence and one selected card, but do not grant omens.
- The max-50-card pre-battle purge and min-25-card battle padding rules are not
  implemented. Battle bootstrap uses the exact quest deck.
- The Dreamsign cap is only partially enforced. Offering and draft screens show
  a purge overlay at 12 Dreamsigns, but the central `addDreamsign()` silently
  no-ops at the cap, so shop, reward, and tempting-offer acquisition paths can
  fail invisibly instead of forcing a purge.

## Configuration and Data Sources

- The prototype still violates the broad TOML/configuration goal in many
  places. Starter deck IDs, package thresholds, atlas topology, site weights,
  site counts, biomes, shop prices, reroll costs, reward formulas, and battle
  opponent generation are code constants/functions.
- Dreamsign asset generation and runtime data are disconnected.
  `setup-assets.mjs` writes `public/dreamsign-data.json`, but
  `loadQuestContent()` uses hardcoded `DREAMSIGN_TEMPLATES` instead.
- Runtime Dreamsign objects drop generated metadata such as id and image fields,
  so collected Dreamsigns cannot use the generated Dreamsign art pipeline.
- Opponent decks are generated dynamically from the card database and random
  Dreamcaller-derived descriptors. `quests.md` says opponent Dreamcallers,
  Dreamsigns, decks, and difficulty are statically defined in TOML for now.
- Run-specific pools persist through `sessionStorage`, but restored
  `resolvedPackage`, `draftState.remainingCopiesByCard`, and
  `remainingDreamsignPool` are not revalidated against current content after
  data changes.

## Atlas and Progression

- Initial atlas topology is hardcoded: the prototype always creates a completed
  Nexus and exactly two available starting dreamscapes.
- Atlas expansion differs from the plan. `quests.md` says each completed
  dreamscape adds around two to four unavailable nodes adjacent to newly
  available nodes. The prototype adds one node after the first completion and
  two to three later, connected to the completed node, then marks nodes
  available if connected to any completed node.
- New dreamscapes are generated with the pre-victory completion level from the
  battle start, which makes site mix and draft counts lag the completed
  dreamscape count by one.
- Additional site generation samples with replacement. This can produce
  duplicate non-Draft/non-Essence sites even though `quests.md` says each site
  type should appear at most once per dreamscape except Draft and Essence.
- Reward-site contents are not known in advance. The atlas leaves reward site
  data unresolved, `rewardPreviewLabel()` returns null, and rewards are
  generated when entering the site.
- Enhanced sites are opportunistic. Biomes only mark a matching site if random
  generation already included that type; the plan says each biome produces its
  affinity enhanced site.
- Enhanced-site metadata differs from the plan: `TemptingOffer` has a biome
  affinity despite being absent from `quests.md`, and `DreamsignDraft` does not
  have its own biome affinity.
- The player can return to the Atlas before completing a dreamscape battle via
  the `Return to Atlas` button. The plan describes choosing another dreamscape
  only after completing the current dreamscape's battle.
- Battle locking is mostly UI-level. The Dreamscape screen disables the Battle
  site until non-battle sites are visited, but routing can still render a battle
  site if state is set directly.
- Battle site count is not configurable; generation always appends one Battle
  site.

## Site and Battle Behavior

- Default battle flow is auto-victory, not a playable match against an AI
  opponent. Playable mode exists behind runtime config, while auto mode always
  proceeds from pre-battle to animation to victory.
- Defeat flow only exists for playable battles. Auto battles cannot lose.
- Playable battles skip the planned pre-battle opponent presentation; auto mode
  has a pre-battle splash, but playable mode routes directly into the battle
  screen.
- Opposing Dreamsigns are represented as a count, not concrete shown
  Dreamsigns.
- The shared Victory/Defeat screen described in `quests.md` is not present as
  specified. Auto battles use `BattleRewardSurface`, while terminal run
  outcomes use separate `QuestCompleteScreen` and `QuestFailedScreen`.
- `TemptingOffer` is a real site type in code, appears in atlas generation, has
  routing, and has a biome affinity, but has no section in `quests.md`.
- Transfiguration is partial. The prototype supports Viridian, Golden, Scarlet,
  Azure, and Bronze. It lacks Magenta, Rose, and Prismatic, and Golden modifies
  text procedurally rather than using TOML-defined golden variants.
- Cleanse is not random. The plan says remove up to three random Banes; the
  prototype takes the first three bane items in deck/Dreamsign order and
  removes all shown.
- NPC/dialog/portrait-vs-landscape behavior is broadly omitted from non-battle
  site screens. Current screens are mostly direct card grids, panels, or
  buttons.
- Dreamcaller and Dreamsign gameplay effects are mostly display summaries, not
  engine-affecting effects.
- Dreamwell is implicit in battle energy behavior, not represented as a named
  quest/battle state or screen concept.

## UI and Prototype-Only Behavior

- `quests.md` contains an internal tension around initial routing. Its current
  prototype section says Dreamcaller selection enters the first dreamscape
  directly, while the Dream Atlas section says the Atlas is navigated after
  selection. Current code follows direct-to-first-dreamscape.
- Dreamcaller selection is simpler than the planned UI: clickable 2D/portrait
  cards, no separate primary action button below each card, and no selected
  card animation into the HUD.
- The Dream Atlas is a 2D SVG graph with draggable pan and single-click entry,
  not 3D miniature worlds with hover/long-press preview and click-again zoom.
- Dreamscape sites render as a vertical list of cards, not white icons on black
  circular backgrounds with camera zoom and NPC intro.
- HUD/debug/log/card-source surfaces exist but are not described in
  `quests.md`: `View Deck`, `Why Cards`, `Debug`, `Download Log`, package
  debug overlay, and card provenance overlay.
- In-tab `sessionStorage` persistence exists but is not described in the master
  quest document.
- Runtime debug entry points can bypass normal quest start, such as playable
  battle start-in-battle fixtures.

## Testing Differences

- Some tests explicitly lock in behavior that conflicts with `quests.md`:
  reward sites unresolved until entry, one-node first atlas expansion, essence
  shop costs for Dreamsigns/rerolls, dynamically generated battle opponents,
  and the partial transfiguration set.
- The plan calls for integration-style quest tests and manual Unity/ABU QA.
  The web prototype mostly uses Vitest unit/component tests for generators,
  screens, and state helpers.
- Screen test coverage is uneven. Reward and Dreamsign screens have tests, but
  several site screens do not have direct behavioral screen tests.

## Overall Read

The current prototype is closest to `quests.md` in the package-based start and
fixed draft-pool systems. The largest divergences are the missing omen/cap
economy, hardcoded configuration, atlas generation/progression details,
reward-site preview behavior, auto/synthetic battle flow, partial site effects,
and prototype-only web UI/debug/persistence surfaces that the master quest
document does not currently describe.
