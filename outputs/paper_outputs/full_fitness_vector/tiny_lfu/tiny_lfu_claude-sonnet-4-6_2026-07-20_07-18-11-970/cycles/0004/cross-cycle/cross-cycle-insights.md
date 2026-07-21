## Recurring wins

- **Hash-work reduction via bit-slicing beats serial multi-hash.** Deriving all row positions from a single hash output (cycle 0003) produced the first sketch winner; adding a second seeded hash (cycle 0004 v001) partially recovered row independence and further improved hit rate, confirming that eliminating redundant full-hash invocations is a real gain when serial inter-hash dependencies are avoided.
- **Amortised sampling turns expensive probes into winners.** Best-of-2 eviction at 1-in-8 probe rate (cycle 0004 v002) achieved hit-rate improvement with latency parity; the hit-rate signal survives coarse sampling, proving the underlying eviction signal is strong enough to exploit cheaply.
- **Idiomatic Rust abstractions outperform hand-rolled arithmetic.** In every cycle where a hand-optimised alternative (sentinel u32, double-hash chain, explicit flat strides) competed against compiler-friendly idioms, the idiomatic form won or tied; explicit arithmetic introduced branch-prediction or ILP penalties that negated the intended saving.

## Recurring failures

- **Sketch decay at any tested interval.** Two cadences failed: `width/2` (cycle 0001, too aggressive) and `capacity × 10` (cycle 0003, still too aggressive). The LIRS trace's stable hot-set makes accumulated frequency a reliable signal; blanket half-decay penalises hot and cold keys alike and the periodic full-table sweep pressures cache, raising latency. This axis is definitively closed.
- **Node memory layout changes.** Compact u32 sentinel nodes regressed latency (cycle 0001); flat single-allocation sketch doubled latency via cache-set conflicts (cycle 0002); halved sketch depth hurt both hit rate and latency (cycle 0002). NonZeroU32 with idiomatic `Option` match corrected the discriminant regression (cycle 0002 v001) but delivered no winner-level gain. The node-layout and sketch-structure axes are both exhausted.
- **Serial hash dependencies.** Any scheme that chains hash computations sequentially (full double-hash, cycle 0001; second independent SipHash seeded from first, cycle 0004 v001's p99 cost) concentrates overhead in tail latency due to ILP loss; the gain from fewer total invocations is eroded or reversed.
- **Stricter admission gates without accompanying eviction relief.** A harder-to-satisfy admission threshold (cycle 0004 v003) improved hit rate but disrupted the eviction-firing cadence enough to raise latency substantially — the benefit did not survive alone.

## Open questions

- Can a cheap bijective re-mix (multiply-xorshift step) of the first hash output replicate v001's row-independence gain without the full cost of a second SipHash invocation, closing the p99 tail gap?
- Does a finer amortisation ratio for best-of-2 eviction (1-in-4 or 1-in-2 probe rate) capture proportionally more hit-rate benefit while remaining latency-neutral, or does latency break before 1-in-2?
- Does combining cycle 0004 v003's stricter admission gate with v002's amortised eviction compound their hit-rate signals while v002's reduced eviction rate neutralises v003's latency disruption?
- Are there untested fitness gains on window/protected-segment sizing or probationary-to-protected promotion policy, orthogonal to the now-exhausted sketch and node axes?

## Per-policy notes

- **CountMinSketch:** Four-row multi-allocation baseline is well-balanced for this trace and resists structural simplification. Single-hash bit-slice (cycle 0003 winner) is the current anchor encoding; two-hash bit-slice (cycle 0004 v001 winner) improves hit rate at marginal p99 cost. Flat allocation and depth reduction are both closed. Decay is definitively closed at all tested intervals.
- **Node layout:** `Option<NonZeroU32>` with idiomatic match arms is the correct encoding; it recovered the cycle 0001 discriminant regression but provides no further fitness gain. The node-layout axis is exhausted and should not be revisited.
- **Eviction policy:** Amortised best-of-2 victim comparison at 1-in-8 sampling rate is a confirmed winner (cycle 0004 v002); eviction firing rate is a latency lever — reducing it relieves the common path. The hit-rate signal from frequency-aware victim selection is strong and survives coarse sampling. Stricter admission alone is latency-disruptive; it is a candidate only in combination with eviction amortisation.
