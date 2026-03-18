---
name: qs-analyze
description: Use when analyzing quest prototype logs, investigating draft behavior, reviewing site visit sequences, or answering questions about a quest session. Triggers on quest log analysis, draft analysis, AI bot picks, session replay, qs-analyze, analyze quest, what happened in the draft.
---

# Quest Prototype Log Analysis

Analyze JSONL logs from quest prototype sessions to answer questions about player behavior, draft decisions, and quest progression.

## Step 1: Get the Log

Quest logs are downloaded from the prototype's HUD "Download Log" button as `.jsonl` files. The user will provide a file path. If no path is given, check for recently downloaded files:

```bash
ls -t ~/Downloads/quest-log-*.jsonl | head -5
```

## Step 2: Load and Parse Events

Read the JSONL file. Each line is a JSON object with `timestamp`, `event`, and `seq` fields, plus event-specific data. Key event types:

### Quest Lifecycle

| Event | Key Fields | Use For |
|-------|-----------|---------|
| `quest_started` | `essence`, `completionLevel` | Session start |
| `screen_transition` | `from`, `to` | Screen flow reconstruction |
| `quest_completed` | `essence`, `completionLevel`, `deckSize`, `dreamcallerName`, `dreamsignCount` | Final stats |

### Draft Events

| Event | Key Fields | Use For |
|-------|-----------|---------|
| `draft_pool_initialized` | `poolSize`, `excludedCount` | Pool creation |
| `pack_dealt` | `round`, `seatIndex`, `packSize` | Pack distribution |
| `draft_pick_player` | `cardNumber`, `cardName`, `pickIndex`, `packSize` | Player picks |
| `draft_pick_bot` | `seatIndex`, `cardNumber`, `cardName`, `pickIndex` | AI bot picks |
| `draft_pool_refreshed` | `newPoolSize`, `roundsCompleted` | Pool refresh after 3 rounds |
| `draft_site_entered` | `picksAvailable`, `currentRound`, `currentPick` | Draft site visits |
| `draft_site_completed` | `cardsDrafted` | Draft site completion |

### Site Visit Events

| Event | Key Fields | Use For |
|-------|-----------|---------|
| `dreamscape_entered` | `dreamscapeId`, `biomeName` | Dreamscape navigation |
| `dreamscape_completed` | `dreamscapeId`, `sitesVisitedCount` | Dreamscape completion |
| `site_entered` | `siteType`, `siteId` | Site visits |
| `site_completed` | `siteType`, `siteId` | Site completions |
| `dreamcaller_selected` | `name`, `tide`, `essenceBonus` | Dreamcaller draft |
| `battle_started` | `completionLevel`, `enemyName` | Battle initiation |
| `battle_won` | `completionLevel`, `essenceReward` | Battle completion |
| `rare_card_drafted` | `cardNumber`, `cardName` | Post-battle reward pick |

### Economy Events

| Event | Key Fields | Use For |
|-------|-----------|---------|
| `essence_changed` | `oldValue`, `newValue`, `delta`, `source` | Essence tracking |
| `card_added` | `cardNumber`, `cardName` | Deck additions |
| `card_removed` | `cardNumber`, `cardName` | Card purging |
| `shop_purchase` | `itemType`, `cardNumber`, `essenceRemaining` | Shop purchases |
| `shop_reroll` | `rerollCost`, `rerollCount` | Shop rerolls |

### Other Events

| Event | Key Fields | Use For |
|-------|-----------|---------|
| `dreamsign_acquired` | `name`, `tide`, `effect` | Dreamsign collection |
| `tempting_offer_accepted` | `journeyName`, `costName` | Tempting offer decisions |
| `card_transfigured` | `cardNumber`, `transfigurationType`, `colorName` | Transfiguration |
| `card_duplicated` | `cardNumber`, `copyCount` | Card duplication |
| `purge_completed` | `purgedCount`, `cardNumbers` | Card purging |
| `cleanse_completed` | `cleansedCards`, `cleansedDreamsigns` | Bane removal |
| `reward_claimed` | `rewardType` | Reward acceptance |
| `deck_viewer_opened` | — | UI interaction |
| `atlas_node_generated` | `dreamscapeId`, `biomeName` | Atlas growth |

## Step 3: Tide Mapping

The 7 tides and their theme colors:

| Tide | Color |
|------|-------|
| Bloom | Green / emerald |
| Arc | Yellow / amber |
| Ignite | Red / crimson |
| Pact | Pink / magenta |
| Umbra | Purple / deep violet |
| Rime | Blue / ice blue |
| Surge | Cyan / teal |
| Wild | Gray / silver (neutral) |

## Step 4: Answer Questions

### Draft Behavior

Track via `draft_pick_player` and `draft_pick_bot` events:
- Player picks show which cards were chosen and at what pack sizes
- Bot picks show AI decisions per seat (seats 1-9)
- Pack size decreases each pick (15 → 14 → ... → 6, then 5 discarded)
- Packs rotate direction by round (odd=left, even=right)

### Quest Progression

Use `screen_transition` events to reconstruct the full session flow. Cross-reference with `essence_changed` (source field) to track economy.

### Deck Evolution

Track `card_added`, `card_removed`, `card_transfigured`, and `card_duplicated` events chronologically to see how the deck evolved from empty to final state.

### Site Visit Sequence

Filter `site_entered` events to see the order of site visits. Group by `dreamscape_entered` events to see per-dreamscape breakdown.

## Analysis Script

For complex queries, write a TypeScript or Python script inline:

```python
import json, sys
events = [json.loads(line) for line in open(LOG_PATH)]
# Filter by event type
picks = [e for e in events if e["event"] == "draft_pick_player"]
# Track essence over time
essence_events = [e for e in events if e["event"] == "essence_changed"]
```

## Common Questions

- **"What cards did I draft?"** — Filter `draft_pick_player`, list `cardName` fields
- **"What did the bots pick?"** — Filter `draft_pick_bot`, group by `seatIndex`
- **"How did my essence change?"** — Filter `essence_changed`, plot `newValue` over `seq`
- **"What sites did I visit?"** — Filter `site_entered`, list `siteType` fields
- **"What was my final deck?"** — Check `quest_completed` or reconstruct from `card_added` / `card_removed` events
- **"How many battles did I win?"** — Count `battle_won` events
- **"What dreamcaller did I pick?"** — Find `dreamcaller_selected` event
- **"What dreamsigns did I collect?"** — Filter `dreamsign_acquired` events
- **"Did I accept any tempting offers?"** — Filter `tempting_offer_accepted` events

## When Logs Cannot Answer the Question

If the existing log events lack the data needed, add new logging rather than guessing.

### Procedure

1. **Identify the gap.** State exactly what data is missing.
2. **Add logging in `src/logging.ts`.** Call `logEvent(eventName, fields)` at the appropriate point in the relevant screen component or mutation.
3. **Re-run the prototype** to generate a fresh log with the new data.
4. **Update this skill.** Add the new event to the event table above.

### Rules

- Never invent data or guess values that aren't in the logs. If it's not logged, say so and offer to add logging.
- Follow the `qs` skill acceptance criteria after any code changes.
