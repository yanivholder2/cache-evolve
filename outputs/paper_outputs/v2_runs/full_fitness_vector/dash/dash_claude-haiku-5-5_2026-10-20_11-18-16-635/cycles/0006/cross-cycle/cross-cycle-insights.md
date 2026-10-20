# Cross-cycle insights

## Recurring wins

- Frequency-aware victim selection with a full scan of the bucket for the lowest-counter resident (cycle 0004) is the only change with a repeated hit-rate gain. Narrowing it (sampling, stash-only) lost the gain, and maintaining the minimum incrementally did not pay, so the full scan remains the base.
- Protecting hot residents at eviction works better than at insert. Insert-side changes that displaced residents tended to lose hit rate.
- The deferred stash-to-target promotion (cycle 0003) keeps its hit-rate gain when its first-hit bookkeeping is made cheaper (cycle 0004).
- Layout and replacement are separable. Repacking records (cycles 0004 to 0006) left eviction order and hit rate unchanged.

## Recurring failures

- Latency claims have not held up in any cycle (0001 to 0006). Per-op gains are often on paths the change did not touch, and put-side movements are unexplained. Winner labels have not tracked measured regressions.
- Insert-position and admission gating lost hit rate: gating promotion on bucket fullness (cycle 0002), blocking first-touch admission (cycle 0003), and ghost-ring re-admission (cycle 0006). Each displaced residents without adding re-referenced hits.
- Recency tie-breaking among equal counters (cycle 0003) moved no hit rate and added eviction cost.
- Stash occupancy fast paths (cycle 0002) raised get and put latency, so empty stash buckets appear rare.
- Cheaper victim searches did not hold up: two-choice sampling (cycle 0005) and an incremental minimum cache (cycle 0006) both lost hit rate or raised put and get tails.
- Packing DashSegment fields (cycle 0006) reduced footprint but added decode work on get and promotion paths.
- The segment repack meant to shrink size (cycle 0005) did not change the segment, so its per-op gain has no mechanism.

## Open questions

- Does full-scan frequency-aware eviction keep its hit-rate gain on a second, non-workload-a trace? This check has been requested since cycle 0003 and has not been run.
- Can the frequency victim be found more cheaply without losing the full-scan ranking? Sampling and incremental caching have both failed so far.
- Is put-path cost from victim search, or from something else? Its source is still unidentified.
- Does the deferred promotion keep its hit-rate gain on a stash-dominant trace?
- Can the packed layout keep its size saving while avoiding decode cost on hot paths?
- Does the ghost ring's byte cost change the picture once counted in metadata size?
- What is the Winner basis for variants with recorded regressions? It is still not visible in the insights.

## Per-policy notes

- **Victim selection (saturating access counter):** Full-scan lowest-counter eviction is the strongest hit-rate result. Sampled and stash-only versions lost it, and the incremental-minimum version added put and get cost. Needs a second trace.
- **Insert and promotion:** MRU insert protects new keys but can displace hot residents in full buckets. Fullness-gated insert and first-touch admission gating lost hit rate. The deferred stash-to-target promotion gained hit rate, and it works when its first-hit state is cheap.
- **Ghost ring re-admission:** Lost a small amount of hit rate on workload-a and added put-path bookkeeping. Drop unless hit rate improves and its metadata cost is reported.
- **Recency tie-breaking:** No effect on hit rate; added cost. Drop.
- **Record layout (not a policy):** Repacks did not change hit rate. Packed fields reduced footprint but slowed get and promotion. Treat as separate from replacement policy and do not credit per-op gains until re-measured.
