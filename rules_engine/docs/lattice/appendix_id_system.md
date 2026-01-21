# Appendix: ID System

This appendix documents the ID generation algorithm and collision handling.
See [Lattice Design](lattice_design.md#the-id-system) for an overview.

## ID Format

A Lattice ID is a 6-character identifier consisting of:

1. **Prefix**: The literal character `L`
2. **Scrambled Payload**: 5 Base32 characters encoding a permuted 25-bit integer

Example: `LJCQ2X`
- Prefix: `L`
- Payload: `JCQ2X` (a scrambled encoding that maps back to a specific
  client/document pair)

Lattice uses RFC 4648 Base32 alphabet: `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`

## Design Goals

The previous ID system concatenated a document counter with a client ID, producing
sequences like `LK3DTX`, `LK4DTX`, `LK5DTX` for consecutive documents. This made
IDs difficult to distinguish at a glance.

The new system applies a keyed bijective permutation to the input, ensuring:

1. **Visual distinction**: Consecutive IDs appear unrelated (`LJCQ2X`, `LWN5RP`,
   `L4DKAT`)
2. **Determinism**: All clients produce identical IDs for the same input
3. **Reversibility**: Any ID can be decoded back to its original components
4. **Collision-free**: The bijective property guarantees no two inputs produce
   the same output

## Input Space

The 25-bit input space (5 Base32 digits = 32^5 = 33,554,432 values) is
partitioned between client identification and document counting:

| Component        | Bits | Range       | Purpose                          |
|------------------|------|-------------|----------------------------------|
| Client Partition | 10   | 0-1023      | Identifies the creating client   |
| Document Counter | 15   | 0-32767     | Per-client document sequence     |

The combined input is: `input = (client_partition << 15) | document_counter`

This supports up to 1024 distinct clients, each capable of creating up to 32,768
documents—sufficient for any practical repository.

## Permutation Key Derivation

The permutation key must be identical across all clients working on the same
repository. Lattice derives the key from the repository's **initial commit hash**:

```
key_material = SHA-256(initial_commit_hash)
```

The initial commit hash is:
- Deterministic: all clones of a repository share the same initial commit
- Immutable: cannot change after repository creation
- Unique: effectively random across different repositories

For repositories without commits (freshly initialized), Lattice uses a fallback
key derived from the repository path until the first commit is made. Any IDs
generated before the first commit are re-mapped during `lat fmt` after committing.

### Key Storage and Caching

The derived key is cached in `.lattice/config.toml` under `permutation_key` for
performance. On each invocation, Lattice verifies the cached key matches the
current initial commit hash; mismatches trigger key regeneration and a warning.

## Permutation Algorithm

Lattice uses an 8-round Feistel network, which is bijective by construction.
The Feistel structure guarantees that every input maps to exactly one output
and vice versa, regardless of the round function used.

### Feistel Network Structure

For a 25-bit input:

1. Split into left half L (12 bits) and right half R (13 bits)
2. For each round i (0-7):
   - L' = R
   - R' = L XOR F(R, K_i)
3. Concatenate final (L', R') as 25-bit output

### Round Function

The round function F(R, K_i) produces a 12-bit output from a 13-bit input and
round key:

```
F(R, K_i) = ((R * K_i) + (K_i >> 4)) mod 4096
```

Where K_i is a 32-bit round key derived from the key material:
- K_0 through K_7 are bytes 0-3, 4-7, 8-11, 12-15, 16-19, 20-23, 24-27, 28-31
  of the SHA-256 hash, interpreted as little-endian 32-bit integers

The multiplication and addition provide good diffusion, ensuring small input
changes produce large output changes.

### Decoding

To decode an ID back to its components, run the Feistel network in reverse:

1. Parse the 5 Base32 characters to a 25-bit integer
2. Split into L (12 bits) and R (13 bits)
3. For each round i (7 down to 0):
   - R' = L
   - L' = R XOR F(L, K_i)
4. Extract client_partition (high 10 bits) and document_counter (low 15 bits)

## Client Partition Assignment

When a client first uses Lattice in a repository, it needs a unique partition
number (0-1023). The assignment algorithm:

1. **Hash-based initial selection**: Compute `SHA-256(machine_id || repo_path)`
   and take the result modulo 1024
2. **Collision detection**: Scan existing document IDs to find all used partitions
3. **Increment on collision**: If the selected partition is in use, increment
   (wrapping at 1024) until finding an unused partition
4. **Store assignment**: Save the partition in `~/.lattice.toml` keyed by
   repository path

The machine_id is derived from platform-specific identifiers (hostname, MAC
address, or `/etc/machine-id` on Linux).

### Partition Recovery

If `~/.lattice.toml` is lost, the client must recover its partition:

1. Scan all documents in the repository
2. Decode each ID to find its client partition
3. For documents the client authored (determined by git blame or commit author),
   identify the most common partition
4. Assign that partition to the client

If no documents can be attributed to the client, assign a new partition using
the standard collision-avoidance algorithm.

## Document Counter

Each client maintains a counter starting at 0. When generating a new ID:

1. Increment the counter
2. Combine with client partition: `input = (partition << 15) | counter`
3. Apply the Feistel permutation
4. Encode as 5 Base32 characters, prepend `L`

Counters are stored in `~/.lattice.toml` per repository. On re-clone, counters
are recovered by scanning existing documents and finding the maximum counter
value for the client's partition.

## Example ID Generation

Given:
- Initial commit hash: `a1b2c3d4e5f6...` (SHA-256 produces key material)
- Client partition: 42
- Document counter: 100

Steps:
1. Compute input: `(42 << 15) | 100 = 1376356`
2. Apply 8-round Feistel with derived keys
3. Result (example): `19847562` (25-bit integer)
4. Encode as Base32: `JCQ2X`
5. Final ID: `LJCQ2X`

A subsequent document (counter 101) might produce `LWN5RP`—visually distinct
despite being consecutive.

## Migration from Legacy IDs

Existing repositories with legacy IDs (concatenated counter + client ID) can
be migrated using `lat migrate-ids`:

1. Parse each legacy ID to extract counter and client ID
2. Map legacy client IDs to new partition numbers (first-come-first-served)
3. Generate new scrambled IDs using the Feistel permutation
4. Update all documents and links atomically
5. Commit the changes

The migration is reversible; `lat migrate-ids --reverse` restores legacy format.

## Display and Linking

IDs are displayed in uppercase. In markdown, IDs can be used directly as link
targets: `[text](LJCQ2X)`.

## Errors

- Missing prefix: `JCQ2X` → "Invalid Lattice ID: missing L prefix"
- Wrong length: `LABCD` → "Invalid Lattice ID: must be exactly 6 characters"
- Invalid characters: `L01ABC` → "Invalid Lattice ID: invalid character '0'"
- Unknown ID: Link to `LXXXXX` where no document exists
- Partition exhausted: "Client partition space exhausted (1024 clients maximum)"
- Counter exhausted: "Document counter exhausted for this client (32767 maximum)"

## Security Considerations

The permutation is not cryptographically secure—it provides visual scrambling,
not secrecy. Anyone with access to the repository can:

- Derive the permutation key from the initial commit
- Decode any ID to its partition and counter
- Predict future IDs if they know a client's partition and counter

This is acceptable because Lattice IDs are not secrets; they are stable
identifiers for public documents within a repository.
