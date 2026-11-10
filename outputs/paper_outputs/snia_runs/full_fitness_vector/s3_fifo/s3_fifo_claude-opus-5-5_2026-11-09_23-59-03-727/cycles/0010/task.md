# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant made one targeted change. v001 used stricter ghost→main admission: small-demotee ghost hits went back into small with `freq = 1`. v002 halved main freq on each sweep instead of decrementing it, built on cycle-6 v003. v003 replaced the ghost deque with a fixed `main_size` ring. None beat the parent (cycle-5 v002). v001 and v002 lost summary hit rate, v003 kept hit rate flat but made the `put` tail worse, and v001 also inflated `put` latency. All three are Winners by rank only.

## Patterns

- The ghost handoff is ruled out as the lever for web_0 at 131072. Looser admission (cycle-6 v003) and stricter admission (v001) both regressed that cell. On this parent, a small-demotee ghost hit already confirms reuse. Forcing another probation pass sends long-reuse-distance keys round the ghost → small → ghost loop and takes small slots from new arrivals. v001's only gain came at web_0 16384, where main turns over fast.
- Faster aging in main hurts. Halving evicts high-freq keys whose inter-hit gap is longer than a sweep, and hm_0 at 16384, the cell it targeted, fell. Together with cycle 5's MAX_FREQ gain, this suggests strongly reused keys need more tenure in main, not less. The "dead residency" theory did not hold up.
- The `put` tail is set by hash probes per miss, not by occasional O(n) cleanup. The ring's constant per-insert liveness probe cost more than the parent's rare, amortized `retain`/scan. v001's extra `evict_small` cascade and ghost churn also raised the tail. Changing the shape of the ghost container is not the lever.
- Per-entry metadata stayed flat again. All diffs fit inside the existing 24-byte entry.

## Open questions

- After probation length, small share, ghost bound and both directions of ghost admission have all failed, is web_0's large-size gap simply out of reach for this policy family?
- Would small-cache hm_0 respond to giving strongly reused keys more retention, for example protecting keys at the freq cap, or to a small/main split that depends on cache size?
- Can the miss path drop a hash probe, for example by tracking ghost liveness without a separate `ghost_index` lookup or by storing fingerprints?

## Continuation

Keep cycle-5 v002 as the parent. Drop all ghost-admission variants, faster freq aging, and ghost container reshaping. For hm_0 at 16384, test levers that extend main tenure for highly reused keys, or a split that depends on cache size. Treat web_0 at 131072 as low priority unless a genuinely new mechanism appears. For timing, target fewer hash probes per miss. Keep eviction work bounded, and give no timing credit to policy-only diffs.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002. v001 raised `MAX_FREQ` from 7 to 15, v002 gave graduating keys `freq = 1` instead of 0, and v003 keyed the ghost structures by 32-bit fingerprints. None beat the parent. v001 and v002 lost summary hit rate, including on their target cell, hm_0 at 16384. v003 kept hit rate identical but made `put` slower. All three are Winners by rank only.

## Patterns

- Both levers for main tenure are now used up from both directions. On freq aging, halving (cycle 7) hurt and a cap of 15 hurt, so 7 with decrement-by-one is the local optimum. On graduation credit, full carry-over (cycle 3) hurt and a single credit hurt, so resetting to 0 is right. More tenure that keys have not earned in main, whether old hot keys keep it or newly graduated keys get it, blocks keys that are actually reused in a small, fast-turning main. Cycle 7's call for "more tenure" was only half right. What matters is tenure earned in main, not more of it.
- 131072 stayed flat again under every policy change. Main retention at the large size is saturated, and all of the remaining hit-rate room is at 16384.
- `put` p99 rose in all three variants. In v001 and v002 the cause was longer sweeps: more decrement-and-reinsert passes before a victim is found. In v003 it was extra hashing. The tail tracks work per miss, which is hash computations and sweep steps, not bytes touched. Making the ghost footprint smaller saves nothing when the keys are already 8 bytes and cheap to copy.
- Per-entry metadata stayed flat. Every diff fit in the existing 24-byte entry.

## Open questions

- Would a fingerprint ghost table with an identity/pass-through hasher, so the fingerprint is the hash and there is one hash per miss, finally beat the parent on `put`?
- Is there room left at hm_0 16384 for a lever that is neither tenure nor credit, for example a small/main split that depends on cache size, or should this policy family be treated as converged on hit rate?

## Continuation

Keep cycle-5 v002 as the parent. Drop all freq-cap, aging-rate, and graduation-credit variants, since each has now failed in both directions. On hit rate, the only untested lever is a small/main split that depends on cache size, aimed at 16384. If that fails, treat hit rate as converged. On timing, retry fingerprints only with a pass-through hasher, or target removing a hash or sweep step per miss. Keep eviction work bounded, and give no timing credit to policy-only diffs.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002. v001 used a small share of 1/5 only at capacities up to 32768. v002 merged ghost-expiry probe pairs into `entry()` calls. v003 replaced the main queue's pop/push sweep with an equivalent CLOCK ring. v001 is the first variant since cycle 5 to beat the parent on summary hit rate: hm_0 and web_0 both rose slightly at 16384, and 131072 was unchanged by construction. v002 and v003 kept hit rate bit-identical but made the `put` tail worse.

## Patterns

- Gating the change on cache size worked. Because the split only changes below the threshold, it cannot lose at the saturated 131072 cells, so any small gain at 16384 comes through cleanly. Together with cycle 5's global 10% (hurt web_0) and cycle 6's larger share (hurt everywhere), the small share at 16384 looks like it has a sweet spot near 1/5, not a monotonic trend. The gain was small, so this lever is close to used up.
- Both timing variants missed the mark in instructive ways. Cutting a probe through std `entry()` added a hidden `reserve(1)` on the many stale-tuple misses, which can set off occasional rehashes, a classic p99 cost. The CLOCK ring showed that a sweep step's cost is the slab entry access and the freq decrement, not moving the queue index. Container shape is now ruled out for both the ghost (cycle 7) and main (this cycle).
- The `put` tail tracks the number of sweep steps and rare expensive map events. It does not track per-step bookkeeping or probe counts on the ghost-expiry path. Removing a nominal operation does not help if the replacement API hides extra work.
- Hit-rate-preserving refactors reproduced the parent exactly. That confirms the eviction-order and ghost-liveness semantics are well understood, so pure implementation variants can be trusted not to drift.
- Per-entry metadata stayed flat. Only v003 grew the top-level struct, slightly.

## Open questions

- Is 1/5 the optimum at 16384, or would a nearby value (for example 1/6) or a threshold at a different capacity add a bit more without costing web_0?
- Can the ghost-expiry path drop a probe using calls that never reserve (keep `get`, then call `remove` only on a match), or is that path just not a tail driver?
- Since sweep length sets the tail, can the number of sweep steps per miss be capped without changing eviction order in practice, for example by bounding the steps and falling back to the current hand position?

## Continuation

Promote cycle-9 v001 (the size-gated 1/5 split) as the new parent. On hit rate, at most one fine-tuning probe of the share or threshold at 16384 is worth running; after that, treat hit rate as converged. Drop container reshaping of main and ghost and any `entry()`-based probe merging. On timing, target the number of sweep steps per miss or the rare rehash/`retain` events, keep eviction work bounded, and give no timing credit to ~1 ns shifts from policy-only diffs.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
