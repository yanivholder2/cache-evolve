# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on 0007 v002's stash presence mask and went after its upkeep cost in different ways. v001 used a coarse counting mask, v002 used per-slot fingerprints on the stash bucket, and v003 swapped in a cheap fmix64-style hasher in place of SipHash. v003 is the Winner. It improved every per-op percentile and kept hit rate flat within noise. Both structural fixes repaired `put` but gave back the `get` p50 gain.

## Patterns

- Hashing cost was a first-order term all along. 0007 v001 cut the number of hash calls and p50 did not move. v003 made each call cheaper and p50 dropped. Making each re-hash cheaper also shrank most of the mask-refresh penalty on `put` and on the `get` tail, without changing the refresh structure.
- Mask upkeep affects only `put`. Both O(1)-maintenance variants (v001, v002) fully recovered `put`. Neither recovered `get` p99, and neither kept p50. Making the update cheap does nothing for reads.
- The read fast path depends on a dense, separate mask word with low false positives. v001 shrank the filter, false positives rose, and the filter ended up costing more than no filter at all. v002 moved presence tracking onto the bucket's own line as a byte scan, which was slower than one AND against a compact mask word even though it was more exact. The "extra cache line" that 0007 suspected turned out to be cheap for reads.
- The `get` tail sits on the stash-hit/promotion path, not in the mask refresh. v002 removed the refresh and the tail still got worse, because adding work to the item shift on promotion lands directly on that path.

## Open questions

- With cheap hashing in place, does a structural O(1) mask update still pay off? Any such update must keep the dense 64-bit (or wider) word so the false-positive rate does not rise.
- Is there more p50 left in the common-path hash, for example the segment-level selection, or in the remaining false positives on the mask test?
- Would the cheap hasher also change the trade-offs on the deferral line (0005 v002 lineage), since that line's costs were partly re-hash-driven?

## Continuation

Build on 0008 v003 (fast hasher plus the 0007 v002 mask). Keep the separate dense mask word for normal-hit rejection. Drop shrunken counting masks and in-bucket fingerprint scans as the reject path. If `put` needs to go lower, try O(1) mask maintenance that keeps at least 64-slot resolution. Also look at the cost of the stash-promotion shift for the `get` tail, without adding work to that shift.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on 0008 v003 (the fast hasher plus the dense stash mask), and all three are Winners. v001 added a collision word so most stash-mask clears run in O(1). v002 switched the mask to a k=2 Bloom test in the same u64 word. v003 changed stash victim selection to spare a once-hit LRU item across a two-slot window. The two mask variants improved per-op-time with bit-identical hit rate. v003 is safe, but its effect sits inside noise.

## Patterns

- Cycle 0008's diagnosis held up. If the dense read-side mask word stays untouched, O(1) upkeep recovers `put` without the `get` p50 loss that 0008 v001/v002 suffered. In v001 the read structure was the only difference from those variants, and it was the thing that mattered.
- Within the one-word budget, the false-positive rate is now the main tuning axis for the filter. v002 sped up both `get` and `put` at no metadata or hit-rate cost, which suggests `put`'s pre-insert lookup goes through the same reject test.
- v001 and v002 change different parts of the mask: v001 changes how bits are cleared, v002 changes how keys map to bits. They look stackable, but v001's collision tracking assumes one bit per key. Under k=2, collisions happen more often, so more removals would take the fallback rebuild.
- The `get` tail still sits on the stash-hit promotion path: the item shift, plus the refresh fallback in v001. None of the mask changes fixed it, and v001 got slightly worse there.
- Stash victim choice based on per-item hit state is cheap and harmless. With a two-slot window, though, it almost never fires, so the hit-rate plateau has not moved. v003's timing gains on a change that only adds work are noise. All three variants have one sample each, so tail moves stay unconfirmed.

## Open questions

- Does v001's O(1) clear still pay off when combined with v002's k=2 mapping? It would need per-bit collision tracking so that either bit of a key can trigger the fallback. Is k=3 worth trying, given that fuller buckets shrink the benefit?
- How much of the `get` p99 is the promotion item shift, and how much is the refresh fallback?
- Does widening the stash victim window (evict the LRU-most never-hit slot, protecting each item at most once) move hit rate at all, or is stash eviction simply a weak lever on these traces?

## Continuation

Merge v002's k=2 mask with v001's collision-word O(1) clear on top of 0008 v003. Check that the fallback rate stays low under k=2, and try k=3 as a separate variant. For the `get` tail, go after the cost of the promotion shift directly. Give stash victim selection one more try with a whole-bucket window and a protect-once bound, and drop it if hit rate stays flat. Rerun any result that rests on a small tail change.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
