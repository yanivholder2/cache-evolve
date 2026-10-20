# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007 v001 (the VISITED bitset). v001 stacked the half-size ghost on top, v002 recorded main evictions in the ghost, and v003 moved VISITED into a spare bit of the map value. All three are listed as Winners. The two policy changes each gave a small hit-rate gain with no real memory or latency cost. The representation change won on the medians and on struct size but lost badly at the `put` tail.

## Patterns

- The half-size ghost gain held on a second node layout, so it is a real policy gain that does not depend on layout. Main-eviction ghosting was also positive, but barely. Together they close out ghost tuning on workload-a: what the ghost reaches back to and what feeds it now matter only at the margin.
- Where a flag lives has to match how it is accessed. The hit path reaches entries by key, but the SIEVE hand and small-queue pops reach them by index. Putting VISITED in the map bucket helped the readers that come in by key and made every hand step a hash plus a scattered probe. The `put` tail is set by the eviction scan, so index-addressed flag storage (the bitset) is still the better overall trade.
- The hit path is close to its floor. With the third structure gone, the gain was sub-nanosecond: one map probe plus one value line now accounts for nearly all of `get`.
- Latency measurement is still unreliable. v001's code left the `get` path untouched, yet its latencies all slowed by about the same proportion. Cross-run latency deltas cannot currently be attributed to a change.

## Open questions

- Do the half ghost and main-eviction ghosting stack, or do they interfere? A smaller table makes dilution from main victims worse.
- Can the bucket-flag `get` win and the bitset's cheap hand both be kept, for example by mirroring the flag or by having the hand sweep touch fewer nodes?
- Does any of the S3-FIFO lineage hold at another cache size or trace? This has gone unverified for seven cycles.

## Continuation

Use cycle 0008 v001 (bitset plus half ghost) as the parent. Optionally fold in v002's main-eviction ghosting as a single combined check, then stop probing the ghost. Drop map-bucket VISITED unless the hand gets index-addressed access back. Before more micro-tuning, put latency comparisons in the same run as the parent and evaluate at a second cache size or trace. Further hit-rate gains will probably need a structurally different idea, not more S3-FIFO knob changes.
