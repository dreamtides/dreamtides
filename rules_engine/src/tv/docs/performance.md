# TV Performance Analysis Report

**Date:** 2026-01-28
**Updated:** 2026-01-28 14:56 (with enhanced logging verification)
**Data Source:** `~/Library/Application Support/tv/logs/tv_2026-01-28.jsonl` and `tv_perf_2026-01-28.jsonl`

## Executive Summary

TV exhibits significant UI freezes (700-1700ms) after cell edits. The primary causes are:

1. **False conflict detection** - Every save triggers spurious "external change" detection (CONFIRMED)
2. **Cascading operations** - Each false conflict triggers reload → getDerivedColumnsConfig → derived computations
3. **File watcher detecting self-modifications** - Watcher emits events for our own saves

## Latest Findings (Post-Instrumentation)

Enhanced backend logging confirmed the root causes:

### Backend Performance is Fast

```
14:56:30.696  File saved: total=47ms (read=0ms, parse=26ms, write=15ms)
14:56:30.803  get_derived_columns_config: total=25ms (parse=25ms)
```

The backend operations are fast (~25-50ms). The 600-1400ms delays observed from the frontend were caused by **cascading operations triggered by false conflict detection**, not slow backend code.

### False Conflict Detection Confirmed

```
14:56:30.648  begin_save: mtime_before_save_ms=1769640986612
14:56:30.697  check_for_external_changes:
              mtime_before_ms=1769640986612
              mtime_after_ms=1769640990681  (+4069ms)
              external_change_detected=true
```

The mtime increased by ~4 seconds because **we wrote the file**. The conflict detection logic incorrectly flags this as an external change.

### File Watcher Detecting Self-Modifications

```
14:56:31.210  File watcher event will be emitted (T+514ms after save)
14:56:31.315  File watcher event will be emitted (T+619ms after save)
```

The file watcher is emitting events for modifications we made, causing additional reload cycles.

## Symptom Timeline

A typical single cell edit produces this cascade:

```
T+0ms      User edits cell, onChange fires
T+500ms    Debounce expires, saveData() starts
T+547ms    IPC saveTomlTable completes (47ms backend)
T+549ms    "Conflict detected" fires (FALSE POSITIVE - mtime changed by our save)
T+600ms    File reload triggered
T+625ms    File loaded (25ms backend)
T+700ms    getDerivedColumnsConfig called again (25ms backend, but waits for reload)
T+1014ms   File watcher emits event (detecting our own save)
T+1100ms   Another reload cycle may start
T+1200ms   saveData total: 700-1700ms (due to cascading operations)
```

## Issue 1: getDerivedColumnsConfig Called Redundantly

### Observed Behavior (Before Instrumentation)

The `getDerivedColumnsConfig` IPC call consistently took **600-1400ms** as observed from frontend perf logs:
```
14:39:00.740  getDerivedColumnsConfig: 603ms
14:39:03.284  getDerivedColumnsConfig: 661ms
14:39:08.619  getDerivedColumnsConfig: 1056ms
14:39:57.972  getDerivedColumnsConfig: 1360ms
```

### Observed Behavior (After Instrumentation)

Backend logging reveals the actual execution time is only **25ms**:
```
14:56:30.802  Parsed derived columns: read=0ms, parse=25ms, content=77556 bytes
14:56:30.803  get_derived_columns_config completed: parse=25ms, total=25ms
```

### Root Cause (Clarified)

The 600-1400ms delays were **not** caused by slow backend parsing. They were caused by:

1. **Cascading from false conflict detection**: The conflict triggers a reload, which blocks subsequent IPC calls
2. **Multiple calls per save**: The function is called twice per save cycle (once in save path, once in reload path)
3. **JavaScript main thread blocking**: Univer spreadsheet operations during reload block the event loop

### Architectural Problem

`getDerivedColumnsConfig` is called on **every save** at line 477 of `app_root.tsx`, but:
- The derived column configs rarely change (only when metadata section is edited)
- The configs are already available in React state from initial load
- Each call re-reads and re-parses the entire 78KB TOML file

### Recommendation

Use the cached configs from React state instead of re-fetching:

```typescript
// CURRENT: Called on every save (unnecessary)
const configs = await ipc.getDerivedColumnsConfig(sheetInfo.path);

// PROPOSED: Use cached value
const configs = derivedColumnState.configs[sheetId];
```

Only refresh configs when:
- File is first loaded
- A genuine external file change is detected
- User explicitly edits the metadata section

## Issue 2: False Conflict Detection (CONFIRMED)

### Observed Behavior

Every `File saved` event is immediately followed by `Conflict detected` with `external_change_detected=true`:

```
14:56:30.648  begin_save: mtime_before_save_ms=1769640986612
14:56:30.696  File saved (47ms)
14:56:30.697  check_for_external_changes:
              mtime_before_ms=1769640986612
              mtime_after_ms=1769640990681
              external_change_detected=true   ← FALSE POSITIVE
```

The mtime increased by 4069ms because **our save operation updated the file**.

### Root Cause (Verified)

In `state_machine.rs`, `check_for_external_changes()` compares:
- `mtime_before`: File modification time recorded when save began (before we wrote)
- `current_mtime`: File modification time after save completed (after we wrote)

The logic `current_mtime > mtime_before` will **always be true** after a successful save because our save operation updates the file's mtime.

```rust
// state_machine.rs - the bug
fn check_for_external_changes(path: &Path, mtime_before: Option<SystemTime>) -> bool {
    // ...
    current_mtime > before  // Always true because WE just modified the file!
}
```

### Impact

Each false conflict:
1. Emits `sync-conflict-detected` event to frontend
2. Frontend triggers full file reload (~25ms backend, but blocks UI)
3. Reload triggers `getDerivedColumnsConfig` again
4. Re-triggers all derived column computations
5. File watcher also detects the change and may trigger another cycle

### Recommendation

**Option A**: Record the mtime immediately after writing, then compare against that:
```rust
// In end_save(), after atomic write completes:
let mtime_after_our_save = get_file_mtime(&path);
// Store this, then in check_for_external_changes:
// external_change = current_mtime > mtime_after_our_save
```

**Option B**: Skip conflict detection entirely for self-initiated saves. The file watcher already handles external changes.

**Option C**: Use file content hash instead of mtime to detect actual content changes.

## Issue 3: File Watcher Detecting Self-Modifications

### Observed Behavior

The file watcher emits events 500-600ms after our own save completes:

```
14:56:30.696  File saved
14:56:31.210  File watcher event will be emitted (+514ms)
14:56:31.315  File watcher event will be emitted (+619ms)
```

### Root Cause

The file watcher uses `notify-debouncer-mini` with 500ms debouncing. When we save a file:
1. Our atomic write modifies the file
2. The OS notifies the watcher
3. After 500ms debounce, the watcher emits the event
4. The `is_busy()` check in `file_watcher.rs:227` returns `false` because the save already completed
5. The watcher emits a `toml-file-changed` event for our own modification

### Impact

This causes a second reload cycle after the false-conflict-triggered reload, compounding the performance issue.

### Recommendation

**Option A**: Extend the busy window to cover the debounce period:
- Keep the state in `Saving` for 600ms after save completes
- Or track "recently saved" files with timestamps

**Option B**: Use a save-ID or content hash:
- Before saving, record expected content hash
- In watcher, compare current hash to expected
- Skip events that match our expected state

**Option C**: Disable watcher during save window:
- Stop watcher before save
- Restart after save + debounce period

## Issue 4: Triple Event Duplication

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

## Issue 5: File System Latency (Minor)

### Observed Behavior

The file is located on Google Drive:
```
/Users/dthurn/Documents/GoogleDrive/dreamtides/rules_engine/tabula/cards.toml
```

File operations show reasonable latency:
```
14:56:30.696  File saved: read=0ms, parse=26ms, write=15ms, total=47ms
```

This is acceptable and **not** the primary cause of the 600-1400ms delays.

### Recommendation

Low priority. The primary issues are the cascading operations from false conflict detection.

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

### P0: Fix False Conflict Detection (Root Cause)

The conflict detection logic incorrectly flags self-modifications. This is the **primary cause** of the cascading performance issues.

**Problem:** `check_for_external_changes()` compares mtime before save to mtime after save. Since our save updates the mtime, this always returns `true`.

**Fix options:**
1. Record mtime after our write completes, compare against that for future checks
2. Skip conflict detection for self-initiated saves entirely
3. Use content hash instead of mtime

**Fix location:** `state_machine.rs:304-330`

### P1: Suppress File Watcher During Save Window

The file watcher detects our own modifications and triggers additional reload cycles.

**Problem:** The 500ms debounce means watcher events arrive after `is_busy()` returns false.

**Fix options:**
1. Extend busy window to 600ms after save
2. Track "recently saved" timestamps per file
3. Temporarily stop watcher during save

**Fix location:** `file_watcher.rs:227-235`

### P2: Cache getDerivedColumnsConfig Results

Stop calling `getDerivedColumnsConfig` on every save. Use cached React state.

**Fix location:** `app_root.tsx:474-478`

### P3: Deduplicate Event Listeners

Audit and fix the triple conflict event subscription.

**Fix location:** `app_root.tsx` useEffect hooks

## Monitoring After Fixes

After implementing fixes, verify improvement by checking:

```bash
# Check for false conflict detection (should show external_change_detected=false after fix)
grep "external_change_detected" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -20

# Check file watcher events (should not fire immediately after saves)
grep "File watcher event" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -20

# Check backend timing (should remain fast)
grep "get_derived_columns_config completed" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -20

# Check File saved timing breakdown
grep "File saved" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -20

# Check frontend saveData total timing
grep "saveData total" ~/Library/Application\ Support/tv/logs/tv_perf_*.jsonl | tail -20
```

### Target Metrics After Fixes

| Metric | Current | Target |
|--------|---------|--------|
| `external_change_detected` | Always `true` | `false` for self-saves |
| File watcher events after save | 2 events at +500ms | 0 events |
| `get_derived_columns_config` calls per save | 2 | 0 (use cache) |
| `saveData total` (frontend) | 700-1700ms | <200ms |
| Backend `File saved` | 47ms | 47ms (already fast) |

## Appendix: Log Inspection Commands

```bash
# View recent logs
tail -100 ~/Library/Application\ Support/tv/logs/tv_$(date +%Y-%m-%d).jsonl

# Search by component
grep "tv.sync.state_machine" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50
grep "tv.sync.watcher" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50
grep "tv.commands.derived" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50
grep "tv.toml.metadata.derived" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50

# Check mtime changes during saves
grep "mtime" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -30

# Check file watcher behavior
grep "watcher" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -30

# Find slow operations (100ms+)
grep -E "duration_ms.*[0-9]{3,}" ~/Library/Application\ Support/tv/logs/tv_perf_*.jsonl

# Find errors
grep '"level":"ERROR"' ~/Library/Application\ Support/tv/logs/tv_*.jsonl

# Full save cycle trace (combine state machine + toml + watcher)
grep -E "Saving state|File saved|external_change|watcher event" ~/Library/Application\ Support/tv/logs/tv_*.jsonl | tail -50
```

## Appendix: Sample Log Trace (Post-Instrumentation)

A single cell edit now produces this log trace:

```
14:56:30.648 [tv.sync.state_machine] Transitioned to Saving state
             mtime_before_save_ms=1769640986612

14:56:30.696 [tv.toml] File saved
             duration_ms=47, read=0ms, parse=26ms, write=15ms
             content_bytes=77576, output_bytes=77556

14:56:30.697 [tv.sync.state_machine] Checked for external changes after save
             mtime_before_ms=1769640986612
             mtime_after_ms=1769640990681
             external_change_detected=true  ← BUG: false positive

14:56:30.802 [tv.toml.metadata.derived] Parsed derived columns from file
             read=0ms, parse=25ms, content_bytes=77556

14:56:30.803 [tv.commands.derived] get_derived_columns_config completed
             parse=25ms, total=25ms

14:56:31.210 [tv.sync.watcher] File watcher event will be emitted
             event_timestamp_ms=1769640991210  ← BUG: self-modification detected
```
