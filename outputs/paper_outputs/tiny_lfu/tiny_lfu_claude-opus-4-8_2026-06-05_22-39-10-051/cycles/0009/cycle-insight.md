# Cycle 0009 Insight

## Cohort outcome

Forked from cycle 0008's twin winners, all three variants returned to the
hit-rate axis along the admission/eviction mechanics the prior digest pointed
to: an online hill-climbing controller for the protected/probationary split
(v001), making the eviction gate score the incoming key by its pre-increment
frequency (v002), and adding a fixed hysteresis margin to the admission
contest (v003). Both stricter-gate variants — v002 and v003 — took the Winner
slots on barely-perceptible upticks; the adaptive controller regressed.

## Patterns

- The segment boundary is conclusively not the lever, even when made
  dynamic. Static sweeps had already tied across 0.5–0.9; v001 shows an
  adaptive controller does no better and slightly worse — hill-climbing
  `protected_cap` only churns proven-hot nodes back to probation while
  oscillating around an optimum the static sweeps already sat on. Phase
  tracking finds no phases worth tracking on this trace.
- Tightening the admission gate is the one direction that still nudges
  upward, but only at the edge of noise. Both winners *restrict* who can
  evict a resident — v002 by removing the incoming key's self-count so a
  once-seen key can't arrive pre-inflated, v003 by demanding a margin so a
  newcomer must be two observations hotter. Each spares a handful of warm
  probationary residents from marginal, non-reused arrivals.
- The strict-guard theme is now firmly directional. Cycles 0007–0008 showed
  *loosening* gates loses; this cycle shows *tightening* them is the only
  thing that gains. The admission contest sits just slightly looser than
  optimal, but the headroom is tiny — these are correctness/consistency wins,
  not exploitable levers.

## Open questions

- The two winning tightenings (self-consistent scoring, hysteresis margin)
  are partially overlapping fixes to the same gate — do they stack, or do
  they both consume the same handful of tie-or-near-tie contests the way
  victim-sampling and promotion-gating overlapped in cycle 0006?
- The efficiency pivot the last two digests reserved — trimming the
  confirmed-redundant sketch depth or protected slack for free — was again
  skipped in favor of hit-rate. Is there appetite to actually fire it, or
  does the cohort keep reflexively returning to the saturated axis?

## Continuation

Hit-rate is essentially exhausted: the segment boundary is inert even when
adaptive (drop dynamic-sizing ideas entirely), and the only remaining lift is
sub-noise admission tightening. Both winners point the same way — stricter
gates win, looser gates lose — so if hit-rate is pursued at all, test whether
the two gate tightenings (v002 + v003) *combine* rather than proposing a third
independent guard. But the stronger move is the long-deferred efficiency
pivot: cycle 0008 proved sketch depth-8 redundant and protected sizing flat,
so probe trimming depth or protected slack to buy cost for free without losing
the ceiling hit-rate. Stop reaching back to the saturated hit-rate axis.
