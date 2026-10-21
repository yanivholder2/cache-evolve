## Recurring wins

- Behaviour-preserving efficiency and layout changes are the most reliable gains. Fast hashing (c1), compact `u32` links (c2), CLOCK reference bits replacing the LRU list (c6) and the region flag packed into a link bit (c5) all shrank cost or metadata without moving hit rate.
- Estimate accuracy was the only lever that raised hit rate, and only by small amounts. Conservative update (c3), `u16` counters at 2× width with the byte size unchanged (c4) and a single count per access (c5) each gave small gains, and each gain was smaller than the one before.
- The strict single-victim `>` duel at the door, with non-decaying counters, has outlasted every alternative tried against it.
- Refactors on the CLOCK base keep hit rate identical bit for bit (c7, c8), so the base is safe to change.
- The fused, register-held sketch increment is the only latency gain explained by a mechanism that showed up more than once (c4, c6). It did not carry over in c5, most likely because of bench drift.

## Recurring failures

- Decay or aging in any form: 4-bit aging (c2), halving with full-width counters (c4), and a doorkeeper whose clear acts as abrupt aging (c8). Workload-a rewards stable long-term frequency, and decay removes the margin that keeps scans out.
- Admission windows at about 1% (c5) and 10% (c6). Capacity given to unfiltered arrivals goes to one-hit keys.
- Choosing the victim from several candidates: LRU-tail sampling (c2) and min-of-two CLOCK candidates (c7). It either changes nothing or loosens admission.
- Changing the tie rule so newcomers win ties (c3) hurt, because the strict rule was not over-rejecting.
- Heavy structure: W-TinyLFU window with SLRU (c1). The extra relinking and memory did not pay for itself.
- Sketch layouts that break row independence: the blocked cache-line layout (c3) inflated estimates. Widening past the current size (c8) gained nothing measurable and pushed the sketch out of L1/L2, so median `get` slowed.
- Cutting hash work: the duplicate hash (c6) and hash-once or identity-keyed maps (c8). For `u64` keys the hash is nearly free, and any added guard or branch costs more than it saves.
- Optimising rare paths: the CLOCK sweep (c7) and the fill-phase double probe (c7). Fixed per-call overhead and struct growth beyond two cache lines outweighed the savings.
- A custom index table storing only node indices (c6) did worse than SwissTable, because every probe must load from `nodes`.

## Open questions

- Is workload-a's hit rate at its ceiling for strict-duel TinyLFU on CLOCK? Every admission, aging, window and sketch-input change since c3 has been flat or negative.
- Can a smaller or denser sketch cut per-access memory traffic, the main remaining `get` cost, while keeping hit rate within noise? Options include narrower counters, fewer rows, or a cache-line block that keeps the rows independent. The c3 blocked layout hurt, so this is unresolved.
- Where does `put` p99 on CLOCK come from? Candidates are the full-miss map work (get, remove, insert) and the eviction path. Count how often each path runs before targeting it.
- Would an index storing hash tags or home slots beat std `HashMap`, or is the map not a bottleneck?
- Benchmark noise is the main obstacle to evaluation. Per-op p99 and small median differences have repeatedly moved with no cause in the code (c2–c8). Only gains that have a mechanism and repeat side by side against the parent should count.

## Per-policy notes

- **TinyLFU + LRU (c1–c5):** Hit rate is capped by sketch accuracy, not by list structure or exact recency order. The per-hit relink cost was never the median bottleneck.
- **W-TinyLFU (window + main):** Every size tried (about 1%, 10%, with or without SLRU) lowered hit rate on workload-a. Treat it as closed unless the workload changes.
- **TinyLFU + CLOCK (c6–present, current base, width factor 8):** Hit rate is the same as LRU with less metadata. The sweep is rarely long because newcomers enter with the reference bit clear. Keep the hot struct layout small, and avoid adding fields or multi-candidate victim choice.
- **CountMin sketch (all policies):** Conservative update, `u16` saturating counters and one increment per access are settled choices. Accuracy gains are used up. The sketch's footprint is now the cost to work on, not its accuracy.
