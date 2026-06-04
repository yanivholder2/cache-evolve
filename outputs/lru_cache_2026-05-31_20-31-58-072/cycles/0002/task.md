# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002

## Pareto front

- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort attacked three independent axes off the baseline LRU: segment
structure (v001 SLRU), replacement policy (v002 CLOCK), and link
representation (v003 u32+sentinel). Only v002 landed on the Pareto
front, because hit-rate is currently the sole scored dimension and CLOCK
was the only variant whose hit-rate signal was both non-trivial and
favourably positioned against the baseline sample.

## Patterns

- Two of three variants (v002, v003) made bets whose intended payoff
  lives on dimensions the fitness vector does not yet score (per-op time
  for CLOCK, metadata size for the u32 sentinel). Their structural wins
  are real but invisible to selection.
- The one variant whose bet did target hit-rate (v001 SLRU) showed a
  scan-resistance win in qualitative terms, but the variant and baseline
  were sampled at different cache sizes, so a same-size delta over LRU
  was not actually demonstrated.
- All three variants preserved the arena-slot-reuse pattern and the
  P1/P2/P3 invariants cleanly, suggesting the baseline's structural
  scaffolding is permissive of large representational rewrites.

## Open questions

- Without per-op time and metadata-size dimensions scored, how can the
  loop distinguish a real structural win (v002, v003) from neutral
  noise?
- Is v001's apparent hit-rate movement actually attributable to SLRU, or
  is it a sampling artefact of the size/trace pairing?
- Does CLOCK's slight recency-fidelity loss matter on traces less
  recency-skewed than LIRS:multi1?

## Continuation

Next cycle should treat the fitness vector itself as a bottleneck: until
per-op time and metadata size are scored, variants like v002 and v003
cannot be properly selected and the cohort risks collapsing onto
hit-rate-only tweaks. Hypothesis-writers should either (a) build on
v002's CLOCK base with hit-rate-recovering refinements (e.g.
two-handed CLOCK, CLOCK-Pro-lite) that compete on the one scored axis,
or (b) explore SLRU-style segmentation parameter sweeps anchored to a
shared baseline size so the same-size delta is actually measurable.
Drop pure representation-only repacks (v003-style) until a metadata-size
dimension exists.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
