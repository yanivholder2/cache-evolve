## Recurring wins

- Changes that only alter layout or metadata, with no new hot-path work, are the most reliable gains. Examples: fast hashing (c1), compact `u32` links (c2), the region flag packed into a link bit (c5), CLOCK ref bits (c6), and `u8` counters (c9).
- The only levers that have raised hit rate are those that make cold-key estimates more accurate without coupling the rows: conservative update (c3), wider `u16` counters at the same byte size (c4), and counting each access once (c5). Each gain was small, and the lever is now used up.
- Behaviour-preserving refactors on the CLOCK base keep hit rate exactly the same every time. That makes the base safe for efficiency work.
- Counters only need a small range. Narrowing them (`u16`, then `u8`) never changed a duel, because admission is decided by low counts. Shrinking the sketch helps `put` more than `get`, since a full-miss `put` walks the sketch twice.
- Holding the sketch increment in registers in a single pass is the only `get`-path latency gain with a clear mechanism behind it (c4, c6). Whether it carries over has been inconsistent (it did not in c5).

## Recurring failures

- Aging or decay in any form lowers hit rate on workload-a: 4-bit aging (c2), periodic halving with full-width counters (c4), and the doorkeeper's all-at-once clear (c8). The workload rewards stable, non-decaying frequency.
- Admission windows of any size hurt: about 1% (c5) and 10% (c6). Any capacity given to unfiltered arrivals goes to keys that are seen once.
- Changing the duel hurts or does nothing. Ties favouring the newcomer (c3), LRU-tail victim sampling (c2), and taking the minimum of two CLOCK candidates (c7) all loosened admission. Strict `>` against a single victim remains the best rule.
- Blocked or one-cache-line sketch layouts (c3, c9) break row independence and inflate estimates. Doubling sketch width (c8) added footprint without a measurable hit-rate gain.
- Adding fixed per-access cost to save on rare paths loses at the median. Failed examples: prefetching, extra hash mixes, recomputed indices, mask math, extra struct fields, word-scan sweeps, fill-phase tweaks and hash-once maps (c6–c9).
- Reducing hash count does not move latency for `u64` keys (c6, c8). An index table that stores only node indices does worse than SwissTable (c6). Heavy list restructuring such as SLRU (c1) did not pay off.
- Single-run latency readings, especially p99, are unreliable. Several variants with no code-level cause sped up or slowed down on every op.

## Open questions

- Would packed 4-bit counters, which saturate at 15, keep hit rate exactly the same while cutting metadata further? Or would the nibble masking cost more at the median than the smaller footprint saves?
- Is `get` p50 at its floor (one SwissTable lookup plus one sketch increment)? It has not moved under changes to relinking, hashing, sketch size or sketch layout.
- Is workload-a's hit rate at its ceiling for strict-duel TinyLFU? Every admission, window, aging and sketch-accuracy change since c5 has been flat or negative.
- Does CLOCK's `put` p99 come from the full-miss map path (probe, then remove and insert) rather than from the sweep?

## Per-policy notes

- **TinyLFU admission / CountMin sketch:** Use conservative update, non-decaying saturating counters (now `u8`), independent rows, and count each access once. The sketch fits in L1/L2 at the benchmark sizes, so cache-miss-oriented sketch work has nothing left to win.
- **LRU (early base):** Exact recency order gave admission no useful information. Restructuring it (SLRU, windows) added cost without adding hit rate.
- **CLOCK (current base since c6):** Hit rate matches LRU and per-entry metadata is smaller. Long sweeps are rare in steady state, because newcomers enter with a clear bit. Keep the hand and victim choice simple. Keep the cache struct within two cache lines.
- **W-TinyLFU / windowed variants:** Consistently negative on this workload. Don't revisit them without strong new evidence.
- **Evaluation practice:** Count how often each path runs before targeting it. Accept a latency claim only if a mechanism explains it and median results repeat side by side against the parent.
