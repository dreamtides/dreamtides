# Model C: Sub-Pool Carousel with Guaranteed Floors

## One-Sentence Player Explanation

Each draft pack draws from rotating archetype pools that naturally ebb and flow, so you always see cards from your archetype but also get tempted by whatever's running hot this run.

## Number of Archetypes: 7

Seven archetypes, drawing on Q1's analysis that N=7 is in the sweet spot where:
- Early-draft diversity is high (88% chance of 3+ archetypes in a random 4-card pack)
- Archetypes are large enough for internal variety (~51 S-tier cards each)
- Convergence is achievable with moderate algorithmic help
- 21 archetype pairs provide strong combinatorial replayability
- Signal reading is meaningful (archetype density is detectable)

## Card Fitness Distribution

360 unique cards distributed across 7 archetypes:

- **Narrow Specialists (35%, 126 cards):** S in 1 archetype, B in 1, C/F elsewhere. The backbone -- clear archetype signals, limited splash value.
- **Specialists with Splash (30%, 108 cards):** S in 1 archetype, A in 1-2 others, B/C elsewhere. The convergence workhorse -- a card that is great in your archetype and good in a neighbor.
- **Multi-Archetype Stars (10%, 36 cards):** S in 2 archetypes, B in 1-2 others. True dual-identity cards at archetype intersections. Expensive to design but create the best draft tension.
- **Broad Generalists (20%, 72 cards):** No S-tier, but A in 2-3 archetypes, B in 2-3 more. Flexible filler that keeps early drafts open.
- **Universal Stars (5%, 18 cards):** S in 3+ archetypes or very high raw power. Rare, highly contested cards.

Each archetype gets ~51 S-tier and ~45 A-tier cards (~96 total S/A, ~27% of pool). Overlap topology is **clustered**: each archetype has 2-3 "neighbors" sharing 15-20 S/A cards and 3-4 "distant" archetypes sharing 3-5.

## Pack Construction: The Sub-Pool Carousel

This is the core innovation. Instead of drawing from a single weighted pool, the system maintains **archetype sub-pools** and uses a **structured 4-slot pack template** that shifts based on draft phase.

### The Mechanism

The 360 cards (with rarity-based copy counts, ~1000 pool entries) are organized into 7 archetype sub-pools. Each card appears in every sub-pool where it has S or A tier fitness (a card that is S in archetype 1 and A in archetype 3 appears in both sub-pool 1 and sub-pool 3). Sub-pools overlap.

**Pack construction uses 4 slots with roles, but the roles rotate and have controlled randomness:**

**Pre-commitment phase (picks 1-5):**
- Slot 1: Random draw from a randomly selected archetype sub-pool ("spotlight slot")
- Slot 2: Random draw from a *different* randomly selected sub-pool
- Slot 3: Random draw from the full pool (any card)
- Slot 4: Random draw from the full pool (any card)

The spotlight slots cycle through archetypes using a **carousel** -- a shuffled sequence of archetypes that determines which sub-pools contribute to packs. The carousel ensures that over picks 1-5, the player sees cards from at least 4-5 different archetypes, without any single archetype dominating.

**Post-commitment phase (picks 6+):**
Once the player has drafted 3+ S/A-tier cards in a single archetype (commitment detected):
- Slot 1: Draw from committed archetype sub-pool ("anchor slot")
- Slot 2: Draw from committed archetype sub-pool OR a neighboring archetype (weighted 60/40)
- Slot 3: Draw from a randomly selected non-committed sub-pool ("splash slot")
- Slot 4: Random draw from the full pool ("wild slot")

**Key properties:** Slot 1 guarantees at least 1 fitting card. Slot 2 provides a second ~80% of the time (60% direct + 20% from neighbors sharing S/A cards). Slots 2-4 all have randomness, so packs are structured but not predictable. Slot 3 ensures off-archetype temptation. Slot 2's neighbor bias naturally supports hybrid decks.

### Depletion

Cards are removed from sub-pools when drafted (not when seen but unpicked). Because sub-pools overlap, drafting a multi-archetype card depletes it from multiple sub-pools simultaneously. This creates emergent signaling: an archetype whose sub-pool is thinning produces weaker Slot 1/2 offerings, signaling that it is being heavily drafted (or was under-stocked this run).

Depletion is gentle -- over 30 picks from a ~1000 entry pool, only ~3% is removed. But because sub-pools are smaller (~200-280 entries each), the per-archetype depletion is ~5-10%, which is enough to be detectable by observant players.

## Variety Across Runs

Three mechanisms create run-to-run variety:

1. **Pool restriction:** Each run randomly suppresses 1-2 archetypes, reducing their sub-pool to 60% of normal size. The suppressed archetypes are still draftable but harder to build around. This forces players into different archetypes across runs.

2. **Carousel ordering:** The pre-commitment carousel (picks 1-5) uses a randomized archetype sequence, so the first few packs spotlight different archetypes each run. Combined with pool restriction, this creates strong early signals about which archetypes are "open."

3. **Depletion cascades:** Because sub-pools overlap, drafting heavily from one archetype thins neighboring archetypes' sub-pools. This creates dynamic, emergent asymmetries that differ each run based on early picks.

The combination of pool restriction (coarse variety) + carousel ordering (early signal variety) + depletion (emergent mid/late variety) provides variety at every timescale.

## Why This Is Structurally Different

Unlike weighted random sampling from a single pool, this design uses: (1) sub-pools as first-class objects with overlapping membership, (2) pack slots with defined roles (anchor, neighbor, splash, wild), and (3) an explicit phase transition from carousel-mode to anchor-mode at commitment detection.
