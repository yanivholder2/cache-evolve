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
