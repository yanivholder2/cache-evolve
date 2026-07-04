# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants probed distinct tuning axes — replacing the warm-lock eviction
heuristic with a post-decay cold scan, adding ARC-style ghost-buffer re-admission,
and halving the counter ceiling — and every one regressed or was negligibly neutral.
v003 was selected as winner purely by elimination, carrying the smallest regression.

## Patterns

- **The positional recency of `remove_at(0)` is already the best warm-lock target.**
  Under mid-point insertion, index 0 is the oldest item in the bucket; swapping to a
  post-decay cold scan risks selecting a more recently inserted item whose counter
  happened to reach zero first, losing the recency signal for no frequency gain. The
  parent's blind index-0 removal was not a design debt — it was the right heuristic
  for this layout.
- **Ghost-buffer re-admission requires workload-a's missing property: frequent
  post-eviction re-access.** The ARC-style mechanism elevated re-admitted keys to
  warm-pool status, but those keys proved no warmer than ordinary new arrivals on this
  trace. The overhead (ghost scan per put, extra eviction on every ghost hit) was
  real; the benefit was not. Re-admission schemes belong on traces with scan-then-reuse
  patterns, not on workload-a.
- **The counter ceiling axis is conclusively exhausted.** Cap=4 was confirmed inert in
  cycle 0005; cap=2 now produces a marginal regression without any compensating gain.
  Neither the ceiling magnitude nor its existence is a meaningful lever on this workload
  at this cache size.
- **Four consecutive cycles of structural and parametric challenges have failed to
  dislodge the accumulated-stack local optimum.** The pattern is no longer ambiguous:
  the complementarity of cold-first eviction, positional recency, mid-point insertion,
  and episodic bulk decay is robust on workload-a.

## Open questions

- The per-op latency comparison recommended since cycle 0005 remains unmeasured. Does
  the simpler pure-LFU + wide-geometry path (cycle 0006 v003) offer a competitive
  efficiency trade-off despite its hit-rate deficit?
- Would ARC-style ghost re-admission perform differently on a trace with stronger
  post-eviction temporal locality, or is the mechanism structurally mismatched to the
  cold-first eviction family?

## Continuation

The current mechanism family is saturated on hit-rate for workload-a; further
eviction-policy tuning is unlikely to yield gains. The single most actionable next
step is the latency audit deferred across multiple cycles: measure per-op cost for the
full accumulated stack versus the simpler pure-LFU + wide-geometry alternative to
determine whether the hit-rate delta justifies the mechanism complexity. If latency
parity is found, the simpler path should become the new baseline and free up design
space for structural alternatives. Do not revisit ghost re-admission, post-decay cold
scans, or any further counter-ceiling variation — all three axes are now confirmed
closed on this workload.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
