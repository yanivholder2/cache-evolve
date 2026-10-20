## Recurring wins

- **Removing work from the hot path.** The gains that held up all deleted work from the per-op path: cheap unkeyed hashing (0001), the fused register-held sketch increment (0004 and 0006), and the xor-fold finaliser (0010, the first change to lower `get` p50). When the critical path is a serial ALU chain, the finaliser's latency matters more than the number of hash calls.
- **Shrinking metadata without changing behaviour.** Compact `u32` links (0002), CLOCK in place of LRU links (0006), a region flag packed into a link bit (0005) and `u8` counters (0009) all cut memory with hit rate identical or within noise.
- **Sketch accuracy with row independence kept.** Conservative update (0003), `u16` counters at twice the width (0004) and single-counting each access (0005) each gave a small hit-rate gain. The returns shrank each time, and this lever now looks used up.
- **Changing eviction order inside the main region.** GCLOCK's multi-life usage counter (0010) raised hit rate with the admission duel left unchanged. It is the first structural hit-rate gain in the run.
- **Behaviour-preserving refactors on CLOCK.** These have kept hit rate identical across several cycles, so the base is safe to refactor.

## Recurring failures

- **Aging and decay of any kind.** This covers halving with 4-bit counters (0002), halving with full-width counters plus conservative update (0004), the doorkeeper's all-at-once clear (0008) and 4-bit saturation without aging (0010). Workload-a rewards stable long-term frequency, and decay or a low cap creates ties that the strict `>` rule resolves against newcomers.
- **Admission windows of any size.** A ~1% window (0005) and a 10% window (0006) both lowered hit rate. Capacity given to unfiltered arrivals goes to one-hit keys.
- **Changing the duel's inputs.** Tie-favouring `>=` (0003), sampling at the LRU tail (0002) and the colder of two CLOCK candidates (0007) were all flat or negative. The strict single-victim duel is still best.
- **Blocked or one-cache-line sketch layouts** (0003, 0009). Correlating the rows inflates cold keys' estimates.
- **Extra fixed per-access work.** Prefetching, extra hash mixes, index recomputation, nibble masking, guard compares and extra struct fields each cost more at the median or tail than the rare-path miss savings they targeted.
- **Optimising rare paths.** Sweep micro-optimisation and fill-phase map tweaks (0007) changed nothing, because steady state seldom reaches those paths.
- **Reducing hash count and replacing the map.** Hash-once and identity-keyed maps (0006, 0008) and index-only custom tables (0006) did not beat SwissTable.
- **Changing sketch footprint as a latency lever.** Widening cost median latency (0008). Shrinking left `get` p50 flat (0009, 0010).
- **Single p99 readings and unexplained median shifts.** These have repeatedly turned out to be bench drift. A gain counts only if a mechanism explains it and it repeats side by side against the parent.

## Open questions

- Do the xor-fold hash and GCLOCK compose? Would interleaving both usage bits in one word per slot keep the hit-rate gain without the tail regression?
- Does a 3-bit usage counter add more hit rate, or do longer worst-case sweeps cancel it out?
- Are there other eviction-order levers inside the main region, now that admission-side levers are used up?
- How much of `put` p50 and the tails comes from map remove/insert plus the sweep, and can that cost be cut without extra hashing or a larger struct?
- Is `get` p50 now at its floor for one SwissTable lookup plus one sketch increment?

## Per-policy notes

- **LRU (0001–0005):** Exact recency order gave the duel no useful information. SLRU or window restructuring added cost and gave no hit rate.
- **CLOCK (0006–0009):** Hit rate matched LRU with smaller metadata. Steady-state sweeps are short because newcomers enter with a clear bit. `put` p99 is sensitive to unbounded sweeps and to work on the rare paths.
- **GCLOCK (0010):** This is the first policy to raise hit rate by protecting residents that get repeat hits. Its costs are an extra bit-plane read-modify-write and longer sweeps, both of which show up in the tails.
- **TinyLFU admission, throughout:** The strict `>` duel against a single victim, with non-decaying count-min counters, held across all ten cycles. Admission is decided by low counts as long as the counter cap sits far above the counts steady state reaches: `u8` works, 4-bit does not.
