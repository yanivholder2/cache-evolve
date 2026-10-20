# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007

## Winners

- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes: a flag-only retry in the protected-side demotion branch (v001), hoisted arena index accessors (v002), and a single shared hash for the admission sketch (v003). Only v001 landed as a Winner, and it won on per-operation cost, giving up a small part of its hit-rate in exchange.

## Patterns

- The list surgery in the protected-side demotion path was a per-operation cost, separate from the second-chance idea itself. Keeping the rescued head in place removed that cost, which is consistent with earlier cycles finding that extra pointer work on shared paths is expensive.
- The hit-rate given back by v001 is small and plausibly comes from losing the MRU rotation that protected the rescued head. This makes the protection and the cost the same mechanism, so further gains here will likely trade against it.
- Hoisting index conversions into accessors (v002) did not speed up `get` or `put`, and both got slower. The access-count explanation from cycle 0002 does not hold on its own; codegen or layout effects are the more likely cause.
- Replacing SipHash with a cheaper shared hash (v003) reduced the admission gate's per-access cost on both paths without changing hit-rate on the one trace tested. This confirms that hashing cost, not the gate's decision logic, was the expense flagged in cycle 0003.
- Attributions are still partly inferred. v001's reading rests on the hypothesis text and fitness vectors rather than on the diff, and v002's cause is untested.

## Open questions

- Does the flag-only retry keep its per-operation gain without losing hit-rate if the rescued head is given a different recency approximation?
- Does the cheap admission hash keep its hit-rate on other workloads and capacities, since only one trace at one capacity was checked?
- Is the `get` and `put` slowdown from the hoisted accessors caused by inlining or codegen, and does comparing the parent's inline casts against the helper form in isolation settle it?

## Continuation

Next cycle, build on v001's flag-only demotion and try to recover the lost protection with a cheap recency signal that does not reintroduce list surgery. Keep the cheap admission hash and test it on another workload and capacity before stacking more on it. Drop further arena access rewrites until a codegen-only comparison explains the v002 slowdown.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a bounded credit for rescued protected heads (v001), narrowed arena links and list pointers to `u32` with a sentinel (v002), and feeding the admission sketch only from writes (v003). The two Winners are the representation change (v002) and the credit-based rescue (v001); v003 was not a Winner because its read-path saving came with a hit-rate loss.

## Patterns

- Narrowing the arena links to `u32` with a sentinel is a clean, policy-neutral win on node and struct size. It did not move `get` and may have helped `put` slightly, which is consistent with the cycle 0003 finding that link encoding is not the hot-path cost.
- A two-bit credit in the existing flags byte recovers only a small share of the protection the parent lost when it dropped the list rotation. Protection seems to depend on the head actually being moved, not only on a retained flag.
- Removing the sketch hash from `get` is the first change that cleanly lowers read cost on the admission path. The price is that read-only popularity no longer counts toward admission, and the only measured effect is a small hit-rate loss on one trace.
- Tail-latency movement in both Winners is small and comes from single runs. Credit to the mechanism should wait for a re-measure, especially where the diff does not touch the moved path (v003's `put` change).
- v002 argues that the earlier index-access slowdowns came from accessor structure rather than the casts. That is an inference from one measurement and is not yet isolated.

## Open questions

- Does the credit scheme's protection gap close with a different recency signal, or is list position the only thing that keeps the head safe?
- Does the read-path saving in the sketch change survive on read-heavy workloads, where reads carry most of the frequency signal?
- Is the `put` tail movement in v001 and v003 real, or run-to-run noise?
- Does the `u32` capacity cap in v002 (rejecting very large capacities at construction) matter for any workload the cache is expected to serve?

## Continuation

Next cycle, keep the `u32` sentinel links as the base layout and build on v001's credit only if it can add protection without list surgery on the shared path. Test the read-path sketch change on a read-heavy workload before stacking more on it, and re-measure `put` tails before crediting any mechanism. Drop further link-width experiments and any attempt to recover the lost rotation by moving the head on the hit path.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three different levers: a smaller protected segment (v001), a cheaper hasher for the key map (v002), and a bounded ghost ring of recently evicted keys that promotes returning keys straight to protected (v003). The clearest Winner is the hasher swap (v002), which cut per-operation cost with no policy change. v001 and v003 are listed as Winners, but both insights record a hit-rate or cost trade-off that the Winner label does not capture.

## Patterns

- Hashing on the key map is a real hot-path cost, and it is separate from the admission sketch. Swapping to a zero-state hasher lowered `get` and `put` cost without touching policy, and it also shrank the map struct. This is the same kind of cost that cycle 0004 found in the sketch hash.
- Shrinking the protected segment hurt hit-rate on the scan-heavy trace. The 4/5 share appears to be doing real work, so further reductions in protected share are unlikely to pay off.
- The ghost ring gained scan resistance, but the gain came with added work on the miss path. Its per-miss scan over the ring and its extra hashing both land in `put`, and its heap storage is not counted by the struct-size metric.
- The v001 per-operation movement is not explained by its diff, which leaves the code paths unchanged. Treat the latency delta as unconfirmed until it is re-measured.
- The v003 insight was written from the hypothesis and fitness vectors because its source could not be read. Its code-level claims are inferred and should be checked against the diff before they are credited.
- A fast hasher with a raw finish leaves the low bucket-index bits least mixed. The v002 gain may depend on the key distribution, and it has been shown only on integer keys.

## Open questions

- Does the zero-state hasher keep its speedup and its bucket spread on non-sequential integer keys and on string keys, where the byte path is different?
- Does the protected-share sensitivity seen in v001 hold at other capacities and traces, or is it specific to workload-a at 2048 entries?
- Can the ghost ring's lookup be made cheap enough to keep its scan-resistance gain, for example with a hash-indexed set in place of a linear scan?
- Are the `put` tail movements in v001 and v003 real, or run-to-run noise?

## Continuation

Next cycle, keep the zero-state map hasher as the base and test its distribution on string and non-sequential integer keys before stacking more on it. Pursue the ghost-ring idea only if its lookup is bounded, and report its heap size alongside struct size. Drop further protected-share reductions, since v001 shows the current ratio is not oversized, and re-measure v001's per-operation deltas before crediting them.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
