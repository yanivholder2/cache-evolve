# Variant v003 Insight

## Helped

- per-op-time: Gating the ghost lookup on a full cache and hashing the victim key once in the eviction path removed redundant hashing from `put`. The gain shows up in put tail latency and only marginally in the median. The aggregate summary did not move.

## Hurt

- (none) hit-rate and metadata-size are unchanged, as the hypothesis intended. Policy behavior was preserved and no per-node state was added.

## Takeaway

Trimming redundant hashing and clones in `put` is a safe, policy-neutral micro-optimization. It only reaches the put tail, because the summary is dominated by `get` (unchanged) and by the median put cost (hash plus map insert). Larger per-op-time gains would need to change the structure of the hot path, such as the number of map operations per put, rather than the number of hashes within a fixed sequence of operations.

(Note: I could not read the overlay source in this session, so the code-level reasoning rests on the hypothesis text.)
