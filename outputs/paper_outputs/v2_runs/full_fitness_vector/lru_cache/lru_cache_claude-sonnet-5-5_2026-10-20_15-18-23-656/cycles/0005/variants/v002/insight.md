# Variant v002 Insight

## Helped

- per-op-time: Probationary hits now only set a reference bit instead of relinking the node, which removes pointer writes from the common get path. Get latency improved at both p50 and p99. Put p50 and p99 also improved, and the K=2 cap on second-chance passes keeps the eviction walk short.
- metadata-size: The reference bit reused the existing flag byte or padding, so the node stayed the same size as the parent.

## Hurt

- hit-rate: Probationary hits no longer promote or reorder nodes. Eviction also gives at most two second-chance passes before it evicts the tail regardless of its bit. Together these make recency tracking coarser, and hit-rate dropped on workload-a.

## Takeaway

Replacing relink-on-hit with a reference bit and a capped second-chance walk moves cost out of the hot paths. It gives up some recency precision, and that precision was what the parent's hit-rate gain depended on. The cap on passes bounds put p99, but it is also what forces evictions of nodes that were recently referenced. Latency and hit-rate trade off directly through K.

(Note: I could not read the overlay source in this session, so the code-level reasoning comes from the hypothesis text and the fitness vectors.)
