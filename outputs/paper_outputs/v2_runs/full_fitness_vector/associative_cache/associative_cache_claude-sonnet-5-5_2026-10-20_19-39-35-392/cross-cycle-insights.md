## Recurring wins

- Scan resistance is what moves hit rate (cycles 0002+). Capacity, associativity and plain LRU reordering were not the limit.
- Mid-bucket insertion (around len/4) with gradual transpose-by-one promotion beats LRU. It was reproduced independently from baseline in cycles 0003, 0006 and 0008. Its cost is put-tail shuffling.
- Real reuse signals beat proxies. The adaptive global hit/miss scheme (0008) and the ghost ring of recent evictions (0010) gave genuine hit-rate gains. Both fit in existing metadata.
- Removing redundant put-path work (half-bucket shifts, repeated hash/modulo) cut put latency without hurting hit rate (0004).

## Recurring failures

- Get latency has had no real gain in eight cycles. Hashing, arithmetic, item movement, tag/alignment layout and fusion were all ruled out. Every per-hit addition (flag, counter, extra compare or swap) made get slower.
- Tuning the insertion constant is saturated on both sides: len/3 and len/5 both lost to len/4. Cascaded or longer displacement chains only moved cost.
- Faster or gated promotion (geometric jump, seen-once gate, position-gated two-slot) removes the frequency filter that transpose-by-one already provides. Hit rate dropped or gained only marginally.
- Stateless randomization (hash-bit victim choice) and weak proxies (victim position, sampled global ratio, one-entry ghost) carry no reuse signal.
- Stacking cheap scan-resistance mechanisms (reference bit plus low insertion) did not compound.
- Alignment and layout tweaks do not reproduce. Put-latency gains with no mechanism keep turning out to be noise.
- Winner status is repeatedly disconnected from the variants' own insights. Many Winners report losses, so treat Winner labels with suspicion.
- No variant has grown metadata, and none has been validated on a second workload.

## Open questions

- Can the ghost-ring signal be made cheaper (smaller tags, hash filter, fewer compares)? Would a larger ring, or ghost-guided eviction, add hit rate?
- Is the get-latency rise seen even when get is untouched a real layout effect or noise? A noise check is needed.
- Where does get time actually go? Harness or dispatch overhead is untested.
- Can swap insertion keep its put saving without demoting the displaced entry to the tail?
- How much of the adaptive gain comes from adaptivity versus fixed mid-insertion mechanics?
- Do any gains hold on a second workload, and are the Winner statuses beyond noise?

## Per-policy notes

- LRU (baseline): position-only tweaks and partial promotion did not help. Hit rate is insensitive to associativity.
- CLOCK reference bit: cheap and a modest win alone (0002). It hurts when stacked on low insertion.
- Frequency counter: marginal gain, with a per-access latency cost. Dropped.
- LIP/BIP-style low insertion: a clear gain alone (0002). Pushing too deep starves new entries.
- Mid-insertion plus transpose promotion: the settled baseline. Swap-based len/4 insertion is cheaper on put but loses hit rate, because the swap demotes the mid entry to the tail.
- Adaptive insertion depth: works with a global hit/miss signal. Put-side proxies and sampled counters failed.
- Ghost admission or ring: a single-entry ghost hurt. A 4-tag ring of recent evictions gave the cleanest recent hit-rate gain at a put-time cost, and is the current lead.
