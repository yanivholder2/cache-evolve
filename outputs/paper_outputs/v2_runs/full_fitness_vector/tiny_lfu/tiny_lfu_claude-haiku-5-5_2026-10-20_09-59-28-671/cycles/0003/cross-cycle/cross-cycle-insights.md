## Recurring wins

- Constant-factor work that leaves eviction order alone is the most reliable source of gains: skipping relinks at the MRU tail, hashing each key once, and trimming pointer work on repeated accesses to the newest key all cut per-op cost without moving hit rate.
- Metadata reduction is decoupled from policy. Packing protected state into existing node fields shrank overhead without changing hit rate, so bookkeeping can be trimmed independently of the eviction logic.
- Counter-index fidelity to the public sketch path matters for admission. Restoring the public per-row seeded indexing recovered hit rate, which suggests the admission gate is sensitive to exact sketch behaviour.

## Recurring failures

- Recency rotation of rejected admission victims lost hit rate and added per-op cost. The frequency gate already does the protective work, so recency adds cost without improving admission.
- Capping the protected segment without demotion lost hit rate. The segmentation benefit appears to depend on the protected set turning over, so caps are the wrong lever.
- Shortening frequency aging lost hit rate. Stale counts do not appear to be what limits admission.
- A one-hit doorkeeper in front of the sketch barely moved hit rate and mostly added state and per-put work. One-hit pollution does not appear to be the bottleneck on this workload.
- Changes that look like speedups or regressions are often attributed to helpers or accessors without verified source inspection. Treat attributions as hypotheses until confirmed.

## Open questions

- Does the marginal hit-rate gain from protected/probation segmentation hold on any trace other than lirs workload-a? Is its latency drop real or measurement variance?
- Can the seeded per-row sketch indexing be made cheaper while keeping identical counter indices, so the fidelity gain survives without the speed loss?
- Are the precomputed-hash helpers index-equivalent to the public sketch methods? Resolving this decides whether the earlier hit-rate drop was a bug or a real trade-off.
- How much of the eviction-order-preserving savings depends on the share of accesses that hit the newest key?
- Is the doorkeeper's memory cost fully reported, and do any of these effects appear on a trace where one-hit keys dominate?
- Do any results survive repeat runs and a second trace? All evidence so far comes from one trace.

## Per-policy notes

- TinyLFU-style admission (frequency sketch and gate): hit rate is sensitive to sketch fidelity and is not limited by stale counts or one-hit pollution on the tested trace. Preserve indexing semantics; avoid shortening aging; avoid adding a doorkeeper unless a one-hit-heavy trace shows an effect.
- Rejected-victim handling: recency rotation of rejected victims is a dead direction.
- Segmented (probation/protected) resident list: the only source of hit-rate movement seen so far, but small, metadata-costly, and dependent on unbounded turnover of the protected set. Re-test on another trace before building on it.
- MRU-tail and newest-key paths: safe targets for constant-factor speedups that keep eviction order unchanged.
