# Cycle 0001 Insight

## Cohort outcome

Three changes to how Dash moves items between stash and the normal tier were proposed: demote to the stash MRU end (H01), require two stash hits before promotion (H02), and balance promotions between target and probing buckets (H03). Only the two-hit gate (v002) got evaluated. The other two variants (v001, v003) failed because the simulator build hit HTTP 429 rate limits from Maven/Gradle, so the failure was infrastructure, not the policy. There are no Winners.

## Patterns

- The cohort's working theory is that the small normal tier gets flushed by weakly re-referenced items. v002 is the only result, and it supports that theory: gating admission to the protected tier raised hit rate on every trace and size tested. The gain was largest at the larger size, on hm_0 most of all.
- The hit-rate gain cost latency. `get` got slower at both p50 and p99, which contradicts the hypothesis's guess that a first stash hit would be cheaper. `put` slowed too, possibly because stash pressure rises when items stay in stash longer. Benchmark noise has not been ruled out as the cause of the `put` slowdown.
- Reusing the unused `lfu_counter` field kept metadata size unchanged. That makes it a good pattern to follow for cheap per-item state.
- H01 and H03 are still untested. Their failures say nothing about the ideas themselves.

## Open questions

- Is the `put` slowdown real (from stash buckets filling and evicting more often) or benchmark noise?
- Can the two-hit gate keep its hit-rate gain without the extra MRU move on the first stash hit?
- Would H01 (MRU demotion) or H03 (balanced promotion) stack with the gate? H03 claims it lowers cascade cost, which could offset the gate's latency.

## Continuation

Re-run H01 and H03 unchanged, since their ideas were never measured. Build on v002's two-hit gate: try a variant that leaves the item in place on the first stash hit (just set the counter, no MRU move), and try combining the gate with H03's balanced promotion to win back per-op time. Keep the gate, and do not trade away its hit-rate gain for latency.
