# Appendix: ID System

This appendix documents the ID generation algorithm and collision handling.
See [Lattice Design](lattice_design.md#the-id-system) for an overview.

## ID Format

A Lattice ID consists of three components concatenated without separators:

1. **Prefix**: The literal character `L`
2. **Document Counter**: Base32-encoded integer, minimum 2 digits
3. **Client ID**: Base32-encoded random identifier, 3-6 digits

Example: `LK3DTX`
- Prefix: `L`
- Document Counter: `K3` (decimal 675)
- Client ID: `DTX`

Lattice uses RFC 4648 Base32 alphabet: `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`

## Document Counter

Counters start at 50 (Base32: `BS`) to ensure all IDs have at least 6
characters total.

When a client requests a new ID, the counter is incremented. If the repository
is re-cloned, counters are recovered from existing documents to prevent reuse.

## Client ID

When a client first uses Lattice, it generates a random client ID that doesn't
collide with existing IDs in the repository. Client IDs are stored in
`~/.lattice.toml` keyed by repository path.

### Selection Algorithm

Client IDs are selected uniformly at random from the set of possible IDs at
the appropriate length. The selection process:

1. Determine the required ID length based on the number of known clients
2. Generate a random ID by selecting each character uniformly from the Base32
   alphabet (`ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`)
3. If the generated ID already exists in the repository, discard it and
   generate a new one
4. Repeat until a unique ID is found

### Length Scaling

Client ID length scales with the number of known clients to balance collision
probability against ID brevity:

| Known Clients | ID Length | Possible Values |
|---------------|-----------|-----------------|
| 0-16          | 3         | 32768           |
| 17-64         | 4         | 1048576         |
| 65-256        | 5         | 33554432        |
| 257+          | 6         | 1073741824      |

If two clients independently select the same ID and both commit, `lat check`
detects the duplicate and the later contributor must renumber.

## Display and Linking

IDs are displayed in uppercase. In markdown, IDs can be used directly as link
targets: `[text](LK3DTX)`.

## Errors

- Missing prefix: `K3DTX` → "Invalid Lattice ID: missing L prefix"
- Too short: `LABCD` → "Invalid Lattice ID: minimum 6 characters"
- Invalid characters: `L01ABC` → "Invalid Lattice ID: invalid character '0'"
- Unknown ID: Link to `LXXXXX` where no document exists
