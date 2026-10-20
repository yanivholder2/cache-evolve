## Recurring wins

- **Wider buckets (first step only) consistently improve hit-rate.** The initial geometry widening produced the single largest gain across all cycles; it remains the necessary foundation for cold-first eviction to function.
- **Cold-first eviction (counter == 0 priority) is the dominant lever.** Shielding any item that has received even one access reliably outperforms all positional or frequency-only alternatives tested so far.
- **The counter == 0 cold threshold is well-calibrated for workload-a.** Single access is a reliable signal of future reuse on this trace; the boundary has survived two independent challenges and should be treated as settled.
- **Episodic bulk counter decay reliably resolves warm-lock.** Firing only when no cold candidate exists, it re-enables cold-first selection with minimal impact on the normal eviction path and outperforms both CLOCK-style steady erosion and selective per-item aging.
- **Positional recency (LRU-front tie-breaker) is non-redundant.** The insertion-order signal encoded in bucket position captures temporal locality that frequency counters alone do not; every attempt to displace it has regressed hit-rate.

## Recurring failures

- **O(1) swap-to-back LRU promotion** corrupts the recency gradient across intermediate slots, producing a clear hit-rate regression regardless of boundary preservation.
- **Threshold tightening (counter ≤ 1 or counter ≥ 2 before shielding)** evicts hot-set members prematurely on workload-a; the single-access threshold is not conservative.
- **Geometry widening beyond ~32–64 slots** consistently yields diminishing returns; confirmed saturated on both plain and mid-point insertion paths.
- **LFU warm-fallback after bulk decay** is counter-productive: decay compresses counter differences to near-zero, making minimum-counter selection noisier than the positional tie-breaker already in place.
- **Mid-point insertion combined with LFU warm-fallback** is adversarial — mid-point insertion disrupts the recency structure that LFU warm-fallback implicitly relies on, and cold-first eviction's position-agnostic scan makes insertion site a secondary concern anyway.
- **CLOCK replacement** trades the rare cost of episodic decay for a steady per-eviction counter erosion that makes still-relevant items evictable too soon; consistently inferior to bulk decay on this workload.
- **Selective / per-item incremental aging** disrupts positional recency signal for smaller frequency-discrimination gains than it loses.

## Open questions

- **Per-op latency remains entirely unmeasured.** The full accumulated stack (wide buckets + cold-first + mid-point insertion + bulk decay + counter cap) carries several O(n) paths whose combined cost has never been profiled. Does the narrow-geometry + mid-point path offer a better hit-rate/latency trade-off?
- **Generalisability beyond workload-a.** Every finding above is derived from a single LIRS trace with a specific scan-dominated locality profile. None of the gains or failures have been validated on alternative traces.
- **Structural escape from the local optimum.** Three consecutive cycles of mechanism challenges have failed to surpass the accumulated stack. An ARC-style or two-queue policy that separates recency and frequency into explicit structures — rather than encoding both in bucket position — is the most credible unexplored path past the current ceiling.
- **Counter-decay + LFU fallback stacking.** The two address orthogonal failure modes (restoring cold supply vs. improving warm selection quality) and have not been cleanly combined since both were confirmed individually.
- **Minimum viable bucket size for mid-point insertion.** Below some slot count, mid-point insertion can no longer maintain enough cold candidates to substitute for bucket widening; this lower bound has not been probed.

## Per-policy notes

- **Baseline ClassicLRU (narrow buckets):** Strict O(n) recency ordering is correct and load-bearing. The bottleneck is bucket width, not ordering cost.
- **Wide-bucket ClassicLRU:** First widening step is transformative; further widening saturates. This is the required foundation for cold-first eviction.
- **Cold-first eviction:** The highest-value eviction strategy found; requires sufficient cold candidates per bucket — achievable via either wider geometry (cycles 1–2) or mid-point insertion (cycle 3). Confirmed strongest on workload-a's scan-dominated pattern.
- **Mid-point insertion:** Substitutes for one step of bucket widening as a cold-candidate concentrator on narrow geometry. Weakly additive when stacked on wide geometry where cold-first already dominates. Carries an unmeasured O(n) insertion cost.
- **Counter decay (episodic bulk):** Effective, infrequent warm-lock recovery; preserves counter differentiation between triggers. Winner in cycle 4. Counter cap (cycle 5) is a safe passive hardening with no measurable hit-rate effect on this workload.
- **LFU (min-counter) eviction, wide geometry:** Approximates the full stack but falls short; loses the temporal-locality signal encoded in position. Viable as a simpler implementation if latency measurements show acceptable parity.
- **CLOCK:** Inferior to episodic decay on workload-a; steady erosion costs more than it gains relative to the rare bulk trigger. Not a promising direction without evidence of a workload where temporal locality is weaker.
