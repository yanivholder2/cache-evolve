# Cross-Cycle Insights

## Recurring wins

- Changes that only alter representation and keep eviction order bit-for-bit have been the safest gains in every cycle from 0003 to 0008. Examples are the singly-linked SIEVE, `u32` links and indices, side arrays, the VISITED bitset and the Fx hasher. They reliably improve metadata, and sometimes latency, with no hit-rate risk.
- Policy changes that pay off are structural, not knob tweaks:
  - Moving away from strict LRU (0001).
  - SIEVE's quick demotion of new keys plus lazy promotion (0002).
  - The S3-FIFO small/main/ghost split (0005), which was also the only policy change to improve the put tail.
- Ghost entries help only when a probationary queue exists for a ghost hit to bypass. A ghost with limited reach (direct-mapped, half-size) works as a useful filter. Shortening its reach has held up as a small gain across two layouts.
- Put flags that are written on every hit where the reader can reach them cheaply. The SIEVE hand and queue pops reach nodes by index, so index-addressed compact storage (the bitset) has been the best overall trade.
- A relink-free hit path combined with cheap hashing (Fx) moved latency more than node layout did. Once the hit path stopped relinking, layout became a memory lever only.

## Recurring failures

- Stacking scan-protection or selectivity mechanisms that overlap. Examples are BIP inside SLRU probation, and the half ghost combined with main-eviction ghosting. They interfere rather than add up.
- Tuning in-queue reuse signals: deeper counters (GCLOCK, a 2-level visited counter) and sweep caps. These cost hit-rate or tail latency. A single VISITED bit is a local optimum.
- Changing admission strength for ghost returners. Placing them at the hand (0007) and pre-setting VISITED on them (0010) both lost hit-rate. Plain tail insertion with VISITED clear is bracketed as optimal.
- Anything that takes effective capacity from main. A wider small queue, a larger or associative ghost, an adaptive split driven by raw counts, and returner credit all lost hit-rate. Main capacity for keys with proven reuse is the scarce resource.
- Hand-rolled or widened hash tables: the key-less custom index, values stored in buckets, and VISITED stored in the map bucket. Each lost to hashbrown's tag filtering, or made probes or hand steps more expensive.
- The TinyLFU duel as specified. It ran the SIEVE hand speculatively, wore down VISITED protection and miscounted hot keys.
- Hash-mixing tweaks. Keys already spread well, so the remaining cost is memory touches.
- Single-queue ghost grafts. Without a probationary queue to bypass, ghost hits pushed out better residents.

## Open questions

- The top unresolved threat: does any of the SIEVE/S3-FIFO lineage hold up at a second cache size or on another trace? It has been unverified for about nine cycles, and the last several Continuations asked for this check.
- Latency attribution is unreliable. Drift between runs is as large as the effects being measured. Compare against the parent in the same run, with repeated runs.
- Metadata accounting does not count heap structures: the ghost table, side arrays, the sketch and map growth. Wins counted per node overstate the real savings.
- Should promoted small keys keep the VISITED bit they earned when they enter main?
- Would ARC-style adaptation work if it were normalized by each ghost source's population? Or does workload-a simply have no phases to adapt to?
- Could frequency admission help if the victim were only peeked at, without disturbing it, and every hit were counted faithfully? Is that only worth trying on a different trace?
- Could a denser index, such as a fingerprint-indexed or open-addressed table into the arena, cut the number of cache lines per probe? The map probe is now the floor for `get`.

## Per-policy notes

- **LRU / SLRU / BIP:** Strict recency order is a liability on workload-a. SLRU probation already isolates keys seen only once, so BIP on top is redundant and harmful. If SLRU is revisited, change the sizing or admission, not the insertion position.
- **CLOCK / GCLOCK:** Removing relinking on a hit is cheap and good. Deeper per-slot credit makes the put tail worse through long hand sweeps.
- **SIEVE:** One queue and one bit gave the best hit-rate among single-queue policies. Neither counter depth nor sweep caps help. Its put tail came from map work on the miss path, not from the hand. The singly-linked form is a free metadata win.
- **S3-FIFO (current lineage):** The current parent is cycle 0009 v001: Fx hashing, the VISITED bitset, a half-size ghost and main-eviction ghosting. The admission and ghost knobs are a flat plateau on workload-a, and ghost tuning is closed. Further hit-rate gains probably need a new structural signal or a new workload. For latency, target narrower or cheaper map probes.
