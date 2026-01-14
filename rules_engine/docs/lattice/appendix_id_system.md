# Appendix: ID System

This appendix documents the ID generation algorithm and collision handling.
See [Lattice Design](lattice_design.md#the-id-system) for an overview.

## ID Format

A Lattice ID consists of three components concatenated without separators:

1. **Prefix**: The literal character `L`
2. **Document Counter**: Base32-encoded integer, minimum 2 digits
3. **Client ID**: Base32-encoded random identifier, 2-5 digits

Example: `LK3DT`
- Prefix: `L`
- Document Counter: `K3` (decimal 675)
- Client ID: `DT`

Lattice uses RFC 4648 Base32 alphabet: `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`

## Document Counter

Counters start at 50 (Base32: `BS`) to ensure all IDs have at least 5
characters total.

When a client requests a new ID, the counter is incremented. If the repository
is re-cloned, counters are recovered from existing documents to prevent reuse.

## Client ID

When a client first uses Lattice, it generates a random client ID that doesn't
collide with existing IDs in the repository. Client IDs are stored in
`~/.lattice.toml` keyed by repository path.

| Known Clients | ID Length | Possible Values |
|---------------|-----------|-----------------|
| 0-16          | 2         | 1024            |
| 17-64         | 3         | 32768           |
| 65-256        | 4         | 1048576         |
| 257+          | 5         | 33554432        |

If two clients independently select the same ID and both commit, `lat check`
detects the duplicate and the later contributor must renumber.

## Display and Linking

IDs are displayed in uppercase. In markdown, IDs can be used directly as link
targets: `[text](LK3DT)`.

## Errors

- Missing prefix: `K3DT` → "Invalid Lattice ID: missing L prefix"
- Too short: `LAB` → "Invalid Lattice ID: minimum 5 characters"
- Invalid characters: `L01AB` → "Invalid Lattice ID: invalid character '0'"
- Unknown ID: Link to `LXXXX` where no document exists
