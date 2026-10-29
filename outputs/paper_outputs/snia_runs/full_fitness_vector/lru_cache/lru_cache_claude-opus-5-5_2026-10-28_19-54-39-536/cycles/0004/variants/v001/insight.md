# Variant v001 Insight

## Helped

- hit-rate (web_0 @ small size only): This is the one trace/size pair where moving away from the fixed 80% protected share paid off. On web_0 at the small cache size, the climber's first move shrinks protected (`direction` starts at -1), which gives probation more room. That supports the earlier finding that probation is undersized at the small size for web_0. The other target the hypothesis named, web_0 at the large size, barely moved.
- metadata-size: `Node<u64,u64>` stays at 24 bytes. All climber state lives in the header, and segment membership still uses the `PROT_BIT` in `prev`. The "no per-node state" constraint held, so the summary dimension did not change.

## Hurt

- hit-rate (hm_0, both sizes; summary): hm_0 lost hit rate at both sizes, and this outweighed the web_0 gain. `adapt` changes `protected_cap` by `step` every epoch and has no deadband or convergence. Each epoch it compares its hit rate with the previous epoch only, so it reads workload-driven swings as the effect of its own step. On a trace where the fixed 80% split was already near-optimal, the split never settles there. It random-walks around it and costs hits.
- per-op-time: get and put p50 rose sharply, and both p99s roughly tripled. Several pieces of new hot-path work add up:
  - `tick()` runs before the map lookup on every get, including misses. It writes the header and adds a branch to a non-inlined call.
  - `on_hit` gains a counter increment on each branch.
  - The single `if` demotion became a `while` loop. While protected drains after a shrink, it does up to 2 unlink/push_back pairs per promotion, and each touches cold protected-LRU nodes.
  - The header now spans another cache line.

  The "one increment + one compare" cost model in the hypothesis did not hold.
- metadata-size (secondary, `LruCache` header): Nine new `u32`/`i8` fields grew the per-cache header noticeably. The summary does not count this, but it adds to the header cache-line footprint on every op.

## Takeaway

A naive Caffeine-style hill-climber does not help this SLRU. It always steps, has no deadband or step decay, and uses a one-epoch comparison. On these traces the noise in each epoch's hit rate is larger than the effect of a 2%-of-capacity change in `protected_cap`, so adaptation gave a small gain on web_0 at the small size and losses everywhere else. Meanwhile, per-op bookkeeping on every get and the lazy multi-demotion drain made the latency regression far larger than any hit-rate gain. Future split tuning should be a static per-size retune, or adaptation sampled outside the get path (for example counted only on misses/inserts) with hysteresis. A per-op epoch counter is not the way to do it.
