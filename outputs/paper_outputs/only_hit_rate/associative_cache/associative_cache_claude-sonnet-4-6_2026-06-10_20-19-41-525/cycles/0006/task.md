# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tested three refinements of the cycle 0004 decay-based winner: replacing
positional-LRU fallback with LFU minimum-counter selection after decay, stacking
mid-point insertion onto the narrow-geometry LFU-fallback configuration, and capping
counters at a bounded maximum. All three failed to improve on their parents — two
regressed and one was neutral — making the counter-cap variant (v003) the winner by
elimination rather than by merit.

## Patterns

- **Uniform counter decay destroys the frequency signal it was meant to preserve.**
  After a single pass that decrements every counter by the same amount, inter-item
  counter differences compress to near-zero; a minimum-counter scan then becomes
  noisier than the recency order already encoded in bucket position. The positional
  `remove_at(0)` fallback was already well-aligned with insertion ordering and
  outperforms frequency selection in this compressed regime.
- **Mid-point insertion and LFU warm-fallback are adversarial, not additive.**
  Cold-first eviction scans by counter value and is position-agnostic, so mid-point
  insertion's benefit — keeping new items away from the eviction front — is irrelevant
  to it. Meanwhile, the LFU warm-fallback still implicitly relies on the
  promotion-to-back recency structure; disrupting that structure with midpoint inserts
  costs more than the reduced cold-pool exhaustion saves. The two mechanisms should
  not be combined on narrow geometry.
- **The counter-cap hypothesis was sound in theory but inert on this workload.**
  Unbounded accumulation does not manifest as a measurable hit-rate problem on
  workload-a at the tested scale; the decay mechanism already fires frequently enough
  to prevent permanent warm-lock from high-counter items. The cap is safe to carry
  forward as defensive hardening but opens no new lever.
- **The wide-bucket + mid-point + cold-first + decay configuration is a local optimum
  for hit-rate on workload-a.** Two consecutive cycles of refinement attempts have
  returned regressions or neutrality; the design space of eviction-policy tuning on
  this geometry and trace appears saturated.

## Open questions

- Is hit-rate on workload-a genuinely saturated, or do structurally different eviction
  families (CLOCK, ARC-style adaptive replacement, LIRS-proper) offer a path past the
  current ceiling?
- Per-op latency has not been measured for the accumulated mechanism stack; is the
  combined cost of cold-first scan + decay pass + mid-point insertion acceptable, and
  does narrow geometry + LFU fallback offer a better hit-rate/latency trade-off?
- Do the current regressions generalise — i.e., would LFU-after-decay and mid-point +
  LFU also fail on traces with different locality profiles, or are these failures
  specific to workload-a's scan-dominated pattern?

## Continuation

Hit-rate improvements on workload-a appear exhausted for the current mechanism family.
Next cycle should pivot on one of two axes: (a) measure per-op latency across the
full accumulated stack to determine whether the narrow-geometry or wide-geometry path
offers a better efficiency trade-off — this is the last unprobed dimension with a
clear comparison point — or (b) introduce a structurally different eviction policy
(e.g., CLOCK or an adaptive two-queue scheme) to break out of the local optimum. Do
not revisit LFU-after-decay fallback or mid-point + LFU combinations — both are
confirmed counter-productive. The counter cap (v003) is confirmed safe and should
be inherited silently as part of the baseline.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
