# Cross-Cycle Insights (through cycle 0006)

## Recurring wins

- **Moving away from strict LRU recency order** improved hit-rate in cycle 0001 with all three alternatives tried. Since then, the hit-rate gains have come from quick demotion of new keys plus lazy promotion of reused keys (SIEVE, cycle 0002) and from a probationary queue with ghost bypass (S3-FIFO, cycle 0005).
- **Changes to representation alone that keep eviction order the same** have been the most reliable gains (cycles 0003–0006): `u32` links, a singly-linked queue, packed side arrays and key swap. They have never cost hit-rate, and they usually improve metadata.
- **A hit path that never relinks**, using hands or bits instead of moving nodes, has kept get cheap in every hand-based policy since cycle 0001.
- **Putting policy state in padding bytes** (`u8` freq, `bool` visited) adds state at no metadata cost.
- **Cheap hashing (Fx-style) was the main speed lever** once node layout stopped moving per-op-time. It helped get once, and helped the put tail and struct size more (cycles 0004–0005).
- **Structural policy fixes can also help the tail.** S3-FIFO's O(1) evictions from the small queue improved put p99 even though each miss did more work.

## Recurring failures

- **Stacking scan-protection mechanisms.** BIP inside SLRU probation (cycle 0002) shrank the admission window, so new keys had less chance to earn a second touch.
- **Tuning in-queue reuse signals has hit a local optimum.** Deeper counters (GCLOCK, a 2-level SIEVE counter) and sweep caps cost hit-rate and/or put-tail latency (cycles 0002–0003).
- **Ghosts without a probationary queue** (a single-queue SIEVE graft, cycle 0004) lost hit-rate and put latency.
- **Widening S3-FIFO admission.** A larger small queue and bigger or more associative ghosts (cycle 0006) both took capacity from main and cost hit-rate plus put latency. Main capacity for proven-reuse keys is the scarce resource.
- **Hand-rolled key-less hash tables** (cycle 0005) lost to hashbrown, badly at the put tail, because buckets cannot be rejected without dereferencing the arena.
- **Node-shape work for speed** ran out once the hit path no longer relinks. It is now a metadata lever only. Side arrays can even hurt get by splitting a hit across two cache lines (cycle 0006).
- **Blaming the hand sweep for the put tail** was refuted in cycle 0003. Per-miss map work (hashing, probes, key clones, ghost lookups) dominates.

## Open questions

- **Generalization.** Every result is from workload-a at size 2048. A second cache size or trace has been requested every cycle since cycle 0001 and is still missing. This is the biggest risk to the lineage.
- Can VISITED stay inline with the value while only the links move out, keeping the memory win without the extra get cache-line touch?
- Would a *smaller* ghost or softer ghost admission help? Softer admission could mean entering main unvisited, behind the hand. Separately, would split points just below or around 10% (1/8, 1/12) beat the current configuration?
- Can hashbrown's entry or raw-table API give a single-probe miss path, and does that lower put p99 on S3-FIFO?
- Metadata accounting still misses heap side structures (ghost table, side arrays, `Vec` headers). Per-node wins overstate the real savings.
- How much of the reported speed gains come from benchmark noise? This has been flagged since cycle 0001 and never quantified.

## Per-policy notes

- **LRU (baseline):** order-only tweaks were dropped after cycle 0001.
- **SLRU:** won in cycle 0001 through `u32`-compact nodes and second-touch promotion. Adding BIP insertion to probation hurt it. If revisited, tune the probation size or admission, not the insertion position.
- **CLOCK / GCLOCK:** the second-chance bit gained hit-rate unexpectedly in cycle 0001. A deeper credit counter buys some hit-rate but reliably hurts the put tail through longer sweeps.
- **BIP:** helped as a standalone insertion change in cycle 0001 and interfered when layered on SLRU. Not pursued further.
- **SIEVE:** one queue and one bit matched or beat multi-segment policies. The lineage refined it through a singly-linked layout, the Fx hasher and a packed side array. Its in-queue knobs are exhausted. It is the base for the S3-FIFO main queue.
- **S3-FIFO (current hit-rate parent, cycle 0005 v003):** a small FIFO, a main SIEVE and a direct-mapped fingerprint ghost. This is the first policy change since SIEVE to raise hit-rate, and the gain was modest. The ghost's bounded, collision-limited reach works as a useful filter. The put path is limited by hashing and ghost probes on each miss. Its side-array variant (cycle 0006 v002) saves memory at a get cost.
