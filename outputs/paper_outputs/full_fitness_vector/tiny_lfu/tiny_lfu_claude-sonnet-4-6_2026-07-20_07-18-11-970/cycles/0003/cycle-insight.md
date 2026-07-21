# Cycle 0003 Insight

## Cohort outcome

This cycle tested three independent axes: sketch decay cadence, hash-work reduction, and eviction candidate selection. Only the single-hash bit-slicing approach (v002) earned a winner, by eliminating three redundant hash computations per sketch call; the decay variant repeated a prior failure pattern, and the best-of-2 eviction variant improved hit rate but not enough to offset its latency cost.

## Patterns

- **Sketch decay is definitively closed.** The moderate `capacity × 10` interval — intentionally longer than the cycle 0001 failure — still hurt both hit rate and latency: blanket half-decay penalises hot keys and cold keys identically, and the periodic full-table sweep is large enough to pressure L2/L3, raising the median. Two intervals have now failed on this workload; the stable LIRS working set makes accumulated frequency a reliable signal that decay degrades rather than improves.
- **Hash-work reduction wins when it eliminates serial dependencies.** Bit-slicing a single SipHash output into four non-overlapping windows removes three redundant full hash invocations with no intra-iteration data dependency, confirming the cycle 0001 prediction about ILP. The put p50 and get p99 both showed slight regression, suggesting that four non-overlapping 16-bit windows of one 64-bit value provide marginally weaker row independence than four independently seeded hashes — visible in collision-sensitive tail paths but not in the overall median.
- **Eviction logic is a live axis with a real signal.** v003 is the first variant in any cycle to improve hit rate through candidate selection rather than sketch changes. Comparing the two LRU-oldest entries and evicting the lower-frequency one corrects a real class of admission error, but firing two extra sketch probes on every full-cache `put` makes the latency trade-off unfavorable at current eviction frequency.

## Open questions

- Can two-phase hashing (e.g., two seeded hashes, each sliced into two row positions) recover the independence advantage of four independent hashes while still halving the hash cost — preserving v002's gain while closing its tail regression?
- Is there an amortized or deferred form of best-of-2 eviction that batches or lazily skips the second sketch probe to reduce the per-eviction overhead while retaining most of the hit-rate benefit?

## Continuation

Next cycle should build on v002's win: the single-hash bit-slice approach is the new baseline to beat, and the open thread is whether replacing it with two half-hashes (each covering two rows with an independent seed) recovers row independence without re-introducing full redundant hashing. The eviction-axis hypothesis from v003 should be carried forward in a cheaper form — deferred or sampled frequency comparison rather than a mandatory probe on every eviction — since the hit-rate signal it found is real. Decay should not be revisited; both tried intervals have failed and the axis is exhausted.
