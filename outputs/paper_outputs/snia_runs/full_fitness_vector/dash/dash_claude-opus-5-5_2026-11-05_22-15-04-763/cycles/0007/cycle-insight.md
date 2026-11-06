# Cycle 0007 Insight

## Cohort outcome

Two variants made cost-only changes to the `get` p50 gap on 0004 v002: v001 hashes each key once per lookup, and v002 adds a stash presence mask. The third, v003, changed the deferral to depend on whether the stash LRU item is marked (on 0005 v002). v002 is the Winner. It is the first variant to move `get` p50 with bit-identical hit rate, but it costs a lot of `put` and `get` p99.

## Patterns

- The stash probe is now confirmed as the source of the `get` p50 gap. Repeated hashing inside the segment is ruled out: v001 kept hit rate identical and p50 stayed flat. Skipping the stash scan with a mask test lowered p50. Every other common-path suspect from earlier cycles is now eliminated.
- v002's cost is mask upkeep, not the idea itself. Recomputing the mask on every stash removal re-hashes the whole bucket. Stash removals are on the common put/eviction path, so the cost shows up in both `put` percentiles and in the `get` tail. This is the same lesson as earlier cycles in a new form: work moved onto stash churn lands on the hot path.
- Removing extra hashes (v001) trims the stash-promotion tail a little at no hit-rate cost. That makes it a cheap change to stack on others, not a p50 lever.
- The deferral line is close to exhausted. After fill level (0006), the marked-LRU signal (v003) also failed to recover the small-cache hit-rate loss. All it did was change how often the both-full cascade runs, which moved cost between `get` and `put` and gave back some large-cache hit rate. Signals local to a single stash bucket do not separate a harmful deferral from a harmless one.
- Unexplained `put` p99 drift on a path the change doesn't touch (v001) appeared again. Small tail changes still need a rerun.

## Open questions

- Can the mask support O(1) removal, for example with small per-bit counters or with mask bits stored next to the stash bucket metadata, while keeping the p50 gain?
- How much of the remaining `get` tail comes from false positives rather than from the refresh on promotion? Would a wider or two-bit fingerprint change that?
- Does the Dash-level segment hash, which v001 left alone, account for any remaining p50 gap once the stash probe is cheap?

## Continuation

Build on 0007 v002 and make mask maintenance cheap. Removals must avoid re-hashing the bucket and avoid an extra cache line, using counting bits or metadata stored next to the bucket. Folding in v001's single hash is a cheap addition. Drop deferral-gating signals taken from stash bucket state. Return to the deferral line only with a new signal type, and not before the mask's tail cost is fixed.
