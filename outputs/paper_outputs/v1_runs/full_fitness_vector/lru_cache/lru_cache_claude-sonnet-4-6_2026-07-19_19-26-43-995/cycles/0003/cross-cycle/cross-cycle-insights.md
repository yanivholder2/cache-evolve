## Recurring wins

- **Structural node compaction preserving the LRU chain** pays off reliably. The u32 sentinel baseline (cycle 0001) improved cache-line utilization and get latency without sacrificing eviction precision, and has held up as the canonical structural foundation across all subsequent cycles.
- **Meaningful policy or algorithmic changes outperform code-level hints.** The 2Q protected-bit-in-prev encoding and the MRU-tail guard both won in cycle 0002 by changing runtime behavior; every attempt at low-level hints (`#[inline(always)]`, `entry()` API, CLOCK) produced the opposite effect.
- **Spare-bit encoding enables policy enrichment at zero node cost.** Folding 2Q queue membership into the `prev` field's spare bit kept the node at 24 bytes while unlocking full two-queue semantics — compaction and richer policy are not mutually exclusive.

## Recurring failures

- **Algorithm-complexity micro-optimizations consistently backfire.** CLOCK hand sweeps introduced cache-line pollution; `#[inline(always)]` on small helpers pushed hot code out of I-cache; the `entry()` API hit borrow-checker walls on the dominant eviction path (occupied `VacantEntry` must be released before `map.remove()`). All three added hidden cost that erased their structural benefit.
- **Composing prior-cycle winners does not produce additive gains.** Stacking 2Q and the MRU-tail guard in cycle 0003 accumulated cumulative branch and path-length overhead that outweighed each individual contribution on the mixed workload. Orthogonal single-path wins do not compose freely.
- **Guard-style shortcuts are workload-conditional, not universal.** The MRU-tail guard's value is entirely determined by its fire rate. On a diffuse workload the guard fires rarely, making the per-call check the dominant cost. Guards should not be carried forward without fire-rate analysis matched to the target trace.

## Open questions

- **Adaptive quota sizing**: can dynamically shifting the protected/probationary boundary based on observed demotion pressure capture the hit-quality benefit of a larger protected budget without permanently paying the higher unlink/relink cost?
- **Key-clone elision on non-`Copy` types**: the eviction-path refactor could not be validated with `Copy` keys (`.clone()` is a register copy). A `String` or `Arc<str>` key benchmark is needed before the idea is confirmed or discarded.
- **Node size floor**: no cycle has directly probed the density ceiling. How far can the node be packed (toward a single cache line per entry) before returns diminish?
- **CLOCK with lazy or segmented sweep**: cache-line pollution appeared inherent to the eager hand sweep; a lazy-sweep or arena-segmented variant has not been evaluated and may recover the structural win.
- **Guard fire-rate engineering**: can the MRU-tail guard be extended to the probationary tail, or reformulated around a condition with a higher fire rate on diffuse workloads, to make it a robust rather than workload-specific win?

## Per-policy notes

- **LRU (u32 sentinel baseline)**: the structural winner of cycle 0001 and the ongoing baseline. Exact ordering precision is valuable on reuse-heavy workloads; compaction that preserves chain traversal (rather than replacing it with a sweep) consistently pays off on the get path.
- **Two-queue (2Q)**: won cycle 0002 via spare-bit encoding with no node growth. Protected-quota expansion in cycle 0003 confirmed hit-rate sensitivity to the protected budget, but quality and throughput trade off monotonically as the quota grows — the lever is real but expensive at the extremes. Adaptive quota sizing is the main open hypothesis.
- **CLOCK**: underperformed in every evaluation. Eager hand sweeps cause cache-line pollution that erases structural gains; single-bit recency signals reduce hit rate on reuse workloads. Deprioritized unless a concrete mechanism to avoid sweep-induced pollution is proposed.
- **MRU-tail guard (shortcut within LRU/2Q)**: a workload-conditional optimization. Won on cycle 0002's specific trace; failed to generalize in cycle 0003 on a more diffuse workload. Should not be reinstated without a fire-rate justification for the target workload.
