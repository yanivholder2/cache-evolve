## Recurring wins

- Compact `u32`-index nodes are a safe base. They change no policy behaviour, shrink metadata with no side effects, and policy variants built on them keep the size savings.
- Storing policy state in bits the node already has works. In two cycles, a segment flag in a link's high bit and a counter in the padding byte left the node at its compact size. Any new field that doesn't reuse existing space costs a full word, because the node has no free padding.
- SLRU-style segmentation gave the most consistent hit-rate gains on the MSR block traces. In cycle 2 the compact SLRU improved every trace at both cache sizes, more broadly than CLOCK or GCLOCK did.
- Changes to node representation alone move one axis cleanly. Policy changes move several axes at once, with mixed results.

## Recurring failures

- Using the hash map to cut get latency failed twice. Swapping in a cheaper or faster hasher didn't help, and once made get worse. The probe looks bound by memory and sensitive to how hashes spread structured block keys (inferred, since probe lengths aren't measured).
- Changing the list or nodes doesn't speed up get. Removing relinking and compacting nodes both left get p50 roughly flat, so the cost of a hit is not in list maintenance.
- Every frequency- or scan-aware policy has paid for its hit-rate gains on the put/eviction path. In SLRU the cost is the double relink and per-segment branching. In CLOCK and GCLOCK it is long sweeps, made worse by decrement-by-one aging.
- No policy tried so far fixes web_0 at the large size, whether recency segmentation, a single reference bit or small counters.

## Open questions

- Can SLRU's put cost be cut? Options: avoid the second relink on promotion, give updates to existing keys a fast path, or remove the per-segment head/tail branching.
- Would an adaptive or larger probation share fix SLRU at the small cache size?
- What does web_0 at the large size need? Candidates are ghost/history entries (an ARC/LIRS-style approach) or an adaptive split.
- Would table density or better hash mixing move get p50 where hash speed did not? Probe lengths need to be measured first.
- Can CLOCK-family sweeps be bounded or amortised, for example with a capped sweep or bulk-halving aging moved off the eviction path, while keeping their median gains?
- Was GCLOCK's get slowdown real, or an effect of long put sweeps interleaved with gets in the benchmark?

## Per-policy notes

- LRU baseline / representation: `u32` link compaction is the standard layout for later variants and leaves hit rate exactly unchanged. Further compaction is not a speed lever.
- SLRU: it is the strongest hit-rate policy so far. Its first version paid in metadata and latency. Its second, flag-packed on compact nodes, removed the metadata penalty and limited the remaining cost to put. It is the current default base.
- CLOCK: lazy promotion shrank metadata and helped hm_0, but its effect on hit rate depends on cache size and trace (slightly worse on web_0 at the large size). Its put tail latency is unbounded because of the sweep.
- GCLOCK: the counter fits in the padding byte, but its hit-rate gain was mostly hm_0 at the large size. Decrement-by-one aging inflates the sweep, so drop it unless aging moves off the eviction path.
