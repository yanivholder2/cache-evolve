# Cycle 0004 Insight

## Cohort outcome

All three variants branched from cycle-2's v003. v001 was a state-neutral refactor that took redundant hashing and re-scans out of the promotion branches. v002 made the stash two-choice: it fills a free slot in the neighbouring stash bucket before evicting. v003 rescues stash-eviction victims into free slots in their own target or probing bucket. v002 and v003 both confirmed their hypotheses and are the first hit-rate gains since cycle 2, but both cost clear per-op-time. v001 kept hit-rate exactly the same as intended, but its get-p50 claim was refuted.

## Patterns

- Idle capacity, not admission policy, was what held hit-rate back. Both winning hit-rate levers keep items that would otherwise be dropped by using holes elsewhere: a neighbour stash bucket in v002, a normal-tier bucket in v003. Holes from promotions (`remove_at`) are not refilled fast enough by their own keys. This fits cycle 3's verdict that the tiny buckets are the constraint.
- Neither lever adds metadata. The extra placement choice is derived from the key hash or bucket fullness, so metadata-size stays the same in every variant.
- Retaining more costs latency on both sides of the cache. v002 adds a second stash scan to every miss, because the stash is checked first. v003 makes stash overflow a multi-bucket operation and leaves normal buckets fuller, so every lookup scans more. More residency means more scanning, so hit-rate and get latency are now traded off directly.
- Trimming bookkeeping doesn't move get p50 (v001). The median is set by the common non-promoting lookup path and by the promotion moves themselves. Any p50 win has to come from cheaper scans or from doing fewer moves.

## Open questions

- Do v002 and v003 stack? Both touch the stash-insert path. Rescue would fire only when both stash choices are full, so the hit-rate gains may overlap. The latency costs would add up.
- Can the second stash scan be avoided on misses without losing v002's gain? Options include a cheap filter or hint for `s1`, or the target/probing-first order. Cycle 3 found stash-first better on this lineage, so that order needs re-testing under two-choice.
- Would an independent second hash for `s1` keep more items than the fixed `s0+1` stride?
- Does rescue placement at Front, followed by stash-Back re-entry of cold rescued items, ever start to hurt as the cache fills?

## Continuation

Adopt v002, with v001's state-neutral refactor folded in, as the hit-rate parent. Then test v003's rescue on top of it to see if the gains stack. Next, recover get latency on the two-choice structure without changing cache state: filter or short-circuit the `s1` scan on misses, or make per-bucket scans cheaper. Stop pursuing pure bookkeeping trims on the promotion path, because they don't reach p50. From now on, judge any further hit-rate gain against its per-op-time cost.
