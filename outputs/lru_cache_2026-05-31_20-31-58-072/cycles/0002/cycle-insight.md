# Cycle 0002 Insight

## Cohort outcome

All three variants targeted the only scored axis (hit-rate), responding
to cycle 1's call to compete on it. Two built on v002's CLOCK base
(GCLOCK aging counter, CLOCK-Pro-lite cold/hot + ghost ring) and one
rebuilt SLRU on the baseline arena. Only CLOCK-Pro-lite (v002) landed on
the Pareto front; GCLOCK regressed and the SLRU rebuild again failed to
establish a matched-size delta.

## Patterns

- Among CLOCK extensions, what the extra per-slot state encodes
  determined the sign of the result: state that records recency (Cold→Hot
  on second touch, plus evicted-key memory) helped, while state that
  records frequency (saturating aging counter) hurt. On a recency-skewed
  trace, frequency-leaning bits actively shield stale-popular keys and
  cold-scan inserts.
- Layering complementary mechanisms beat widening a single signal:
  combining a promotion class with a small ghost ring closed two
  distinct gaps (re-reference protection and just-evicted recall) that
  no single-axis tweak addressed.
- The cohort hit cycle 1's same-size comparability trap a second time:
  the SLRU variant ran at a capacity disjoint from its baseline's
  sampled sizes, so the SLRU-vs-LRU question is still open after two
  cycles of attempts.
- The fitness-vector bottleneck flagged in cycle 1 (no per-op-time or
  metadata-size dimensions) is still in force; the cohort sidestepped it
  by uniformly competing on hit-rate rather than addressing it.

## Open questions

- Is CLOCK-Pro-lite's win specific to lirs:multi1, or does the
  promotion-plus-ghost-ring pairing hold on traces with different
  re-reference-distance profiles?
- Can the SLRU mechanism ever be evaluated honestly without first fixing
  the sampling discipline so parent and child share capacities?
- What further refinements to CLOCK-Pro-lite (ghost-ring sizing, Hot
  demotion aggressiveness, second-pass policy) extend the win without
  re-introducing the frequency-bit trap GCLOCK fell into?

## Continuation

Treat cycle-2 v002 (CLOCK-Pro-lite) as the new working champion and
build the next round on it: tune ghost-ring size, explore alternative
Hot-demotion rules, or layer a recency-anchored admission filter. Drop
the GCLOCK / frequency-counter lineage entirely until other workloads
are scored — on a recency-skewed trace it is a structural loss. Do not
queue another SLRU attempt unless the next cycle can guarantee matched
capacities between variant and baseline samples; otherwise the result
will be uninterpretable for the third time. The fitness-vector
narrowness remains the unaddressed meta-bottleneck and should be named
again as a constraint for hypothesis-writers.
