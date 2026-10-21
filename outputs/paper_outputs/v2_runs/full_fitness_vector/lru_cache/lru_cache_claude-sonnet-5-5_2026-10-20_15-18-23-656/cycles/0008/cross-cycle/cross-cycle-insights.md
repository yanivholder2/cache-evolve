## Recurring wins

- Policy changes drive hit-rate; mechanics changes (u32 links, hasher, key move, head-touch early return) drive speed and size. The two axes stay largely independent, and mechanics changes preserve eviction order.
- Segmentation gives scan resistance. This is established; do not re-prove it.
- Ghost history helps as an insertion-position filter: cold keys go to the LRU end of probation unless the ghost table remembers them. This was the last real hit-rate gain.
- Ghost overhead can be trimmed without losing the gain: a smaller table and fewer redundant hashes cut put tail latency while hit-rate held.
- Dropping the hash avalanche fold was a real put-speed win on the segmented policy. It came only after the fold was shown to be the cost.
- Capping the second-chance eviction walk cut put p99, and the cost was hit-rate. This is a usable latency/hit-rate dial.

## Recurring failures

- Shifting promotion/demotion work between operations (lazy demotion, reference bit instead of relink on get) only moves the cost. The tail lands on put or eviction.
- Cheap-skip tricks for relinking (head-adjacent skip) fail when few hits land in the skipped case. The added branch costs more than it saves.
- Layout and bit-packing changes stop giving speed once the policy is segmented. They give size only, and sometimes cost latency.
- Promoting ghost hits straight to protected hurt hit-rate. Returning evictees are not hot enough to displace protected entries.
- Constant-tuning (protected share), demotion-position tweaks, put-update semantics, and key-clone or hasher micro-optimizations are saturated or inside noise. Stop spending cycles on them.
- Winner status is weak evidence. Many small per-op-time deltas appeared on paths the change could not touch, so they are noise, and some "Winners" are just noise-tolerant.
- Bundling several changes (fold with layout, hasher with touch path) makes attribution impossible, and authors cannot read diffs.

## Open questions

- What is the per-op-time noise floor? Repeat-run measurement is still missing, and it has been pending for many cycles.
- Does the hit-rate gain from segmentation and ghost history hold on non-scan workloads? This is the longest-pending check.
- Can the number of map operations per put be cut to move the median put, since hash and ghost trims only reached the tail?
- How far can the ghost table shrink or be sampled before hit-rate degrades?
- Can the eviction cap be made adaptive, or a cheaper rule found for which referenced nodes to spare, so put p99 stays low without the hit-rate loss?
- Does a ghost-hit insertion at probation MRU (a middle path) beat both extremes?
- Do u32 links give a speed benefit on the segmented policy at all, or only size?
- Heap-allocated ghost memory is invisible to the per-node size metric, so a heap-inclusive memory measure is needed.

## Per-policy notes

- Plain LRU: u32 sentinel links, a lighter hasher and a key move were pure wins in early cycles, with hit-rate unchanged.
- Segmented LRU (probation/protected): it gives scan-resistance hit-rate at the cost of promotion/demotion relinking on the hot path. Protected-share increases give diminishing hit-rate gains. Demoted nodes are best kept warm, not at the LRU end.
- Segmented LRU with a second-chance reference bit: a small scan-resistance gain and a cheaper get, with work shifted to eviction. A capped walk trades hit-rate for put p99.
- Ghost-gated segmented LRU is the current baseline: a halved ghost table, a single-hash eviction path, and a ghost lookup gated on a full cache. Its remaining cost is put-path overhead and off-node memory.
