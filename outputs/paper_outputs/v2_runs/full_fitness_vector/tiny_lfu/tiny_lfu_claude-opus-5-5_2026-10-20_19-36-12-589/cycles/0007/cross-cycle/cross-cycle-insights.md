## Recurring wins

- Changes to layout and metadata that leave behaviour unchanged are the most dependable gains. Examples: compact `u32` links (c2), `u16` counters at twice the width for the same bytes (c4), packing the region flag into a link bit (c5) and CLOCK ref bits in place of links (c6). Each one cut per-entry or sketch bytes without lowering hit rate.
- Making the sketch more accurate is the only lever that has raised hit rate. Conservative update (c3), wider `u16` counters (c4) and counting each access once (c5) each added a small gain. The gains shrink each cycle, and this lever now looks used up.
- Latency gains with a real cause come from removing per-op hashing or sketch work. Examples: dropping the per-row SipHash (c1) and fusing the conservative-update increment into one register-held pass (c4, confirmed again in c6). Cheap hashing and the fused increment are now the standard building blocks.
- The strict single-victim `>` duel at the door has held up against every alternative tried. Any change in behaviour should be judged against it.
- Refactors on the CLOCK base that preserve behaviour keep hit rate identical bit for bit, so it is safe to build on.

## Recurring failures

- Sketch aging or decay lost hit rate both times it was evaluated (c2 with 4-bit counters, c4 with full-width counters and conservative update; c1's attempt never ran because of an infra failure). Workload-a rewards stable long-term frequency, and halving wipes out the margin that keeps scans out.
- Admission windows lowered hit rate at all three sizes tried: the c1 window+SLRU, about 1% in c5 and 10% in c6. Capacity given to unfiltered arrivals goes to one-hit keys.
- Using more candidates to pick the victim hurt twice: LRU-tail sampling (c2) and min-of-two CLOCK candidates (c7). Candidate estimates are nearly equal, so taking the minimum loosens admission, and new admits end up evicting each other.
- Changing the tie rule (`>=`, c3) and spreading sketch counters across cache-line blocks (c3) both hurt hit rate. Correlated collisions inflate estimates, and the sketch already fits in cache.
- Adding structure or work to the hot path costs latency straight away: SLRU relinking, region tags, re-hashing on overflow, extra struct fields that push the cache past two cache lines.
- Optimising rare paths does nothing in steady state. That covers CLOCK sweep word-scanning and the fill-phase map probe (c7). A custom hash table that stores only indices also lost to SwissTable (c6).
- Per-op benchmark readings, p99 most of all, drift between cycles. Variants with no cause in the code have often looked faster or slower. A latency change only counts if a mechanism in the code explains it and it repeats when benchmarked side by side against the parent.

## Open questions

- Does the CLOCK base's steady-state `put` cost come from map work on a full miss (a get probe, then remove and insert, each re-hashing)? Would reusing the precomputed hash through a raw-entry API cut it?
- Is workload-a's hit rate at its ceiling for this TinyLFU-family design? Recency structure, sketch accuracy, duel inputs and victim choice all look used up.
- Would an index storing hash tags or home slots beat std `HashMap`, or is the map simply not a bottleneck?
- The unkeyed fast hasher was never checked against adversarial or collision-prone keys.
- Before targeting any path, how often does steady state actually run it?

## Per-policy notes

- **LRU + TinyLFU duel (c1–c5 base):** sampling victims from the LRU tail gave nothing. Exact recency order turned out to carry no information the duel uses. SLRU and windows on top of it cost latency and did not raise hit rate.
- **CLOCK + TinyLFU (c6 onward, current base):** hit rate matches LRU and per-entry metadata is smaller. Sweeps are rare because newcomers enter with a clear bit, so sweep-tail tuning is low-yield. Don't change which victim the hand picks. Keep the fused increment and keep struct fields to a minimum.
- **W-TinyLFU (window + main):** a net loss on workload-a at about 1% and at 10%, even with packed region bits. Treat it as closed unless the workload changes.
- **CountMin sketch:** keep conservative update, saturating `u16` counters, rows that stay independent, one count per access and a fused single-pass increment. Don't use aging, blocked layouts or further micro-optimisation.
