## Recurring wins

- Segmented LRU (probation + protected) was the one early change that produced a real hit-rate gain. Later cycles kept building on it.
- Raising the protected share helped through about 90%, then flattened near 95%. Fixed-share tuning is exhausted.
- Scan resistance is what pays on this workload. It came from segmentation, from CLOCK's unreferenced insertion, and from a sketch admission gate.
- Simpler designs stay competitive. The doorkeeper-free SLRU + sketch rebuild won, and the compact conservative-update sketch won by a hair.
- Counting rejected misses in the sketch helps. Recurring keys build frequency across retries.

## Recurring failures

- Weakening the protected segment hurt: a smaller share, and ghost hits admitted straight to protected.
- Probation-side tweaks were inert or tied with the parent: demotion placement, probation-only frequency scans, second-chance on protected eviction. Demotions are rare at a high protected share, and some variants were dead by construction because frequency was never persisted.
- Admission-gate refinements gave marginal or negative results: the doorkeeper, not counting rejected misses, a longer aging window, and a deferred-admission window. Stale high counts and under-counted new hot keys cancel the benefit.
- Adaptive (ARC-style) sizing found no phase variance to exploit.
- Approximate-recency shortcuts (skipping relinks) cost a little hit-rate. Their speed benefit was never measured.
- Single-constant tweaks produce only noise-level gains, and "winner" status often rests on a marginal edge.

## Open questions

- Metadata-size and per-op-time have been missing from every fitness vector for eight cycles. Is this a harness fault? Until it is fixed, claimed cost wins (compact nodes, 4-bit sketch, CLOCK's cheap hits, fast paths) are unverified and selection uses hit-rate alone.
- Do any gains hold on other traces or cache sizes? Everything so far comes from one scan-heavy trace at one size. Long-distance reuse and low scan pressure are untested.
- Is the current incumbent (SLRU + compact sketch gate) at a true optimum or only a local one? A different eviction signal might beat it, such as size or cost awareness, or frequency persisting across segments.
- How does CLOCK + sketch compare head to head with the segmented winner, once costs can be measured?
- Is the sketch gate worth its time and memory over plain SLRU?

## Per-policy notes

- Strict LRU: micro-optimizations are neutral on hit-rate and cannot win without cost metrics.
- Segmented LRU: the best-performing line. It is plateaued on protected share, probation placement, eviction scans and ghost lists. Further tuning is low value.
- CLOCK / frequency-CLOCK: a small hit-rate gain over strict LRU, incremental and not transformative. Its main claimed advantage, cheaper metadata and hits, is unmeasured.
- Sketch admission (TinyLFU-style): a marginal gain on top of SLRU. The simple version with rejected misses counted is enough. The doorkeeper, longer aging and window variants add cost for little return. A 4-bit conservative-update sketch is the current compact incumbent, pending cost verification.
- Adaptive / ARC-style: not useful on this trace.
