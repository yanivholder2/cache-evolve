# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three refinements of the cycle 6 reuse-bit eviction incumbent were tried: a two-bit aged counter replacing the one-bit flag (v001), a changed victim scan order with second-chance clearing (v002), and letting keys bypass the admission filter when free slots exist (v003). Only v001 won, with a modest hit-rate gain on workload-a. v002 was identical to its parent, and v003 dropped slightly.

## Patterns

- Gains shrank sharply compared with the cycle 6 move. Most of the benefit came from using a reuse signal at all, and finer state (counters, aging) added only a little.
- Scan order and the all-referenced fallback (v002) had no effect. The eviction path is saturated in structure, not just in signal richness.
- Bypassing admission for free space (v003) hurt slightly. The admission filter is not redundant with reuse-bit eviction, because it also keeps cold keys out of empty slots.
- Aging counters on every scanned slot may cost CPU time. This is unmeasured, as is metadata size.
- Evidence still comes from one trace and one size, and authors could not read the overlay sources, so mechanisms and costs are inferred.

## Open questions

- Does the counter gain hold on other workloads and cache sizes, or is it noise-level on workload-a?
- Is the counter's eviction-scan cost worth a modest gain?
- With admission, placement and eviction all near saturation on workload-a, is any hit-rate headroom left on this trace?

## Continuation

Make v001 the incumbent but treat it as a small win. Drop scan-order and second-chance tweaks (v002) and admission-bypass ideas (v003); keep the admission filter unconditional. Further eviction-state refinement is now diminishing, so next cycle should validate on other workloads and cache sizes and add per-op-time and metadata-size to the fitness vector. Otherwise it should try a lever outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for different access patterns.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three refinements of the reuse-based eviction line were tried: a ghost table that restores a counter for recently evicted keys (v001), a non-mutating victim scan with periodic global counter halving (v002), and a second "protected" bit giving probation/protected tiers on the one-bit parent (v003). v001 won, but only by a very small hit-rate gain. v003 gained modestly over its parent, and v002 fell below its parent.

## Patterns

- Gains are small again. The reuse signal, the admission filter and the counter already capture most of what returning or reused keys offer on workload-a. Extra history (ghost table) and extra tiers (protected bit) add little.
- Replacing scan-time counter decrement with global timer-driven halving (v002) hurt. Local, scan-time aging is a useful idleness signal, so aging should stay tied to where the evictor looks.
- v001 and v003 both add state or scan work (ghost writes and lookups, multi-pass scan) whose cost is unmeasured. v001's near-noise gain is probably a poor trade for that.
- Evidence is still one trace and one size, with only hit-rate reported and no overlay sources readable, so mechanisms and costs are inferred.

## Open questions

- Is v001's Winner label meaningful, or noise-level selection as in cycles 4, 5 and 7?
- Does the SLRU-style two-tier idea (v003) stack with the counter incumbent, or does it overlap with it?
- Is any hit-rate headroom left on workload-a, or do other workloads and cache sizes need to be the test?

## Continuation

Treat v001 as a marginal incumbent at best. Drop global-timer aging (v002) and further history or ghost-state ideas. Eviction-state refinement (counters, tiers, scan order, aging) is saturated on workload-a. Next cycle should validate the incumbent on other workloads and cache sizes, and add per-op-time and metadata-size to the fitness vector so small wins can be judged on cost. If it explores new levers, pick ones outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three small refinements were tried: tighter ghost-table admission that skips cold victims and caps the restored counter (v001), dropping an incoming key when its whole bucket is hot (v002), and a compact one-bit reuse flag with a second-sighting bit-array admission filter (v003). v002 won, but its hit-rate gain over the parent was within noise. v001 slipped slightly and v003 gave only a marginal gain over baseline LRU.

## Patterns

- This is the fourth cycle in a row with noise-level gains. The Winner label here is probably selection noise, not a real improvement.
- Ghost-history tuning (v001) went the wrong way. Remembering cold victims appears to help, so filtering ghost entries by heat removes useful signal.
- Bucket-level scan resistance (v002) did almost nothing. All-hot buckets are likely rare on workload-a, or the counter and the second-sighting filter already protect hot entries.
- v003 shows a compact design reaches only a marginal gain. Its stated goals, metadata size and per-op time, were never measured, so its value is untested.
- The same gap remains as before. Only hit-rate on one trace and one size is reported, per-op-time and metadata-size are still missing from the fitness vector, and the authors could not read the overlay sources.

## Open questions

- Is any hit-rate headroom left on workload-a, or is every lever at the noise floor?
- Would the compact v003 design be worth adopting if its cost savings were measured?
- Do scan-heavy or other workloads show a benefit from bucket-level protection (v002) that this trace hides?

## Continuation

Treat v002 as a marginal incumbent at best, and drop further ghost-table tuning (v001) and bucket-level admission blocking. Stop refining admission, placement, eviction state and ghost history on workload-a. Next cycle should first add per-op-time and metadata-size to the fitness vector, and validate the incumbent and v003's compact design on other workloads and cache sizes. If it explores new levers, choose ones outside those areas, such as adaptive sizing or scan-resistance for different access patterns.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
