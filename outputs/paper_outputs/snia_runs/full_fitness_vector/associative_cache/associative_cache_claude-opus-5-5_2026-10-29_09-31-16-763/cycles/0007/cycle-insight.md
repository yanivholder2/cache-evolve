# Cycle 0007 Insight

## Cohort outcome

All three variants branched from cycle 0006/v002 (fused rotation on the byte-tag base). v001 inlined the ClassicLRU hit path using `rotate_left(1)`. v002 made the BIP period depend on bucket width (1/32 narrow, 1/128 wide). v003 removed the global atomic tick while keeping bucket stride fixed. All three are Winners, but none is a clean speed win. v002 gave the only real movement, a small net hit-rate gain. v003 closed an open question without improving speed. v001 is a negative result that tells us how to do it next time.

## Patterns

- Per-op-time drifted again. Both v002 and v003 got slower on code paths they barely changed or only removed work from, and get-p99 swung in both directions with no code change behind it. Three cycles of this mean single-run tail numbers should not be used to judge anything. Only same-session, repeated runs against the parent count.
- Removing the atomic clock is settled as speed-neutral on the tag/fused base. Bucket stride was pinned (80 bytes) and there was still no put gain. The atomic is uncontended and stays in L1, so it was never the bottleneck. Keep the change for correctness and concurrency only, and stop expecting a speed win from it.
- Generic standard-library helpers can hide slow paths. `rotate_left(1)` on short suffixes falls into a scalar loop that moves one element at a time and cost more than the remove+push memmoves it replaced. The pattern that wins on put is a hand-written `ptr::read`/`ptr::copy`/`ptr::write`, and it hasn't been applied to the hit path yet.
- A static policy split by bucket width was the first hit-rate change in several cycles that did what it predicted. Web_0 on wide buckets improved, and narrow cells stayed bit-identical. But hm_0 on the same wide buckets gave most of it back, so on wide buckets the BIP rate is close to a zero-sum knob between the two workloads, not a free lever.
- Hit-rate invariants again held exactly for every mechanical rewrite. Policy semantics stay robust to restructuring.

## Open questions

- Does an inline, devirtualised hit path with an MRU-skip and a manual `ptr::copy` shift (with the same done for tags) finally lower get p50?
- Is there a wide-bucket BIP period between 1/32 and 1/128 (for example 1/64) that keeps most of the web_0 gain and less of the hm_0 loss? Or is that curve flat?
- Do v002's and v003's apparent slowdowns go away in a back-to-back same-session rerun?

## Continuation

Merge v002's width-keyed BIP and v003's stride-neutral clock removal into one base. Both keep hit rate and size invariant. Re-measure it against cycle 0006/v002 in the same session. On that base, retry v001's hit-path idea with a hand-written `ptr::copy` shift instead of `rotate_left`. Treat the wide-bucket BIP period as a tunable worth at most one more probe, such as 1/64. Drop any further speed expectations from removing the atomic.
