# Cycle 0010 Insight

## Cohort outcome

This cycle tested three structurally distinct ideas at the 87.5% confirmed baseline: a pure hasher swap (FNV-1a in isolation), frequency-biased eviction ordering within the probationary queue, and a lazy-demotion flag mechanism designed to decouple quota tightening from get-path cost. Only v001 won — on struct size alone — while the other two each incurred a latency penalty that negated or outweighed their respective benefits.

## Patterns

- **FNV-1a delivers a clean, cost-free structural win at the clean baseline.** Applied without any quota change, the zero-sized hasher shrank the `LruCache` struct by 16 bytes with no latency regression and no hit-rate effect. This confirms the cycle 0009 prediction that the benefit was real but had been masked by simultaneous quota increases. The caveat is that the fitness scorer tracks Node size rather than cache-instance size, so the gain is invisible to the primary ranking signal — the actual allocation win is real but not credited.
- **Frequency-biased probationary eviction is structurally incompatible with a small probationary pool.** Even a bounded 4-candidate scan could not generate meaningful frequency discrimination when the pool holds only capacity/8 slots; counter aliasing and short item lifetimes reduce the heuristic to noise. More critically, hashing every get unconditionally to maintain the counter adds a measurable hot-path cost, making the mechanism a net loss regardless of pool size.
- **Lazy demotion via a stolen flag bit reduces pointer work but not get-path cost.** v003 showed that replacing eager pointer manipulation with a single bit-OR does not escape the latency coupling: the flag-check branch, the bit-AND on every probationary graduation, and the eager-fallback path on consecutive graduations together still push get p50 above the measurement floor at 90% quota. The cost is in the conditional overhead around the demotion event, not solely in the linked-list updates themselves.
- **The quota-latency coupling persists through every decoupling attempt so far tried.** Intermediate quota (cycle 0009 v001), demotion batching (0009 v002), FNV-1a offset (0009 v003), and now flag-based lazy demotion (0010 v003) have all failed. The common thread is that any branch or bit operation on the get path that fires in proportion to overflow frequency is sufficient to surface above the timer floor.

## Open questions

- Since the fitness scorer anchors on Node size and ignores cache-instance struct size, v001's win is structurally undervalued. Should the fitness function be extended to track the `LruCache` struct footprint separately from the Node footprint — and would doing so unlock a class of zero-node-overhead improvements that are currently invisible?
- With quota, admission, eviction ordering, demotion batching, lazy flags, and hashing all explored, is there a remaining per-operation cost axis — e.g., reducing the number of pointer indirections per promotion rather than amortising or deferring them — that could lower the base get cost enough to absorb a modest quota increase?
- Could a fundamentally different policy topology (e.g., a single-queue scan-resistant variant with a ghost-entry bloom filter rather than a two-queue split) achieve a high effective protected ratio without any per-get demotion enforcement at all?

## Continuation

The next cycle should treat v001 (87.5% quota, FNV-1a hasher, 16-byte Node) as the new baseline and accept that all quota-increase strategies are exhausted. Two directions worth pursuing: (1) explore whether reducing pointer-indirection depth per promotion — e.g., by encoding both queue membership and LRU position in a single index update rather than two separate link-field writes — can lower the baseline get cost and create new headroom; (2) consider a policy redesign that avoids any per-promotion demotion entirely, such as a ghost-entry or clock-hand variant where the protected/probationary boundary is enforced lazily by position rather than by active relinking. Do not revisit frequency sketches, demotion batching, flag-based lazy demotion, or quota values above 87.5%.
