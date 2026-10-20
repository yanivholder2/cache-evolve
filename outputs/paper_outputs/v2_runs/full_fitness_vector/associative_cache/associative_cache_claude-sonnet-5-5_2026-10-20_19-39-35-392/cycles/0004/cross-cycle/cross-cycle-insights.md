## Recurring wins

- Scan-resistant insertion/eviction policy is the only lever that has moved hit rate (cycle 0002 onward). One-hit-wonders must not displace reused entries.
- Mid-bucket (about len/4 to mid) insertion plus explicit transpose-by-one promotion on hit is the best policy shape found so far.
- Cheap, stateless mechanisms beat stateful ones. Insertion position and a reference bit beat frequency counters.
- Put latency improves when per-put work shrinks (O(1) swap insertion, hash-once indexing). Where the new entry lands matters; the exact order of the rest does not.
- No winning variant has needed extra metadata.

## Recurring failures

- Geometry sweeps (bucket width) and position-only LRU tweaks: hit rate is insensitive, so conflict misses are not the bottleneck (cycle 0001).
- Rotate/memmove and arithmetic/hash micro-optimizations: no get gain in four cycles. Item movement and arithmetic do not dominate get time.
- Stacking scan-resistance mechanisms (reference bit plus low insertion) does not compound and can lose both hit rate and speed.
- Pushing insertion too close to the LRU end starves new entries of the chance to prove reuse.
- Weak admission signals (one-entry ghost, per-slot frequency counter) add bookkeeping cost for little or negative hit-rate return.
- Extra per-access logic (branches, masking, shifts) consistently costs latency.
- Isolated latency shifts with no mechanism (put gains in untouched paths, small get shifts) have repeatedly been noise or artifacts. Discount them.
- Winner status has been awarded on thin or negative evidence (e.g. variants whose own insight reports a small loss). Treat those results as unconfirmed.

## Open questions

- Where does median get time go? Memory layout (cache-line alignment, tag/key packing, prefetch) is the untested suspect, and this has gone unmeasured for four cycles.
- Do any gains hold on a workload other than workload-a, or are they specific to its scan-heavy traffic?
- Is scan resistance already saturated by insertion position, or would a deeper or more selective admission history help?
- Can hash-once indexing be made to help get rather than only put?
- Which Winners are real and which are within noise?

## Per-policy notes

- Baseline LRU: the position and geometry axes are exhausted. Partial promotion hurts hit rate.
- CLOCK reference bit: cheap and cut reordering cost alone, but a one-bit second chance is too coarse when stacked on low insertion.
- Frequency counter (2-bit, LRU tie-break): small hit-rate gain, slower get and put. Dropped.
- Low-position insertion (LIP/BIP-style): clearest single-mechanism hit gain, but at the extreme LRU end it starves new entries.
- Mid-insertion plus transpose promotion: current policy baseline. The O(1) swap form removes the shift cost and is the preferred implementation.
- Ghost admission filter (one entry per bucket): too weak, hurts reusable keys. Dropped.
