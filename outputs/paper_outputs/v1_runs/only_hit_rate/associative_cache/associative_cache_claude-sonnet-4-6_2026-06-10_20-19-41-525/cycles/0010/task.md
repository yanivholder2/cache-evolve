# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-19-41-525/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tested position-split decay on the accumulated single-queue stack (v001),
SLRU with wide-bucket geometry added (v002), and a full SLRU combination that merged
cold-first eviction, decay-by-2 confined to probationary, and wide buckets (v003).
All three fell short of the parent incumbent; v001 was selected as winner for having
the smallest regression.

## Patterns

- **Counter value, not bucket position, is the reliable frequency discriminator under
  cap-4.** Upper-half items span the full counter range 1–4, so sparing them with a
  gentler rate-1 decay preserves lukewarm items just as often as hot ones — the
  position split provides no useful signal that the counter itself does not already
  encode. Uniform rate-2 decay remains the better choice.
- **Wide geometry closes most but not all of the SLRU gap.** Adding the 4× wide-bucket
  overlay to SLRU (v002) confirmed geometry was suppressing v002's prior result, yet
  the design still trailed the single-queue family by a meaningful margin. The
  bottleneck is SLRU's demotion/promotion logic or its probationary-first eviction
  priority, not the bucket layout.
- **The strongest SLRU configuration yet still does not match the accumulated stack.**
  v003's three-way combination — SLRU tier separation, wide geometry, cold-first +
  decay scoped to probationary — produced the best SLRU result across any cycle, yet
  it was measured against ClassicLRU as baseline rather than against the parent
  incumbent, and it did not win. The single-queue family's implicit recency encoding
  (mid-point insertion + positional eviction) appears to carry less overhead than
  SLRU's explicit bookkeeping while achieving equivalent or better ordering.
- **The SLRU branch is converging toward a ceiling.** Two focused cycles of SLRU
  exploration (0008–0009) have each improved the design but have not crossed the
  single-queue threshold.

## Open questions

- What proportion of SLRU's deficit is attributable to the 50/50 segment split? A
  protected-heavy split (e.g., 25/75) might retain more of the working set per
  bucket.
- Per-op latency remains unmeasured. If v003's SLRU combination nearly matches the
  single-queue hit-rate, a lower per-op cost could make it the preferred
  implementation even without a hit-rate lead.
- Could a counter-value-based asymmetric decay (rate-1 for items at counter ≥ 3,
  rate-2 for items at counter ≤ 2) unlock the incremental gain that position-split
  decay failed to deliver?

## Continuation

The single-queue accumulated stack continues to hold. The SLRU branch has been
thoroughly explored and is approaching its apparent ceiling without overtaking the
incumbent — further SLRU geometry or eviction tuning is unlikely to be decisive. The
next cycle should execute the long-deferred latency audit: measure per-op cost for the
full accumulated stack versus the best SLRU configuration (v003 this cycle) to
determine whether the hit-rate delta justifies the mechanism complexity. If latency
parity or advantage is found for SLRU, it becomes the preferred baseline for further
exploration; if not, the single-queue branch should be declared the winner and
exploration should pivot to a qualitatively different policy family. Do not revisit
position-split decay — counter value is the correct discrimination axis.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
