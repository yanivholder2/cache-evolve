## Recurring wins

- Better estimate quality is the only lever that gave a clearly large hit-rate gain (a wider sketch in c3). Every other win in five cycles was marginal.
- Lifetime frequency is the right signal for workload-a. Lineages that keep long-run counts intact always do at least as well as ones that add freshness.
- Lineage stacking works: each cycle's single Winner (fast hash → sampled victims → wide sketch → dedup → SLRU) is a sound base for the next one, and the accumulated changes are compatible.
- Making counts honest (deduplicating the get-miss-then-put double increment) gave a small but real gain once collision noise was gone.
- The sketch is over-provisioned. A narrower sketch with smaller counters plus conservative update left every decision unchanged, so it is a likely memory win with no hit-rate cost.

## Recurring failures

- Sketch aging/halving lost or stayed flat in every form tried (short period with conservative update, long period alone, very long period on a clean sketch). On this trace it throws away useful history.
- Changing the contest rule (tie-admission, `>=`) hurt: on ties it mostly swaps one cold key for another. Keep strict `>` with keep-victim-on-tie.
- Widening or deepening the victim sample hurt. Without aging, the lowest estimate marks the youngest resident, not the coldest, so a wider sample evicts recent useful keys.
- Window restructuring did nothing, whether as a static 1% window or an adaptive hill-climber. The naive climber drifts toward an LRU-like region, which this LIRS-style trace punishes.
- Structural tweaks on clean estimates (SLRU split) mostly duplicate what min-estimate victim sampling already does, so gains stay marginal.
- Accuracy fixes don't stack: width and conservative update correct the same collision error.
- For five cycles running, per-op-time and metadata-size have gone unscored, so speed- and memory-oriented variants cannot win on what they actually improve.

## Open questions

- Once per-op-time and metadata-size are scored, which combination dominates (SLRU, narrow conservative-update sketch, hash-cached fast path), and what does SLRU's per-hit list work cost?
- How small can the sketch get (width, counter size) before hit-rate moves?
- Has workload-a reached a ceiling for TinyLFU-style policies, or is something left in the probation ratio or an admission window in front of SLRU?
- Does the plateau and the "aging is bad" result generalize beyond workload-a? Every conclusion so far rests on a single trace.
- A new newcomer still ties a resident that was accessed once, and loses. Is there a cheap fix for this that doesn't reintroduce the noise of tie-admission?

## Per-policy notes

- TinyLFU admission contest: strict `>` with keep-victim-on-tie is settled. The bottleneck was estimate quality, not the rule itself.
- Count-min sketch: wide (or narrow plus conservative update) with dedup is the current best. Never-aging counters are correct for this trace. Fast early-exit shortcuts (estimate ≤ 1) depend on there being no aging and must be re-derived if decay is ever added.
- W-TinyLFU window: neither fixed nor adaptive helped. Deprioritize.
- SLRU probation/protected: a marginal win that overlaps with victim sampling. Its value now depends on per-op cost, which is still unmeasured.
- Sampled-victim eviction: a modest default sample of the oldest entries helps a little. Widening it is exhausted.
