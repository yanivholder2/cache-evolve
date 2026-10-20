# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tested three distinct axes: removing mid-point insertion to save shift cost (v001), introducing an SLRU two-queue partition separating probationary and protected items (v002), and doubling the decay magnitude to collapse the lukewarm counter tier faster (v003). Notably, both v002 and v003 improved hit-rate — the first time in four consecutive cycles that any variant has done so — while v001 regressed. v003 was selected as winner.

## Patterns

- **The hit-rate design space is not saturated after all; structural separation of recency tiers helps.** SLRU's explicit probationary/protected split improved hit-rate by preventing scan traffic from displacing the confirmed working set — validating the long-standing suggestion that an adaptive two-queue scheme could break the local optimum. The co-encoding of position and counter in a single queue is no longer the only viable model.
- **Faster lukewarm demotion is a genuine, if narrow, lever.** Doubling the decay step converts the counter-2 tier directly to cold in one warm-lock event, modestly enlarging the eviction-candidate pool. The gain's small size indicates this tier is real but thin relative to the hot core.
- **The latency audit (v001) answered the hit-rate side but not the latency side.** Back-append removal of mid-point insertion confirmed that mid-point placement provides non-trivial eviction-ordering value — it was not merely cosmetic. However, per-op-time was again not captured, leaving the efficiency trade-off that has been deferred since cycle 0005 still unresolved.

## Open questions

- v002's SLRU design improved hit-rate but did not win the cycle; how does it compare quantitatively to v003, and is it the stronger long-term branch?
- Should SLRU segment sizing and promotion threshold be tuned, or is the current split geometry already near-optimal for workload-a?
- Per-op latency remains unmeasured across all variants; without it, the mid-point insertion trade-off and SLRU's additional bookkeeping cost cannot be evaluated.

## Continuation

SLRU is the most promising branch to pursue: it improved hit-rate through structural means rather than parametric tuning, opening a new evolutionary direction that the single-queue family could not reach. Next cycle should explore SLRU further — varying segment size ratios and promotion thresholds — while treating v003's doubled decay as the new parametric baseline. The latency measurement gap is urgent: SLRU introduces non-trivial per-operation bookkeeping, and without per-op-time data the efficiency case for or against it is blind. Do not revisit back-append as a hit-rate strategy unless the fitness vector is extended to include latency.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
