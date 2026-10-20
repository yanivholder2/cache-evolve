## Recurring wins

- **Wider bucket geometry improves hit-rate** by giving the per-bucket eviction pool enough resident items to survive scan bursts; this was the first lever found (cycle 0001) and remains a prerequisite for most subsequent gains.
- **Cold-first eviction (`counter == 0`) is the dominant single mechanism.** Preferring unaccessed items as eviction candidates reliably shields the hot set from scan displacement; this has compounded positively with every compatible layer added on top of it.
- **Episodic bulk counter decay corrects warm-lock without destabilising the normal eviction path.** Firing only when no cold candidate exists, it re-enables cold-first selection at minimal cost and has been a net positive across every cycle it appeared in.
- **Mid-point insertion provides non-trivial eviction-ordering value.** Placing new items at the bucket midpoint keeps them off the immediate eviction front; removing it (cycle 0008 v001) confirmed a measurable hit-rate regression, ruling it out as a cost-saving cut.
- **Positional recency (LRU-front index-0 tie-breaker) is a real, loadbearing signal.** Under mid-point insertion, index 0 encodes the oldest item and rewards workload-a's temporal locality; every attempt to replace this with a frequency-only heuristic has regressed.
- **Structural queue separation (SLRU) broke the local optimum that single-queue tuning could not.** Cycle 0008's probationary/protected split improved hit-rate for the first time in four consecutive stagnant cycles, opening a new evolutionary branch.

## Recurring failures

- **O(1) LRU swap-to-back (cycle 0001):** Corrupts the recency gradient across intermediate bucket slots; the O(n) cost of correct LRU ordering is worth paying.
- **Threshold tightening beyond `counter == 0` (cycles 0003, 0005, 0007):** Requiring two accesses before shielding an item causes genuine hot-set members to be evicted prematurely on workload-a; single-access protection is already well-calibrated.
- **LFU minimum-counter fallback after bulk decay (cycle 0005):** Uniform decay collapses inter-item counter differences to near-zero, making minimum-counter selection noisier than the recency order already encoded in position. LFU selection and bulk decay are adversarial in combination.
- **Mid-point insertion stacked with LFU warm-fallback (cycles 0005, 0006):** Cold-first eviction is position-agnostic; midpoint placement's benefit is irrelevant to it, while LFU fallback relies on recency structure that midpoint inserts disrupt. The two should not be combined.
- **CLOCK-style incremental counter aging (cycle 0006):** Steadily eroding counters on every eviction call makes still-relevant items evictable too soon; the episodic bulk-decay approach concentrates its cost and outperforms continuous sweep.
- **Selective (per-item) decay targeting minimum-counter items (cycle 0006):** Loses the positional recency benefit of the index-0 tie-breaker without recovering meaningful frequency discrimination.
- **ARC-style ghost-buffer re-admission (cycle 0007):** Post-eviction re-access is rare on workload-a; the ghost scan overhead is real but the promotion benefit is not. Re-admission schemes are mismatched to this trace.
- **Counter ceiling tuning (cycles 0005, 0007):** Both cap=4 and cap=2 are confirmed inert or mildly regressive; the ceiling magnitude is not a meaningful lever on this workload.
- **Pure geometry pushing beyond 32–64 slots per bucket (cycles 0002, 0004):** Gains from bucket widening saturate early; mid-point insertion can substitute for further widening as a cold-pool concentrator.
- **Post-decay cold scan as warm-lock heuristic (cycle 0007):** Selects items whose counter reached zero first rather than oldest-by-position; risks evicting recently inserted items, losing the recency signal for no frequency gain.

## Open questions

- **Per-op latency is unresolved after eight cycles.** The combined cost of cold-first scan + episodic decay + mid-point insertion has never been measured. The simpler pure-LFU + wide-geometry path (cycle 0006 v003) approaches the best-known hit-rate — whether its efficiency is competitive is a critical unanswered question.
- **SLRU segment sizing and promotion threshold are untuned.** Cycle 0008's SLRU gain came from an initial design; varying the probationary/protected split ratio and the promotion threshold may compound that improvement.
- **Generalisability beyond workload-a is untested.** Cold-first eviction, positional recency, and bulk decay are all calibrated to a scan-dominated LIRS trace; it is unknown whether the accumulated stack retains its advantage under traces with different locality profiles (e.g., Zipf-heavy, uniform random, or scan-then-reuse patterns).
- **Optimal decay magnitude and interaction with SLRU.** Doubling the decay step was a net positive (cycle 0008 v003), but the decay mechanism was designed for a single-queue layout; its correct parameterisation inside an SLRU two-queue structure has not been established.
- **Whether cold-first eviction and mid-point insertion remain complementary inside SLRU.** Both mechanisms were developed in the single-queue context; their interaction with SLRU's explicit tier boundaries is unknown.

## Per-policy notes

- **ClassicLRU (single-queue, bucket-partitioned):** The base policy on which all gains through cycle 0007 were built. Hit-rate is effectively saturated on workload-a at the level reached by the accumulated stack (cold-first + wide buckets + mid-point insertion + episodic bulk decay + counter cap). Four consecutive cycles of structural and parametric challenges failed to improve it further before SLRU opened a new branch.
- **SLRU (two-queue: probationary + protected, cycle 0008 v002):** First structural alternative to produce a hit-rate gain after four stagnant cycles. Explicit tier separation prevents scan traffic from displacing the confirmed working set — the mechanism the single-queue family encoded only implicitly in counter values and position. The most promising branch for further development; untuned segment sizing and no latency data yet.
- **CLOCK (cycle 0006 v001):** Reference-bit sweep is a clean approximation of LRU but its continuous counter erosion is mismatched to the episodic decay pattern workload-a rewards. Confirmed inferior on this trace; do not revisit.
- **Pure-LFU on wide geometry (cycle 0006 v003):** Closed most of the gap from the unmodified baseline to the accumulated best without any positional tricks, confirming that a large candidate pool partially substitutes for cold-first bias. Remains the best simple alternative if latency data makes the full stack's complexity unjustifiable.
- **ARC / ghost-buffer re-admission (cycle 0007 v002):** Structurally mismatched to workload-a's access pattern; post-eviction re-access is too rare for the ghost buffer to pay off. May be worth revisiting only on traces with explicit scan-then-reuse locality.
