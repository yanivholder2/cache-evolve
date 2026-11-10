# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-59-03-727/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants were single-change children of cycle-5 v002 (MAX_FREQ 7 on the two-filter parent). Each tested one way to fix the cycle's remaining gaps: v001 grew the small queue to 1/3, v002 gave freq-1 small keys one reinsertion into small, and v003 re-admitted main-victim ghost hits with `freq = 1`. None clearly beat the parent. v002 and v003 each gained a sliver in one place and lost a sliver in another, and v001 lost slightly everywhere. All three are Winners by rank only, and none should replace the parent.

## Patterns

- web_0 at the large size slipped in every variant, the cell two of the three hypotheses targeted. Cycle 5's reading that this gap is about probation length is now falsified from both sides. Shrinking small (cycle 5) and growing it (v001) both regressed, so the 1/4 share sits at a local optimum. Extending probation only for freq-1 keys (v002) also failed to help.
- The ghost queue already acts as the second chance for demoted keys. v002's in-small reinsertion mostly duplicated the ghost→main path and pushed out newcomers. v003's extra credit on the ghost→main path helped hm_0 but let stale returners displace live residents on web_0. Whatever web_0 lacks sits in how the ghost hands keys to main, not in what happens inside small.
- hm_0 at the small size is still stuck. Mechanisms that fire only on ghost hits are too rare at 16384 to move it, because the ghost there is short and reuse-gated. v003's gain showed up at the large size instead, the opposite of what it predicted.
- Timing gave the same lesson again. Policy-only diffs showed noise-level shifts. The one structural cost, v002's multi-pop eviction loop, clearly inflated the `put` tail: any eviction path whose work per miss is unbounded is a tail-latency risk.
- Per-entry metadata stayed flat. Both new bits fit into existing padding or tag space.

## Open questions

- Would a stricter ghost→main admission rule for small-demotee ghost hits (for example re-admitting to small instead of main) close web_0's large-size gap? That would be the opposite direction from v003.
- Does the `freq > 1` threshold, or a main-eviction lever that acts on every sweep, move hm_0 at 16384 where ghost-path changes cannot?
- Could v003's large-size hm_0 gain be kept without the web_0 cost, for example by applying the credit only when the ghost hit comes soon after eviction?

## Continuation

Keep cycle-5 v002 as the parent. No cycle-6 variant replaces it. Drop small-queue resizing and any extra rounds inside small, since probation length is ruled out. For web_0, test the ghost→main admission rule as a single variable, preferring stricter admission over looser. For small-cache hm_0, try levers that act on every main eviction, not on ghost hits. The ghost-index shrink for the `put` tail is still untested. Keep eviction work bounded per miss, and give no timing credit to policy-only diffs.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
