## Recurring wins

- (One cycle so far, so these are early signals, not confirmed patterns.)
- Shrinking node metadata wins reliably. Two independent routes got there in cycle 0001: removing links (CLOCK) and narrowing them (`u32` indices). Both were Winners.
- Representation-only changes, such as `u32` link compaction, move one axis cleanly and leave hit rate unchanged. That makes them safe to stack under any policy change.

## Recurring failures

- Adding fields to nodes costs metadata. The node layout has no free padding, so a new byte costs a full word unless it reuses existing space, such as link high bits or a sentinel.
- List-maintenance and node-layout changes do not lower get p50. The `HashMap` probe appears to be the main cost of a hit, so changing the list is not a lever for get latency.
- Policy changes tend to move several axes at once with mixed results: hit rate goes up while size or latency goes down.

## Open questions

- Can a frequency or scan filter (an SLRU segment flag) live inside a compact `u32`-index node without the metadata and latency penalty?
- Does an adaptive or larger probation share fix SLRU's weakness at the small cache size?
- Can CLOCK's put tail latency be bounded (capped sweep, amortised hand advance) while keeping its median gain?
- Does a faster hasher or a different map layout finally move get p50?

## Per-policy notes

- **LRU baseline / `u32` compaction:** policy-neutral, with no cost to hit rate. Use it as the default base for new variants.
- **SLRU:** modest hit-rate gains on the MSR block traces, mostly at the large cache size. It is weak at the small size, and its added segment field increased node size and slowed operations.
- **CLOCK (lazy promotion):** dropped the links, so metadata shrank and the median improved. Hit rate rose on hm_0 at both sizes and on web_0 at the small size, with a slight loss on web_0 at the large size. Put tail latency is the open risk.
- **Across policies:** neither frequency-filtering approach clearly beat the other. Their gains depend on cache size and trace.
