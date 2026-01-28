# TV Performance Analysis Report

**Date:** 2026-01-28
**Analyzed Period:** 2026-01-28 11:29:05 to 14:40:33
**Data Source:** `~/Library/Application Support/tv/logs/tv_2026-01-28.jsonl` and `tv_perf_2026-01-28.jsonl`

## Executive Summary

TV exhibits significant UI freezes (700-1700ms) after cell edits. The primary causes are:

1. **Redundant IPC call** - `getDerivedColumnsConfig` re-parses the entire TOML file on every save
2. **False conflict detection** - Every save triggers spurious "external change" detection
3. **Cascading operations** - Each false conflict triggers a full reload cycle

## Symptom Timeline

A typical single cell edit produces this cascade:

```
T+0ms      User edits cell, onChange fires
T+500ms    Debounce expires, saveData() starts
T+560ms    IPC saveTomlTable completes (60ms backend)
T+562ms    "Conflict detected" fires (false positive)
T+600ms    File reload starts
T+625ms    File loaded (25ms backend)
T+1200ms   getDerivedColumnsConfig completes (600-1400ms!)
T+1200ms   saveData total: 700-1700ms
```

## Issue 1: getDerivedColumnsConfig Bottleneck

### Observed Behavior

The `getDerivedColumnsConfig` IPC call consistently takes **600-1400ms**, despite:
- Backend file read: ~5ms
- Backend TOML parse: ~10ms
- Total backend work: <20ms

Sample measurements from perf logs:
```
14:39:00.740  getDerivedColumnsConfig: 603ms
14:39:03.284  getDerivedColumnsConfig: 661ms
14:39:05.298  getDerivedColumnsConfig: 645ms
14:39:07.028  getDerivedColumnsConfig: 731ms
14:39:08.619  getDerivedColumnsConfig: 1056ms
14:39:57.972  getDerivedColumnsConfig: 1360ms
```

### Root Cause Hypothesis

The massive gap between backend execution time (~20ms) and frontend-observed time (600-1400ms) suggests:

1. **IPC thread contention**: Tauri's IPC may be waiting for other operations
2. **JavaScript main thread saturation**: Univer spreadsheet operations blocking
3. **State machine lock contention**: Sync state machine mutex blocking IPC handlers

### Architectural Problem

`getDerivedColumnsConfig` is called on **every save** at line 477 of `app_root.tsx`:

```typescript
const configs = await ipc.getDerivedColumnsConfig(sheetInfo.path);
```

This re-reads and re-parses the entire 78KB `cards.toml` file just to retrieve 2 derived column configs that rarely change. The configs are already cached in React state from the initial load.

### Recommendation

Cache derived column configs and only refresh on explicit metadata changes. The current pattern:

```typescript
// CURRENT: Called on every save
const configs = await ipc.getDerivedColumnsConfig(sheetInfo.path);
```

Should become:

```typescript
// PROPOSED: Use cached value
const configs = derivedColumnState.configs[sheetId];
```

## Issue 2: False Conflict Detection

### Observed Behavior

Every `File saved` event is immediately followed by 3x `Conflict detected`:

```
14:40:13.959  File saved (44ms)
14:40:13.960  Conflict detected
14:40:13.960  Conflict detected
14:40:13.962  Conflict detected
```

The conflict detection fires within 1-3ms of the save completing.

### Root Cause

In `state_machine.rs`, `check_for_external_changes()` compares:
- `mtime_before`: File modification time recorded when save began
- `current_mtime`: File modification time after save completed

The logic `current_mtime > mtime_before` will **always be true** after a successful save because the save operation itself updates the file's mtime.

```rust
// state_machine.rs:304-314
fn check_for_external_changes(path: &Path, mtime_before: Option<SystemTime>) -> bool {
    // ...
    current_mtime > before  // Always true after we saved!
}
```

### Impact

Each false conflict:
1. Emits `sync-conflict-detected` event
2. Triggers full file reload (~25ms)
3. Triggers `getDerivedColumnsConfig` again (~700ms)
4. Re-triggers all derived column computations

### Recommendation

The conflict detection should compare `mtime_before` to the mtime recorded **during** the save operation, not after. Alternatively, track the save operation's expected mtime and exclude self-modifications.

## Issue 3: Triple Event Duplication

### Observed Behavior

Every conflict event is logged 3 times with identical timestamps:

```
14:40:06.879  Conflict detected x3
```

### Root Cause

Likely causes:
- 3 event listeners registered for `sync-conflict-detected`
- Event being emitted from 3 locations
- React StrictMode double-mounting with an additional listener

### Recommendation

Audit event listener registration in `app_root.tsx` to ensure single subscription.

## Issue 4: File System Latency

### Observed Behavior

The file is located on Google Drive:
```
/Users/dthurn/Documents/GoogleDrive/dreamtides/rules_engine/tabula/cards.toml
```

File operations show occasional high latency (40-60ms for reads, 40-60ms for writes), but this alone doesn't explain the 600-1400ms delays.

### Recommendation

Consider whether development can use a local file path, or implement a local cache with background sync.

## Performance Logging Adequacy

### Current Frontend Logging (Adequate)

| Operation | Logged | Context |
|-----------|--------|---------|
| `extractDataFromSheet` | Yes | commandId, rowCount, columnCount |
| `isDataEqual comparison` | Yes | rowCount, dataUnchanged |
| `handleChange` | Yes | scheduled/skipped reason |
| `IPC saveTomlTable` | Yes | path, rowCount, uuidsGenerated |
| `getDerivedColumnsConfig` | Yes | configCount |
| `saveData total` | Yes | success, rowCount |

### Current Backend Logging (Now Enhanced)

After this analysis, the following logging was added:

| Operation | Component | Fields Added |
|-----------|-----------|--------------|
| `get_derived_columns_config` | `tv.commands.derived` | `parse_duration_ms`, `total_duration_ms` |
| `parse_derived_columns_with_fs` | `tv.toml.metadata.derived` | `read_duration_ms`, `parse_duration_ms`, `content_bytes` |
| `save_toml_document` | `tv.toml` | `read_duration_ms`, `parse_duration_ms`, `write_duration_ms`, `content_bytes`, `output_bytes` |
| `check_for_external_changes` | `tv.sync.state_machine` | `mtime_before_ms`, `mtime_after_ms`, `external_change_detected` |
| `begin_save` | `tv.sync.state_machine` | `mtime_before_save_ms` |
| File watcher events | `tv.sync.watcher` | `event_timestamp_ms` |

### Remaining Logging Gaps

| Missing | Why Needed |
|---------|------------|
| IPC queue depth | Detect IPC thread contention |
| JS event loop lag | Detect main thread saturation |
| Mutex wait times | Detect lock contention |
| Derived computation queue depth | Detect compute backlog |

## Recommended Fixes (Priority Order)

### P0: Fix False Conflict Detection

The conflict detection logic incorrectly flags self-modifications. This causes:
- Unnecessary file reloads
- Redundant `getDerivedColumnsConfig` calls
- Cascading performance degradation

**Fix location:** `state_machine.rs:304-314`

### P1: Cache getDerivedColumnsConfig Results

Derived column configs rarely change. Cache them in React state and only refresh when:
- File is first loaded
- User explicitly edits metadata section
- A file reload is triggered by actual external change

**Fix location:** `app_root.tsx:474-478`

### P2: Deduplicate Event Listeners

Audit and fix the triple conflict event subscription.

**Fix location:** `app_root.tsx` useEffect hooks

### P3: Make getDerivedColumnsConfig Async

If caching isn't feasible, ensure the IPC call doesn't block the UI thread by moving the await outside the critical path.

## Monitoring After Fixes

After implementing fixes, verify improvement by checking:

```bash
# Check getDerivedColumnsConfig timing
grep "getDerivedColumnsConfig" ~/Library/Application\ Support/tv/logs/tv_perf_*.jsonl | tail -20

# Check for conflict detection
grep "external_change_detected" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -20

# Check saveData total timing
grep "saveData total" ~/Library/Application\ Support/tv/logs/tv_perf_*.jsonl | tail -20
```

Target metrics after fixes:
- `getDerivedColumnsConfig`: <50ms (or eliminated via caching)
- `saveData total`: <200ms
- False conflict rate: 0%

## Appendix: Log Inspection Commands

```bash
# View recent logs
tail -100 ~/Library/Application\ Support/tv/logs/tv_$(date +%Y-%m-%d).jsonl

# Search by component
grep "tv.sync.state_machine" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50
grep "tv.commands.derived" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50

# Find slow operations
grep -E "duration_ms.*[0-9]{3,}" ~/Library/Application\ Support/tv/logs/tv_perf_*.jsonl

# Find errors
grep '"level":"ERROR"' ~/Library/Application\ Support/tv/logs/tv_*.jsonl
```
