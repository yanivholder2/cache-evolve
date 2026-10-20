## Recurring wins

- Changes to representation alone, with eviction order identical bit-for-bit, have been the most reliable gains (cycles 3–8). They almost never cost hit-rate. Since about cycle 6 they have stopped being free on latency, so check `get` and the `put` tail on each one.
- Leaving strict LRU order is the main hit-rate lever. The progression has been LRU → SLRU/CLOCK/BIP → SIEVE (quick demotion plus lazy promotion with one bit) → S3-FIFO (small FIFO + main SIEVE + ghost). Each step that paid off added a new structural signal, not deeper per-entry counters.
- A ghost of evicted keys pays off only when a probationary queue sits in front of main. On a single queue it lost (cycle 4). Inside S3-FIFO it gained (cycle 5), and the gain held across node layouts.
- Keeping the ghost's reach short works better than extending it. Direct-mapped collisions and a half-size ghost act as a useful recency filter (cycles 6–8).
- Hit paths that never relink, combined with compact `u32`-index nodes, keep `get` cheap and shrink metadata.
- Cheap hashing (an Fx-style hasher on hashbrown) was the biggest single latency lever, especially for the `put` tail on misses. It now mainly benefits the miss path.
- Flags written on every hit belong in a tiny structure addressed by index (the VISITED bitset). That keeps hand sweeps cheap and avoids most of the side-array `get` penalty.
- Structural policy changes can lower the `put` tail. S3-FIFO's O(1) small-queue evictions did this despite doing more work per miss.

## Recurring failures

- Stacking scan-protection mechanisms. BIP inside SLRU probation shrank the window for a second touch and lost hit-rate.
- Deeper in-queue reuse credit (GCLOCK, 2-level SIEVE counter) hurt the `put` tail and did not gain hit-rate.
- Sweep caps did not fix the `put` tail. Long sweeps were not the cause, and evicting runs of visited nodes early costs hit-rate.
- Tuning S3-FIFO knobs on workload-a (wider small queue, larger or associative ghost, returners placed at the hand) gives noise-sized or negative moves. It is a plateau, and main capacity is the scarce resource.
- Weakening admission after a ghost hit backfires. A ghost hit is a strong reuse signal, so tune recall, not where returners are placed.
- Hand-rolled key-less hash tables lose to hashbrown, because they can't reject a bucket without dereferencing the arena. The loss is sharp at the `put` tail.
- Putting flags where only the key-addressed readers can reach them (VISITED in the map bucket) makes every hand step a hash probe, so the `put` tail regresses.
- Node-shape work aimed at speed stopped paying once the hit path stopped relinking. Layout now moves memory only.

## Open questions

- Does any of the lineage's ranking (SIEVE over SLRU/CLOCK, S3-FIFO over SIEVE, ghost sizing) hold at another cache size or trace? This has gone unverified for every cycle so far and is the largest gap.
- Can latency deltas be trusted? Unchanged code paths have moved by uniform proportions across runs. Latency should be compared within the same run as the parent, with repeated runs.
- Do the half ghost and main-eviction ghosting stack, or does a smaller table get diluted by main victims?
- What structurally different idea could add hit-rate beyond the S3-FIFO plateau? Candidates: frequency-aware admission (TinyLFU-style), adaptive split sizing, LIRS/ARC-style non-resident history.
- Can the miss path get down to one probe through hashbrown's entry or raw APIs without losing tag filtering?
- Metadata accounting ignores heap side structures (ghost tables, side arrays, bitsets, `Vec` headers). Per-node wins overstate the real savings until these are counted per entry.

## Per-policy notes

- **LRU (baseline):** Its strict recency order is a liability on workload-a. Do not tweak it further.
- **SLRU:** It gained over LRU and `u32` links helped it. Insertion position inside probation was the wrong knob. If revisited, change probation sizing or admission instead.
- **CLOCK / GCLOCK:** A hit path with no relinking helps `get`. Deeper per-slot credit costs the `put` tail with no hit-rate payoff. SIEVE superseded it.
- **BIP:** It worked as a standalone departure from LRU (cycle 1) but was counterproductive inside SLRU.
- **SIEVE:** One visited bit looks like a local optimum for in-queue signals. A singly-linked queue and packed side arrays are safe memory wins. A ghost grafted onto a single queue lost.
- **S3-FIFO (current lineage):** Small FIFO + main SIEVE + direct-mapped fingerprint ghost, using an Fx hasher, a VISITED bitset and a half-size ghost. It holds the best hit-rate and has a healthy `put` tail. Its admission and ghost knobs have plateaued on workload-a. Main-eviction ghosting is marginally positive.
