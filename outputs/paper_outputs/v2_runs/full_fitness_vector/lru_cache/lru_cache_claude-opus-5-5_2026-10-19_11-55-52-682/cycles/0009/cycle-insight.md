# Cycle 0009 Insight

## Cohort outcome

All three variants branched from cycle 0008 v002, which ghosts main evictions. v001 halved the ghost on top of it. v002 added a TinyLFU count-min duel at small-queue exit. v003 moved values into the map buckets. All three are listed as Winners, but only v001 is non-negative: a sliver of hit-rate gain with no cost. v002 lost on every axis. v003 left hit-rate unchanged and slowed both `get` and `put`.

## Patterns

- Ghost tuning is finished. The half ghost and main-eviction ghosting interfere rather than stack. Both make the ghost more selective, so combining them mostly adds collision dilution. The hypothesis's own exit condition is effectively met.
- The first structurally new admission signal, frequency, also failed to break the plateau. It reduced hit-rate because the duel ran the SIEVE hand speculatively and wore down main's VISITED protection. The sketch also undercounted hot main keys while overcounting churny probationary ones. The ghost already captures most of the returning-key signal on workload-a. This is more evidence that the plateau belongs to the trace, not to the admission policy.
- The hit path's floor is the map probe, not the value line. At this capacity the 16-byte arena stays cache-resident, so removing the `nodes[idx]` read saved nothing. Widening buckets made every probe, insert and remove slower. Map bucket width is now the latency lever, and it is better made smaller than larger.
- Metadata accounting is still blind to the heap, which made all three results look better than they are: the ghost table, the sketch, and the larger hash table all go uncounted. v003's apparent per-node halving only moved bytes into the map.

## Open questions

- Would frequency admission work if it only peeked at the victim, without clearing VISITED bits, and counted main hits on every hit? Or is workload-a simply unable to reward any admission signal?
- Can the map bucket shrink below key+`u32`, for example with a fingerprint-indexed or open-addressed table keyed into the arena?
- Does the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for eight cycles, and it is now the main threat to every conclusion above.

## Continuation

Make cycle 0009 v001 the parent, and treat ghost size and inputs as closed. Drop value-in-bucket layouts and drop the TinyLFU duel as specified. Revisit frequency only with a non-destructive victim peek and faithful hit counting, and only after it is shown to matter on a second trace or cache size. That evaluation should be the next cycle's priority over any further workload-a tuning. For latency, target narrower or cheaper map probes.
