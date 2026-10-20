## Recurring wins

- Stateless or near-stateless scan resistance via insertion position works. Mid-bucket (about len/4) insertion plus gradual transpose-by-one promotion beat plain LRU, reproduced from baseline several times (cycles 0003, 0006, 0008).
- Cheapening the put path pays off when it removes real work. Swap-based O(1) mid-insertion replaced the half-bucket shift, and hash-once indexing cut redundant arithmetic. Approximate order among displaced entries does not matter; where the new entry lands does.
- Policies that keep one-hit-wonders from displacing reused entries move hit rate. Capacity and associativity do not (cycles 0001, 0002).
- Adaptive insertion driven by a global hit/miss counter (cycle 0008) is the only recent mechanism-backed gain. It fits in existing padding.

## Recurring failures

- Get latency has not improved in eight cycles. Item movement, hashing, arithmetic, bucket layout, tag-first scans and alignment are all ruled out. Every per-hit addition (flag, counter, branch) taxed the hot path.
- Stacking scan-resistance mechanisms (reference bit + low insertion, extra gating on promotion) does not compound and usually loses.
- Tuning the insertion constant is exhausted. len/3 and len/5 both failed to beat len/4.
- Faster or gated promotion (geometric jump, seen-once gate) hurts hit rate. Transpose-by-one already acts as a frequency filter.
- Stateless hash-bit randomization for eviction adds noise and branches, with no reuse signal.
- Weak admission signals (one-entry ghost per bucket) cost put time and hit rate.
- Micro-fusing or restructuring the put path (single-scan, put-hit tweaks) adds variance and silently changes placement. Most puts are misses.
- Winner status is often disconnected from a variant's own evidence. Many Winners have insights reporting losses or put-latency gains with no mechanism. Treat these as likely noise.
- Allocation-only layout tweaks (alignment) did not reproduce their put-tail gain.

## Open questions

- Where does get time go if not bucket memory, hashing or reordering? Harness, dispatch, or the key/value path are untested.
- How much of the adaptive-insertion gain comes from the adaptive switch versus fixed swap insertion? A fixed-position control is missing.
- Can adaptivity be driven from the put-miss path alone, to leave the get hot path untouched?
- Does v002's unintended placement (first-empty-slot fill, victim overwrite) help hit rate when adopted deliberately?
- Is there a middle ground on promotion, such as a jump only after a second hit?
- Can a real reuse signal fit in the 40-byte metadata budget? No variant has grown metadata so far.
- Do any gains hold on a second workload, and survive a noise check? Everything so far is on workload-a only.

## Per-policy notes

- LRU (baseline): position-only tweaks, wide buckets and rotate-based moves were all dead ends (cycle 0001).
- CLOCK reference bit: cheap, and it reduced reordering, but it did not stack with low insertion.
- Frequency counter: small gain, with a per-hit and per-put latency cost. Dropped.
- LIP/BIP-style low insertion: clearest early gain. Pushing too deep starves new entries, so mid-bucket is the sweet spot.
- Mid-insertion + transpose (swap-based, about len/4): current settled baseline. Its cost is a data-dependent put tail.
- Adaptive len/4 vs len/2 insertion: current lead. Better hit rate and cheaper puts, with a slightly higher median get.
- Ghost/admission filters: single-entry versions lose. Deeper or more selective history is untested.
