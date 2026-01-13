# Appendix: ID System

## ID Format Specification

A Lattice ID consists of three components concatenated without separators:

1. **Prefix**: Always the literal character `L`
2. **Document Counter**: Base32-encoded integer, minimum 2 digits
3. **Client ID**: Base32-encoded random identifier, 2-5 digits

Example: `LK1DT`
- Prefix: `L`
- Document Counter: `K1` (decimal 641)
- Client ID: `DT`

## Base32 Encoding

Lattice uses RFC 4648 Base32 alphabet: `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`

This encoding is:
- Case-insensitive (normalized to uppercase for display)
- Avoids ambiguous characters (no 0/O, 1/I/l confusion)
- URL-safe without percent-encoding
- Pronounceable and typeable

### Encoding Rules

For a number N, repeatedly divide by 32 and map remainders:
- 0 → A, 1 → B, ..., 25 → Z, 26 → 2, 27 → 3, ..., 31 → 7

Minimum digits are enforced by left-padding with 'A' (value 0).

### Decoding Rules

For a string S, iterate characters and accumulate:
- A → 0, B → 1, ..., Z → 25, 2 → 26, 3 → 27, ..., 7 → 31

Invalid characters (0, 1, 8, 9, non-alphanumeric) cause parse errors.

## Document Counter

### Initial Value

Counters start at 50 (Base32: `1I`) to ensure all IDs have at least 5
characters total. This provides visual consistency and reduces the
chance of IDs resembling common words.

### Increment Behavior

When a client requests a new ID:
1. Query `client_counters` table for current value
2. If no entry exists, initialize from highest committed document
3. Increment and store the new value
4. Return the incremented value

### Recovery After Re-Clone

If the repository is deleted and re-cloned:
1. The index is rebuilt from scratch
2. `client_counters` is populated from existing documents
3. New counter values start above the highest existing ID

This prevents ID reuse even without persistent local state.

### Batch Generation

The `lat generate-ids` command allocates multiple IDs:
1. Read current counter
2. Generate N IDs by incrementing
3. Return the IDs but do NOT update the stored counter

IDs are only "consumed" when documents containing them are committed.
This allows speculative ID generation without waste.

## Client ID

### Initial Selection

When a client first uses Lattice:
1. Enumerate all Lattice IDs in the repository
2. Extract unique client ID suffixes
3. Count known clients
4. Select ID length based on client count
5. Generate random ID avoiding collisions

### Length Selection

| Known Clients | ID Length | Possible Values |
|---------------|-----------|-----------------|
| 0-16          | 2         | 1024            |
| 17-64         | 3         | 32768           |
| 65-256        | 4         | 1048576         |
| 257+          | 5         | 33554432        |

With random selection and these ranges, collision probability is low
for the window between ID selection and first commit.

### Persistence

Client IDs are stored in `~/.lattice.toml`:

```toml
[clients]
"/absolute/path/to/repo" = "DT"
```

The absolute path key ensures the same client ID is used even if the
repository is accessed via different paths (symlinks, etc.).

### Collision Handling

If two clients independently select the same ID and both commit:
1. `lat check` detects the duplicate
2. The later contributor is asked to renumber
3. Renumbering changes all their IDs in uncommitted work
4. The new ID is stored in `~/.lattice.toml`

## Parsing

### ID Recognition

A string is a valid Lattice ID if:
1. Starts with `L` (case-insensitive)
2. Followed by 4+ Base32 characters
3. No invalid characters

### Component Extraction

Given an ID, extracting components requires knowing the client ID length.
This is determined by:
1. Check if trailing 5 characters form a known client ID
2. If not, check trailing 4 characters
3. Continue down to 2 characters
4. Remaining characters after prefix are the document counter

In practice, client IDs are looked up in the index for disambiguation.

## Section IDs

Section IDs share the same format as document IDs but are stored in
the `sections` table rather than `documents`. When resolving a link:
1. Check `documents` table first
2. If not found, check `sections` table
3. Section lookup returns the parent document ID and line range

## ID Generation Algorithm

```
function generate_id(client_id):
    counter = get_or_init_counter(client_id)
    counter_str = base32_encode(counter, min_digits=2)
    return "L" + counter_str + client_id

function get_or_init_counter(client_id):
    if client_counters.has(client_id):
        return client_counters.get(client_id)

    # Find highest existing counter for this client
    max_counter = 49  # Start at 50 after increment
    for doc in documents where doc.id ends with client_id:
        counter = extract_counter(doc.id, client_id)
        max_counter = max(max_counter, counter)

    client_counters.set(client_id, max_counter + 1)
    return max_counter + 1
```

## Display Conventions

- IDs are always displayed in uppercase
- Parsing is case-insensitive
- In markdown links, IDs replace the URL: `[text](LK1DT)`
- In section headers, IDs are bracketed: `# [LK1DT] Header`

## Error Cases

**Invalid ID format:**
- Missing prefix: `K1DT` → Error: "Invalid Lattice ID: missing L prefix"
- Too short: `LAB` → Error: "Invalid Lattice ID: minimum 5 characters"
- Bad characters: `L01AB` → Error: "Invalid Lattice ID: character '0' not allowed"

**Reference errors:**
- Unknown ID: Link to `LXXXX` where no document exists
- Ambiguous section: Multiple sections with same ID (shouldn't happen)

**Generation errors:**
- Counter overflow: Exceeds Base32 representable range (unlikely: 2^62+)
