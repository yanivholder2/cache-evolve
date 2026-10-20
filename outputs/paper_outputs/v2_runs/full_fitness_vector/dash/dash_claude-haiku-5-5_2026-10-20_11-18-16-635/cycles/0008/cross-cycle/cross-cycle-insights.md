## Recurring wins

- Full-scan, non-decaying, saturating access counter as the victim-choice base. It recurs as the only hit-rate gain (cycles 0002, 0004, 0007), and every attempt to narrow or cheapen the scan lost that gain (cycle 0005).
- Deferred stash-to-target promotion keeps its hit-rate gain when its first-hit state is cheap (cycles 0003, 0004). The cost lives in the state, not in the deferral.
- Reordering `get` to probe target and probing buckets before the stash removes stash-scan cost from the common hit path (cycles 0001, 0008), with hit rate and metadata unchanged.
- Layout changes and replacement policy are separable: packing or repacking records shifts footprint without changing eviction order (cycles 0004, 0006, 0007, 0008).

## Recurring failures

- Insert-position changes lose hit rate. MRU promotion and gating promotion on bucket fullness both traded hit rate away (cycles 0001, 0002). Blocking first-touch admission also lost hit rate (cycle 0003).
- Shortening or cheapening the victim search does not lower put latency. Sampling, early exit, and an incrementally maintained minimum all failed to bring puts toward the parent (cycles 0005, 0006, 0007). Keeping the minimum up to date costs more than the scan it replaces.
- Counter decay lowers hit rate, so the never-decaying counter is doing the retention work (cycle 0007).
- Ghost-ring re-admission loses a little hit rate and adds put-path bookkeeping with no offsetting gain (cycle 0006).
- Stash occupancy bookkeeping on the get and write paths raised latency, which suggests stash buckets are rarely empty on these traces (cycle 0002).
- Recency tie-breaking among equal counters moves nothing and adds eviction-path cost (cycle 0003).
- Packed records and packed counters shift cost onto hot paths. Decode on get and promotion, and counter updates on hits, regress latency even when footprint drops (cycles 0006, 0008).
- Per-op latency attributions repeatedly outrun their mechanisms. Get and put tail movements are often attributed to noise or to paths the edit does not touch (cycles 0001 to 0008). Treat unconfirmed latency gains as unproven.

## Open questions

- Does the full-scan frequency-aware victim choice keep its hit-rate gain on any trace other than workload-a? This check has been deferred since cycle 0004.
- Does the packed-counter plus packed-layout combination (cycle 0007 v003) hold its partial hit-rate gain on a second trace?
- Where does the stash-hit get tail sit once it is reported separately? The reordered `get` win is only partly known until then.
- Is the packed counter stored in spare bits or a packed slot, and can the hit-path counter stay unpacked while cold fields are packed for metadata savings?
- What is the ghost ring's byte cost, and is it counted in metadata size?
- Why are variants that record regressions and no helped dimensions still labelled Winners? The Winner basis is not visible in any cycle insight.

## Per-policy notes

- Insert and promotion: MRU insert, fullness-gated insert, and first-touch admission gating have all lost hit rate. The deferred stash-to-target promotion is the only insert-side change that has gained hit rate, and only when its first-hit state is cheap.
- Eviction (victim choice): the full-scan, non-decaying saturating counter is the only consistent retention gain. Sampled, stash-only, decayed, and incrementally cached variants all fall short of it. Recency tie-breaking has no effect. The per-hit increment size was flat on workload-a (cycle 0008).
- Eviction (ghost or re-admission): ghost-ring re-admission lost hit rate and added put cost. Not a current candidate.
- Layout and metadata: packing reduces footprint and is hit-rate neutral, but has not yet produced a net latency win because of decode and counter-update cost on the hot path. Repacking alone did not change segment size in cycle 0005.
- Get path (lookup ordering): skipping the stash scan on common target and probing hits is the clearest latency change so far. Its effect on stash hits is still unmeasured.
- Across all policies: every hit-rate signal so far comes from workload-a. Stash-dominant and scan-heavy traces, requested since cycle 0003, have not been measured.
