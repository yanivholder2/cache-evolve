# Variant v002 Insight

## Helped

- per-op-time: Capping the victim search at two probes makes the worst-case work per miss constant, and `put` tail latency came in below the parent's, consistent with the hypothesis's tail target.

## Hurt

- hit-rate: The second probe takes slot `j` as the victim even when its counter is non-zero, after only decrementing it, so a recently hit key sitting there is evicted immediately instead of getting the extra passes the unbounded CLOCK sweep would have given it. This is the mechanism behind the hit-rate loss, and it runs opposite to the hypothesis's expected recovery.
- per-op-time: The `get` median and tail movement is not caused by this diff, since the `get` path (map lookup and saturating `bump`) is unchanged from the parent. The `put` median worsening is also not clearly explained by the code, because the bounded sweep does less work per miss than the parent's loop. Treat these median movements as measurement noise until re-measured.

## Takeaway

Bounding the CLOCK sweep to two probes buys a flat worst-case `put` cost, but it does so by forcing eviction of a referenced key on the second probe, and the hit-rate cost is the price of that. The hypothesis expected the bound to recover hit rate; it moved in the opposite direction, so the bound should be tested with the second probe respecting its counter, not with forced eviction.
