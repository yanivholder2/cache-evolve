# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort probed three independent axes — bucket geometry, per-item metadata size,
and LRU operation complexity — against the baseline on the LIRS workload-a trace.
Only the geometry change (fewer, wider buckets) landed in the Winners; the latency
optimisation hurt hit-rate, and the metadata compaction went unmeasured on the
dimensions it targeted.

## Patterns

- **Wider buckets directly improve hit-rate.** Repartitioning the same total capacity
  into fewer, larger sets gives the per-set LRU a bigger eviction pool, reducing
  premature expulsion of still-live items. The baseline's narrow buckets were a real
  bottleneck on this workload.
- **Strict LRU ordering is load-bearing.** The O(1) swap-to-back optimisation
  preserved the two boundary indices (eviction candidate and MRU) but corrupted the
  recency gradient across intermediate slots, and that degradation was large enough to
  register as a clear hit-rate regression. The O(n) cost buys correctness worth keeping.
- **Unmeasured hypotheses remain open, not disproven.** The timestamp-removal variant
  neither helped nor hurt hit-rate (expected, since ClassicLRU never reads the field),
  but the evaluators for the dimensions it actually targeted — metadata size and
  per-op latency — never ran, leaving the structural benefit unconfirmed.

## Open questions

- Does the bucket-widening gain hold beyond the LIRS workload-a trace, or is it
  specific to its locality pattern?
- Is there a recency-aware O(1) promotion strategy that preserves eviction quality
  better than a plain swap (e.g., a clock approximation or segmented LRU)?
- Can the metadata-size and per-op-time evaluators be enabled so the timestamp-removal
  change gets a real verdict?

## Continuation

The geometry axis is the strongest lead: wider buckets produced a real win, and the
direction has not been saturated — next cycle should test pushing the bucket-width
further or making it adaptive. Drop the O(1) swap angle for ClassicLRU unless a
recency-preserving variant emerges. Before spawning a new metadata-compaction
hypothesis, ensure the necessary evaluators are running so any structural benefit can
actually be confirmed.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort probed three anti-scan-pollution strategies — further bucket widening,
cold-first eviction, and mid-point insertion — on top of cycle 0001's geometry win.
The cold-first eviction variant (v002) was the clear winner, delivering a large
hit-rate gain built on the wider-bucket foundation; the other two produced smaller
or marginal returns without being selected.

## Patterns

- **Geometry widening has reached diminishing returns.** Pushing the bucket factor
  from 4 to 16 yielded only a fraction of the gain the first widening step produced.
  The LIRS workload-a hot set largely fits within 64-slot buckets; further geometric
  expansion is unlikely to pay off.
- **Cold-first eviction is the strongest lever found so far.** Preferring
  counter==0 items as eviction candidates shields any item that has received even a
  single GET hit from being displaced by scan insertions. The wider buckets from
  cycle 0001 are a necessary enabler: they provide enough resident cold items to
  absorb scan traffic without immediately exhausting the cold pool and collapsing
  back to plain LRU.
- **Mid-point insertion addresses the same root cause from the insertion side.**
  Landing new items at the bucket midpoint rather than the MRU back also improved
  hit-rate without hurting any measured dimension, but produced a smaller gain than
  cold-first eviction and carries an unmeasured O(n) insertion-latency cost.
- **Both winning angles converge on scan-pollution as the dominant bottleneck.**
  v002 and v003 attacked the problem from eviction and insertion respectively;
  neither was combined with the other this cycle.

## Open questions

- Can cold-first eviction (v002) and mid-point insertion (v003) be stacked, and do
  they compound or merely overlap in their benefit?
- Does v002's linear scan across bucket slots to find a cold item add meaningful
  per-op latency at scale, particularly in wide buckets?
- As the cache warms up over a long trace, does the cold pool thin out — causing
  the fallback to plain LRU to trigger more often — and if so, does the win degrade?
- Do the gains from v002 and v003 generalise beyond the LIRS workload-a trace?

## Continuation

The cold-first eviction approach (v002) is the strongest lead: it produced a large
hit-rate gain, has a clear correctness story, and the geometry axis it depends on is
already confirmed. Next cycle should explore stacking mid-point insertion on top of
v002, probe whether a frequency threshold greater than 1 (e.g., counter ≥ 2) improves
separation further, and add a `per-op-time` evaluation pass to v002 to measure the
scan cost before widening buckets further. Drop pure geometry pushing — that axis
is saturated.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tested three refinements of the cold-first eviction strategy: stacking
mid-point insertion on top of the cycle 0002 winner, tightening the cold threshold
from counter==0 to counter≤1, and applying cold-first eviction on narrow-bucket
geometry via mid-point insertion. The stacked combination (v001) won on a marginal
hit-rate gain; threshold tightening (v002) regressed; and the narrow-geometry
cold-first variant (v003) improved over its parent without being selected, because the
wide-bucket baseline it competed against started from a higher absolute floor.

## Patterns

- **The `counter == 0` eviction boundary is already well-calibrated.** Raising the
  warmth threshold to require two accesses before shielding an item caused genuine
  hot-set members to be evicted prematurely. On workload-a a single access is a
  reliable signal of future reuse, not scan noise; the threshold tuned in cycle 0002
  should be treated as settled for this trace.
- **Cold-first eviction and mid-point insertion are weakly additive when the eviction
  mechanism already dominates.** Stacking both together on wide-bucket geometry
  produced only a marginal gain over cold-first eviction alone, because cold-first
  selection already targets counter==0 items regardless of their vector position —
  insertion site becomes a tiebreaker rather than a primary lever.
- **Mid-point insertion can substitute for bucket widening as a cold-item concentrator.**
  Cold-first eviction delivered a hit-rate improvement even on narrow 16-slot buckets
  when mid-point insertion was present. This challenges the cycle 0002 conclusion that
  geometry widening is a necessary prerequisite; the two mechanisms appear to solve the
  same underlying problem — ensuring enough cold candidates exist per bucket — via
  different means.

## Open questions

- Does the mid-point-insertion-as-geometry-substitute trade-off hold under workloads
  with different locality profiles, or is it specific to workload-a's scan pattern?
- What is the per-op latency cost of mid-point insertion's O(n) `vec.insert` relative
  to the cold-first scan; is the marginal hit-rate gain worth the combined cost?
- At what bucket size does mid-point insertion's substitution effect break down — i.e.,
  is there a minimum slot count below which neither trick maintains enough cold
  candidates?

## Continuation

The cold-first + wide-bucket baseline remains the strongest configuration found so far,
with mid-point insertion adding a small incremental win. Before stacking further
mechanisms, next cycle should instrument per-op latency for both v001 and v003 to
determine whether the narrow-geometry + mid-point path offers a better hit-rate/latency
trade-off than the wide-bucket path. The discovery that geometry widening and mid-point
insertion are substitutable opens a new comparison axis worth probing directly. Drop
threshold-tightening hypotheses — the `counter == 0` boundary is confirmed for this
workload.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants converged on the same root problem — what happens when the cold
pool runs dry — and each attacked it from a distinct angle: restoring cold candidates
via counter decay (winner v001), improving warm-fallback selection quality via LFU
(v002), and enlarging the cold pool via modest geometry widening (v003). Only the
counter-decay variant was selected, but all three showed consistent hit-rate gains,
confirming that warm-lock is a real, non-trivial steady-state failure mode for the
cold-first eviction family.

## Patterns

- **Counter decay is an effective self-correcting patch for warm-lock.** Decrementing
  all counters by 1 only when no cold candidate exists fires rarely but reliably,
  re-enabling cold-first selection without touching the normal eviction path. Its win
  confirms that cold pool exhaustion does degrade hit-rate in steady state and that a
  targeted, infrequent intervention is sufficient to fix it.
- **LFU warm-fallback is independently valuable.** On narrow geometry — where cold
  pool exhaustion is routine — replacing the LRU-position fallback with minimum-counter
  selection produced a substantial hit-rate gain with no geometry change. This is a
  clean orthogonal axis: it improves eviction quality during the warm phase without
  depending on bucket width or insertion site.
- **Mid-point + cold-first geometry gains saturate at ~32 slots per bucket.** Adding a
  factor-2 widening to the narrow-geometry mid-point path matched or marginally exceeded
  the wide-bucket (factor-4) configuration from cycle 0003, confirming that further
  geometry pushing on mid-point paths is exhausted. The useful geometry range for
  mid-point insertion ends around 32 slots.

## Open questions

- Can counter-decay (v001) and LFU warm-fallback (v002) be stacked productively? They
  are aimed at different aspects of warm-lock — restoring cold candidates versus
  choosing better among warm ones — and may compound rather than overlap.
- Does the counter-decay mechanism remain stable over very long traces, or does periodic
  bulk decrement cause oscillation in counter values that destabilises the cold/warm
  boundary?
- Would the LFU fallback add further gain to v001's wide-bucket + mid-point + decay
  configuration, or is decay sufficient to keep the cold pool from exhausting on wide
  geometry?

## Continuation

The most promising next step is combining v001's counter-decay with v002's LFU fallback:
the two mechanisms are orthogonal in function (one restores cold supply, the other
improves warm selection quality) and each was tested independently without the other.
Drop pure geometry widening — that axis is now doubly confirmed saturated for mid-point
insertion paths. Before adding further mechanisms, run a per-op latency pass on v001 to
verify that the decay scan does not impose a meaningful overhead on the normal eviction
path.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
