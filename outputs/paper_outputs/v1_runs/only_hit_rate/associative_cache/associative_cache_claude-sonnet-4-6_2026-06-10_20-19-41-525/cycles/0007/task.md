# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort probed whether the accumulated mechanism stack could be reformed: CLOCK replaced bulk-decay eviction, selective decay targeted only minimum-counter items, and a clean-slate pure-LFU probe tested wide geometry without any positional tricks. All three fell short of the prior best; selective decay (v002) was selected as the least-disruptive departure.

## Patterns

- **Positional recency in the accumulated stack is non-redundant.** Both CLOCK and selective decay improved frequency discrimination but lost the recency signal embedded in bucket position — the LRU-front (index-0) tie-breaker implicitly encodes insertion age and rewards workload-a's temporal locality. Removing that signal cost more than better frequency ordering gained in either case.
- **Episodic bulk decay outperforms per-item incremental aging on this workload.** CLOCK's sweep decrements counters on every eviction call, steadily eroding active items' protection. The bulk-decay fallback fires rarely, concentrating its counter-compression cost; CLOCK trades that infrequent cost for a steady one that makes still-relevant items evictable sooner.
- **Pure min-counter LFU on wide buckets is a near-approximation of the full stack.** Wide geometry plus frequency-only eviction closed most of the gap from the unmodified baseline to the accumulated best-known, confirming that a large candidate pool largely substitutes for cold-first bias and decay. The remaining shortfall quantifies the combined contribution of those positional mechanisms — small but real.
- **The local optimum is increasingly confirmed as robust.** Three consecutive cycles of structural challenges have each returned regressions. The accumulated stack's lead is not narrow-margin luck; it reflects genuine complementarity between its components.

## Open questions

- Is the recency benefit of the LRU-front tie-breaker workload-specific, or would it persist on traces with weaker temporal locality?
- Per-op latency remains unmeasured: does the simpler pure-LFU wide-bucket path offer a better hit-rate/latency trade-off than the full stack, even at a small hit-rate penalty?
- Could a hybrid — LFU eviction with the decay trigger retained but selective-decay semantics applied — recover the recency loss while preserving counter differentiation?

## Continuation

The accumulated stack continues to hold despite sustained pressure. The closest alternative is v003's pure-LFU + wide-geometry configuration, which approaches but does not match the best-known. Next cycle should pivot to latency measurement: compare the full accumulated stack against v003's simpler path to determine whether the hit-rate delta is worth the mechanism complexity. If those results show acceptable parity, the simpler path becomes the preferred baseline for subsequent exploration. Do not revisit CLOCK or selective decay — both confirm that disrupting positional recency costs more than it gains on this workload. An adaptive two-queue or ARC-style policy that separates recency and frequency into explicit queues (rather than encoding both in position) remains the most promising structural alternative if the latency comparison does not resolve the choice.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
