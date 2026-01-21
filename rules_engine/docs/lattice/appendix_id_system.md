# Appendix: ID System

This appendix documents the ID generation algorithm and collision handling.
See [Lattice Design](lattice_design.md#the-id-system) for an overview.

## ID Format

A Lattice ID consists of three logical components:

1. **Prefix**: The literal character `L`
2. **Document Counter**: Base32-encoded integer, minimum 2 digits
3. **Client ID**: Base32-encoded random identifier, 3-6 digits

The counter and client ID are concatenated and then scrambled using a bijective
permutation before encoding. This makes consecutive IDs visually distinct while
preserving all structural properties.

Example: `LJCQ2X`
- Prefix: `L`
- Payload: `JCQ2X` (scrambled encoding of counter + client ID)

Lattice uses RFC 4648 Base32 alphabet: `ABCDEFGHIJKLMNOPQRSTUVWXYZ234567`

## Design Goals

Without scrambling, sequential IDs look nearly identical: `LK3DTX`, `LK4DTX`,
`LK5DTX`. The scrambling layer ensures:

1. **Visual distinction**: Consecutive IDs appear unrelated
2. **Determinism**: All clients produce identical scrambled output for the same
   input, using a hardcoded key
3. **Reversibility**: Any ID can be unscrambled to recover counter and client ID
4. **Collision-free**: Bijective permutation guarantees unique outputs

## Variable Length

IDs have a minimum length of 6 characters (`L` + 2 counter digits + 3 client ID
digits) but grow as needed:

- Document counters start at 50 (Base32: `BS`) and grow monotonically without
  bound
- Client IDs scale from 3 to 6 digits based on the number of known clients

The scrambling operation preserves length: an n-character payload scrambles to
an n-character result.

## Document Counter

Counters start at 50 (Base32: `BS`) to ensure all IDs have at least 6 characters
total. When a client requests a new ID, its counter increments.

Counters are stored in `~/.lattice.toml` per repository. On re-clone, counters
are recovered by:

1. Scanning all document IDs in the repository
2. Unscrambling each ID to extract counter and client ID
3. For IDs matching this client's client ID, finding the maximum counter
4. Resuming from max + 1

## Client ID

When a client first uses Lattice in a repository, it generates a random client
ID. Client IDs are stored in `~/.lattice.toml` keyed by repository path.

### Selection Algorithm

1. Determine the required ID length based on the number of known clients
2. Generate a random ID by selecting each character uniformly from the Base32
   alphabet
3. Unscramble all existing IDs in the repository to extract their client IDs
4. If the generated ID collides with an existing client ID, discard and retry
5. Repeat until a unique ID is found

### Length Scaling

Client ID length scales with the number of known clients to balance collision
probability against ID brevity:

| Known Clients | ID Length | Possible Values |
|---------------|-----------|-----------------|
| 0-16          | 3         | 32,768          |
| 17-64         | 4         | 1,048,576       |
| 65-256        | 5         | 33,554,432      |
| 257+          | 6         | 1,073,741,824   |

If two clients independently select the same client ID and both commit,
`lat check` detects the duplicate and the later contributor must regenerate
their client ID and renumber their documents.

## Scrambling Algorithm

Lattice applies a bijective permutation to the combined counter + client ID
payload. The permutation uses a Feistel network with hardcoded round keys.

### Why Feistel

A Feistel network is:
- **Bijective by construction**: Every input maps to exactly one output, and
  vice versa, regardless of the round function
- **Good diffusion**: Small input changes produce large output changes (the
  "avalanche effect")
- **Trivially reversible**: Run the same network backward to unscramble
- **Simple to implement**: ~30 lines of code

### Payload Encoding

To scramble an ID:

1. Construct the unscrambled payload: `base32(counter) + client_id`
2. Interpret the payload as a base-32 integer (e.g., "BSABC" → numeric value)
3. Determine bit width: `ceil(log2(value + 1))`, rounded up to even
4. Apply the Feistel permutation
5. Encode the result as Base32, padded to the original payload length
6. Prepend `L`

To unscramble:

1. Remove the `L` prefix
2. Decode the payload from Base32 to integer
3. Apply the inverse Feistel permutation (same network, rounds in reverse)
4. Encode as Base32
5. Split into counter (leading digits) and client ID (trailing digits)

The split point is determined by matching against known client IDs.

### Hardcoded Key

The Feistel network uses hardcoded round constants compiled into Lattice:

```
K = [0x7A3B9F2E, 0x1C8D4E5A, 0x6F2B8C1D, 0x9E4A7F3C,
     0x2D5E1B8A, 0x8C3F6A2D, 0x4B1E9D7C, 0x5A2C8E4B]
```

All Lattice installations use these same constants. This provides:
- **Zero configuration**: No key derivation, no machine IDs, no repo-specific
  state
- **Determinism**: Any client can scramble/unscramble any ID
- **Simplicity**: No failure modes from key management

The constants are arbitrary; their specific values don't matter as long as
they're non-zero and provide reasonable mixing.

### Feistel Structure

For an n-bit input (n even):

1. Split into left half L (n/2 bits) and right half R (n/2 bits)
2. For each round i (0 to 7):
   - L' = R
   - R' = L XOR F(R, K[i])
3. Concatenate final (L', R') as n-bit output

The round function F produces an (n/2)-bit output:

```
F(R, K_i) = ((R * K_i) + (K_i >> 4)) mod 2^(n/2)
```

### Handling Odd Bit Widths

If the payload's bit width is odd, pad with one leading zero bit before
scrambling, then remove it after. This ensures the Feistel network always
operates on an even number of bits.

## Example

Generating an ID with counter=51, client_id="DTX":

1. Unscrambled payload: "BT" + "DTX" = "BTDTX"
2. As base-32 integer: 1×32⁴ + 19×32³ + 3×32² + 19×32 + 23 = 1,678,327
3. Bit width: 21 bits, round to 22
4. Apply 8-round Feistel with hardcoded keys
5. Result (hypothetical): 2,847,193
6. Encode as 5 Base32 chars: "JCQ2X"
7. Final ID: `LJCQ2X`

The next document (counter=52) might produce `LWN5RP`—visually distinct despite
being consecutive.

## Performance Considerations

Unscrambling adds overhead to operations that parse IDs:

- **Single ID**: Microseconds (Base32 decode + 8 Feistel rounds + Base32 encode)
- **Bulk operations**: Scanning 10,000 IDs takes ~10-50ms

This overhead is acceptable for interactive use. For performance-critical paths,
Lattice caches unscrambled ID components in the SQLite index.

## Display and Linking

IDs are displayed in uppercase. In markdown, IDs can be used directly as link
targets: `[text](LJCQ2X)`.

## Errors

- Missing prefix: `JCQ2X` → "Invalid Lattice ID: missing L prefix"
- Too short: `LABCD` → "Invalid Lattice ID: minimum 6 characters"
- Invalid characters: `L01ABC` → "Invalid Lattice ID: invalid character '0'"
- Unknown ID: Link to `LXXXXX` where no document exists

## Security Considerations

The scrambling provides visual distinction, not secrecy. Anyone can:

- Unscramble any ID (the key is public, hardcoded in source)
- Predict future IDs if they know a client's client ID and counter

This is acceptable because Lattice IDs are not secrets; they are stable
identifiers for documents within a repository.
