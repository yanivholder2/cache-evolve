# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort split into two families: a bounded second chance on the protected side of the segmented policy (v001), and a representation change that replaces optional arena links with a sentinel index (v002). Both Winners kept hit-rate intact or recovered some of it; v002 won on node size alone, while v001 bought hit-rate back at a visible per-operation cost. A third variant (v003) added a frequency-sketch admission filter, which produced the cohort's clearest hit-rate gain but paid for it on every access.

## Patterns

- Representation changes are the cheap, policy-neutral lever. Dropping the per-link discriminant shrank the node and left hit-rate unchanged, but it did not recover speed, so the `Option` match is not the hot-path cost.
- Arena indexing looks like the real per-operation cost. Both the sentinel change and the admission filter left `get` slower, which points at repeated index access and bounds-checked lookups rather than at link encoding or policy logic.
- Hit-rate gains are coming from what enters or is retained in the cache, not from list order. The admission filter gained hit-rate by rejecting one-hit keys at capacity, and the protected-side rescue recovered part of a prior loss by delaying demotion.
- Anything that runs on every access is expensive in this workload. The admission filter's per-access hashing added cost to both `get` and `put`, and the protected-side rescue's extra pointer work showed up in `get` as well as `put`, even though the hit path was meant to be untouched.
- Attributions are still partly inferred. Both v001 and v003 saw `get` movement the hypothesis did not predict, so the source of that movement is unconfirmed.

## Open questions

- Does the admission filter's hit-rate gain hold beyond workload-a and beyond a single capacity, or is it specific to the scan pattern?
- Can the admission check's hashing be made cheap enough to keep its hit-rate, for example by reusing a hash already computed for the map lookup?
- Is the `get` regression in v001 and v003 real, or is it a layout or noise effect that a re-measure would clear?
- Can the per-access arena lookups be reduced without changing link width?

## Continuation

Next cycle, combine the sentinel-link node reduction with the protected-side rescue only if the rescue's extra pointer work can be moved off the hit path, and re-measure `get` before crediting either change. Pursue admission gating as a hit-rate lever, but drop any design that hashes on every access unless the hashing cost is removed. Stop pursuing further link-encoding changes; the next gains are more likely in arena access patterns.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
