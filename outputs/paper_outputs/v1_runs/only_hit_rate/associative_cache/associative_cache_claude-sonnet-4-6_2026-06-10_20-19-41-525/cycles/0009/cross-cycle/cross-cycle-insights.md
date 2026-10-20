# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- **Bucket widening delivered the first and largest single-step hit-rate gain** (cycle
  0001). Fewer, larger buckets give the per-set eviction pool enough residents to avoid
  premature hot-item expulsion. Gains saturated around 32–64 slots; further widening
  past that range adds negligible return.
- **Cold-first eviction (prefer counter==0)** is the strongest eviction lever found
  (cycle 0002). Shielding any item with at least one access from scan displacement is
  effective and well-calibrated; the `counter==0` boundary has been stable across every
  subsequent cycle that tested it.
- **Episodic bulk counter decay** (fire only on warm-lock, decrement all counters by
  the current magnitude) reliably re-seeds the cold pool without disrupting the normal
  eviction path (cycle 0004, confirmed improved at magnitude-2 in cycle 0008). Bulk
  decay outperforms per-eviction incremental aging because it concentrates its
  counter-compression cost rather than eroding active items on every call.
- **Mid-point insertion** provides non-trivial eviction-ordering value on its own and
  can substitute for geometry widening as a cold-candidate concentrator (cycle 0003).
  Removing it regressed hit-rate (cycle 0008), confirming it is not cosmetic.
- **Positional recency (index-0 as warm-fallback target)** is consistently validated.
  Under mid-point insertion index-0 holds the oldest item; blind `remove_at(0)` encodes
  insertion age cheaply and outperforms any frequency-only warm-fallback that discards
  this signal.
- **The accumulated single-queue stack** (wide buckets + cold-first + mid-point
  insertion + episodic bulk decay + counter cap) has held as the best-known
  configuration through cycles 0002–0009, surviving seven consecutive cycles of
  structural and parametric challenges.

## Recurring failures

- **O(1) eviction shortcuts that reorder intermediate slots** corrupt the recency
  gradient and cause clear hit-rate regressions (cycle 0001). The O(n) cost of
  maintaining true LRU ordering pays for itself in eviction quality.
- **Pushing bucket geometry past saturation** (beyond ~32–64 slots) yields diminishing
  returns across multiple attempts (cycles 0002, 0004). This axis is confirmed
  exhausted for both back-append and mid-point paths.
- **LFU warm-fallback after bulk counter decay** (cycles 0005, 0006). Bulk decay
  compresses inter-item counter differences to near-zero; a subsequent minimum-counter
  scan becomes noisier than the recency order already encoded in bucket position. The
  combination is adversarial — tested twice, regressed twice.
- **Mid-point insertion combined with LFU warm-fallback** on narrow geometry (cycle
  0005). Cold-first eviction is position-agnostic; mid-point insertion's benefit
  evaporates when the eviction heuristic ignores position, and LFU warm-fallback then
  loses the recency structure mid-point insertion was meant to preserve.
- **Counter ceiling / cap tuning** — cap=4 (cycle 0005), cap=2 (cycle 0007) — is
  confirmed inert as a hit-rate lever on workload-a at all tested scales.
- **ARC-style ghost re-admission** (cycle 0007). Workload-a lacks post-eviction
  re-access patterns; elevated re-admission status for ghost-tracked keys carries
  real overhead with no measurable benefit. The mechanism is structurally mismatched
  to the workload.
- **CLOCK eviction** (cycle 0006). Continuous per-eviction counter sweeps erode
  protection for still-active items more aggressively than episodic bulk decay.
  The steady-state cost trades the rare, contained compression event for a persistent
  one that makes relevant items evictable sooner.
- **Position-split decay** (cycle 0009). Under cap-4, upper-half items span the full
  counter range, so a gentler decay rate for high-position items preserves lukewarm
  items just as often as hot ones. Counter value already encodes the signal that
  bucket position was meant to add; splitting by position is redundant and inert.
- **Per-op latency measurement** has been recommended since cycle 0005 and deferred
  every cycle. Without it, the efficiency trade-off between the full accumulated stack
  and simpler alternatives (pure-LFU + wide geometry; SLRU) remains blind.

## Open questions

- **Per-op latency audit is the single most urgent unmeasured gap.** The comparison
  between the full accumulated stack and the pure-LFU + wide-geometry path (cycle 0006
  v003) and the best SLRU configuration (cycle 0009 v003) has been deferred across
  five consecutive cycles. Without this data, the efficiency case for mechanism
  complexity vs. a simpler policy cannot be made.
- **Does the accumulated stack's hit-rate ceiling on workload-a reflect the workload's
  limits or the single-queue family's limits?** A structurally different policy
  (ARC-proper, LIRS-proper, adaptive two-queue with explicit recency and frequency
  partitions) has not been probed at the required quality level to answer this.
- **SLRU segment ratio tuning** — the 50/50 split used across cycles 0008–0009 may
  suppress the protected segment below the working-set size. A protected-heavy split
  (e.g., 25/75) could retain more of the hot set per bucket.
- **Generalization beyond workload-a.** Every pattern reported here is conditioned on
  workload-a's scan-dominated locality. Whether cold-first eviction, mid-point
  insertion, and episodic decay transfer to traces with weaker temporal or frequency
  locality has not been tested.
- **Counter-value-based asymmetric decay** (rate-1 for counter ≥ 3, rate-2 for counter
  ≤ 2) remains untested. Position-split decay failed, but a value-split may preserve
  the hot tier without compressing its frequency signal.

## Per-policy notes

- **ClassicLRU (baseline):** Narrow-bucket geometry is a genuine bottleneck on
  workload-a. Without the bucket-widening foundation, all eviction refinements operate
  on an insufficient candidate pool.
- **Single-queue accumulated stack (ClassicLRU + wide buckets + cold-first + mid-point
  + bulk decay + cap):** Best hit-rate configuration across all nine cycles. Structural
  challenges from cycles 0004–0009 have consistently returned regressions. The
  complementarity of its components appears robust on workload-a; it should be treated
  as the production-quality single-queue ceiling.
- **SLRU (two-queue, probationary + protected):** The only structural variant to
  improve hit-rate after the single-queue stack stabilised (cycle 0008 v002). Wide
  geometry is required for SLRU to approach competitive levels (cycle 0009 v002).
  Even the strongest three-way SLRU configuration (cold-first + decay + wide, cycle
  0009 v003) has not overtaken the incumbent. The gap is likely attributable to
  per-operation bookkeeping overhead and the 50/50 segment split; both remain
  unverified without latency data. SLRU is the preferred branch for structural
  exploration if latency data proves it competitive.
- **CLOCK:** Confirmed inferior to episodic bulk decay on workload-a. Steady counter
  erosion is poorly matched to scan-dominated traces where item reuse is bursty
  rather than evenly distributed.
- **Pure LFU + wide geometry:** A near-approximation of the full accumulated stack
  (cycle 0006 v003), confirming that a large candidate pool partially substitutes for
  cold-first bias and decay. The residual gap quantifies the combined contribution of
  positional mechanisms — real but small. May offer a better efficiency trade-off
  once per-op latency is measured.
- **ARC / ghost re-admission:** Structurally mismatched to workload-a. Applicable only
  to traces with measurable post-eviction re-access rates; do not revisit on this
  workload.
