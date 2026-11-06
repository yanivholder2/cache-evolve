# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed ClassicLRU inside the bucket, each through a different lever: where new items are inserted (v001, LIP-style quarter-depth), how hits are recorded (v002, CLOCK reference bit), and which item is picked as the victim (v003, frequency-aware tail window). The Winners, v001 and v002, both gained hit rate by making new one-touch items the first to be evicted. v003's hit rate came out identical to the baseline, which most likely means its change never ran in the hit-rate harness.

## Patterns

- Scan resistance is the hit-rate lever that worked. Both Winners make new items "earn" their place before they are protected from eviction, either by position or by a cleared bit. The gain is biggest with the wide (1024-item) buckets, where each bucket acts most like a full LRU.
- Every variant made put latency worse, especially the tail. Any extra work on the miss/eviction path (mid-bucket insert, a per-item sweep, a window scan plus aging) cost more than the single bulk memmove it replaced or added to. The assumption that "a memmove is a memmove" did not hold.
- Get p50 barely moved in any variant, even when the hit path became a single store (v002). So most of the get cost is the linear key lookup across the wide bucket, not the reordering.
- The `name() == "ClassicLRU"` string compare on every op appeared in two variants and adds overhead to the put path that has nothing to do with the policy.
- None of the variants changed metadata-size. Reusing `lfu_counter` and changing only the insert position are both free on that axis.

## Open questions

- Does the hit-rate harness actually run the overlaid bucket `evict_item` / ClassicLRU code paths? v003's bit-identical result needs checking before frequency-aware victim selection is judged.
- Does v001's quarter-depth insert combine with v002's cleared reference bit, or do they overlap and give the same scan protection twice?
- Should the insertion depth scale with bucket width, given the weak gain at the smaller size?

## Continuation

Build on the scan-resistant insertion idea (v001/v002). Tune or scale the insertion depth, and decide the policy check once at construction instead of comparing strings on every op. Target speed directly at the linear `get_position` scan (for example a tag array or per-bucket key index) rather than at reordering cost, and avoid unbounded per-item eviction sweeps. Before retrying v003's frequency-window idea, first confirm that the harness runs the overlaid eviction path.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants refined last cycle's scan-resistant policies: bimodal insertion on top of LIP (v001) and a 2-bit GCLOCK with a bounded sweep (v002). The third variant (v003) went after the lookup scan with a dense key mirror. v001 is the only Winner, and only just: its hit-rate summary edged up because of one cell (hm_0 at the 128-wide size), while it lost slightly on web_0 and put latency got worse.

## Patterns

- Hit-rate gains from changing the replacement rule are now small, and each one trades between workloads. BIP's blind MRU admission gives back scan resistance on web_0. Extra GCLOCK levels add only a little hit rate, though they did so at every point. The easy scan-resistance gains were taken in cycle 0001.
- Put latency is still set by the mechanism, not by policy dispatch. Removing the per-op string compare (v001) had no visible effect, because the mid-bucket memmove dominates. Capping the sweep (v002) and scanning dense keys (v003) both helped the put tail. Each also made the typical put slower, through more full-window sweeps or writes to two allocations.
- The get p50 summary got worse in both speed-oriented variants. In v003 the cause was clearly locality: a separate key Vec adds a second memory stream to every hit, and on short scans that costs more than density saves. Splitting hot data across allocations is the wrong shape of fix.
- Tail and median now move in opposite directions. Changes that bound worst-case work (sweep cap, dense absence scan) improve p99, and every one so far has added per-op cost at p50.
- Metadata-size stayed flat again. v003 showed that this dimension cannot see memory held in side arrays inside the bucket.

## Open questions

- Is v002's small get p50 regression real or noise? Its get-path change was tiny.
- Can a period that scales with bucket size, or a reuse-aware MRU admission signal, keep BIP's small-bucket gain without losing web_0?
- Would a per-slot fingerprint or tag array stored in the same cache line, or a dense reference-bit array for the sweep, cut get p50 where a separate key Vec did not?

## Continuation

Keep v002's 16-slot sweep cap and clear-on-insert as the base mechanism for the put tail. Treat further replacement-rule tweaks as low yield unless they target a specific workload (for example, BIP admission that scales with bucket size). Spend most of the next cycle on get p50 with layouts that keep the key and the hit-time data together (inline tags or fingerprints, SIMD-friendly blocks), and drop the separate-Vec key mirror. For LIP-style insertion, look for a cheaper insert-at-depth rather than trimming dispatch overhead.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
