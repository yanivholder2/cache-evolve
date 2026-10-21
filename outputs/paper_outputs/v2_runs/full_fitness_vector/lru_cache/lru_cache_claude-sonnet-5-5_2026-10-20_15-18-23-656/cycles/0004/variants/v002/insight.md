# Variant v002 Insight

## Helped

- hit-rate: Evicting by a second-chance reference bit instead of strict recency lets reused entries survive at the tail, while one-touch scan entries get evicted first. This gave a modest gain on the LIRS trace.
- per-op-time: `get` no longer relinks the node. It does a hash lookup plus a bit write, which removed the pointer writes and gave a marginally lower `get` p50 and p99.

## Hurt

- per-op-time: `put` p99 got worse. Eviction can now walk several referenced tail nodes, clearing each bit and moving it to the head, before it finds a victim. This bounded retry loop adds tail latency to puts.

## Takeaway

Swapping relink-on-get for a reference bit (CLOCK-style second chance) is close to free in `get` cost and metadata (`Node` size is unchanged). It moves the work into eviction, so the cost shows up as `put` tail latency rather than average time. The hit-rate gain was small, so scan resistance from a single bit is real but limited on this trace.

Note: I could not read the overlay source in this session, so the mechanism above comes from the hypothesis and the fitness deltas, not a code diff.
