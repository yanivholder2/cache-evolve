# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three kinds of change: caching the eviction victim per bucket as an incremental minimum (v001), packing the `DashSegment` fields to shrink the segment (v002), and re-admitting recently evicted keys from a ghost ring (v003). Only the packed layout produced a measurable reduction, and it came with per-op regressions. The victim cache and the ghost ring both made the put path more expensive, and the ghost ring also lost hit rate. All three are labelled Winners, but none of them has a clean case on the dimensions the cohort measured.

## Patterns

- Maintaining the lowest-counter victim incrementally (v001) did not pay for itself. Hit rate and metadata size matched the full-scan parent, while put and get tails got worse. This answers the cycle 0005 question about a cheaper way to find the victim: keeping the minimum up to date on every counter write costs more than the scan it replaces, so the scan itself should stay.
- The layout change (v002) delivered the footprint reduction it targeted and left eviction order alone, which supports the cycle 0005 and 0004 view that layout and replacement are separable. But the packed fields added decode work on the get and promotion paths, and both operations slowed. A real size saving is not yet a usable win.
- Ghost-ring re-admission (v003) lost a small amount of hit rate on workload-a. The re-admitted keys appear to displace residents without adding re-referenced hits, which is the same displacement concern raised about promotion in cycle 0003. Its put-path bookkeeping on every insert and eviction is an added cost with no offsetting gain.
- Per-op movements again do not match their mechanisms. v003's reported get gain sits on a path the hypothesis did not touch, and the insight itself calls it unconfirmed. This is the same pattern flagged in cycles 0001 to 0005.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace and stash-dominant checks requested in cycles 0003 to 0005 were not run here.

## Open questions

- The Winner basis for all three variants is still not visible. v001 and v002 record per-op regressions, and v003 records a hit-rate loss.
- Can the packed `DashSegment` keep its footprint reduction without the get and promotion decode cost, for example by packing only fields that are not read on the hot path?
- Does the ghost ring's cost show up in metadata size once its bytes are counted? It is currently unmeasured, so the put-path cost is not fully accounted for.
- Does full-scan lowest-counter eviction (cycle 0004 v002) keep its hit-rate gain on a second trace? That check is still outstanding.

## Continuation

Next cycle should drop incremental victim caching (v001) and keep the full scan as the victim-selection base, and it should test the full-scan frequency eviction on a second, non-workload-a trace as the cycle 0005 continuation asked. Pursue the packed layout (v002) only if the next variant removes the decode work from the get and promotion paths while keeping the smaller segment. Drop ghost-ring re-admission (v003) unless a later variant shows a hit-rate gain and reports the ring's bytes in metadata size.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three changes around the full-scan lowest-counter victim choice: an exact early exit from the victim scan on a zero counter (v001), halving every counter after each eviction (v002), and porting the saturating counter and full-scan victim choice onto the packed `Item<u64,u64>` record (v003). Only v003 showed a hit-rate gain, and it was partial. v001 and v002 have no helped dimensions, and all three are labelled Winners anyway.

## Patterns

- The full-scan, non-decaying counter remains the working base. Exiting the scan early (v001) left hit rate and metadata identical, so the victim choice was preserved. Decaying the counters (v002) lowered hit rate, which suggests the never-decaying counter is doing the retention work rather than just shielding stale residents.
- v003 is the first place the counter-based victim choice has been combined with a packed record. Hit rate rose, but by less than the hypothesis targeted, and metadata size did not change. Layout and replacement look separable again.
- Put cost still is not explained by scan length. The early exit did not make puts cheaper, which is the third cohort in a row where a shortened or cheapened victim search failed to lower put latency. The decay pass added put-path work and was the worst put result in the cohort, so the put-side cost appears to come from bookkeeping that runs on the eviction branch, not from the scan alone.
- Per-op movements again outrun their mechanisms. v001's get regression sits on a path the change does not touch, and v003's small get and put improvements are attributed to noise rather than the change.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 has now been deferred for four cycles.

## Open questions

- Does the v003 counter-plus-packed-layout hit-rate gain hold on a second, non-workload-a trace?
- Is the counter stored in spare bits or in the packed `u8` slot? The overlay source was not inspected, so this is unconfirmed, and it bears on the get and promotion decode cost raised in cycle 0006.
- Why are v001 and v002 Winners when they record no helped dimensions? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the victim-selection base and treat the packed-record port (v003) as the candidate to build on, but only after running the second-trace check that has been outstanding since cycle 0004. Drop counter decay (v002) and the early-exit scan (v001). Do not credit any per-op gain from this cohort until it is re-measured.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried three small, separate edits: doubling the saturating counter's per-hit increment (v001), packing the access counter into the `Item<u64,u64>` record (v002), and checking the target and probe buckets before the stash in `get` (v003). Only v002 and v003 moved a dimension, and each moved it in a different place: v002 reduced metadata size but hurt get and put, and v003 lowered get latency with no hit-rate change. v001 changed nothing measurable. All three are labelled Winners.

## Patterns

- Changing the counter's weight did nothing visible. Doubling the per-hit increment (v001) left hit rate and metadata identical to the parent, so the retention effect is not sensitive to the increment size at this setting. Its small get and put tail differences were attributed to noise.
- Packing the counter saved metadata but moved cost onto the hit path. v002 reproduced the footprint saving that cycle 0007 v003 suggested and kept hit rate flat, but the per-hit counter update on get and the packed decode on put both regressed. This is the same layout-versus-hot-path tension raised in cycle 0006, now with a concrete location: the counter update itself.
- Reordering the `get` lookup helped get latency without touching replacement. v003 removed the stash scan from the common target-hit path, which supports the cycle 0001 finding that the stash scan sits on the get path. Hit rate and metadata were unchanged. Its put movement is best read as noise, since the edit does not touch the put path.
- Stash-hit gets are still unmeasured. The stash-hit tail was the risk the hypothesis named, and it is not split out in the fitness vector, so the net effect of v003 is only partly known.
- Per-op attributions again go further than the evidence. v001 and v003 both explain small or unexplained movements as noise, and v002's hit-path explanation is inferred from the design, not confirmed against the overlay diff.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 has now been deferred for five cycles.

## Open questions

- Does the packed layout (v002) keep its metadata saving if the access counter stays unpacked for the hit increment and only cold fields are packed?
- Does the reordered `get` (v003) still win on stash-hit gets, once that tail is reported separately?
- Does a larger or smaller counter increment matter on a second trace, given that v001 was flat on workload-a?
- Why are all three variants Winners when v001 has no helped dimension and v002 and v003 each record regressions? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter as the base and pursue the `get` lookup reordering (v003) as the candidate most likely to carry forward, but only after the stash-hit tail is reported separately. Test packing with the hit counter kept unpacked (v002) before building on it. Drop the counter-increment change (v001) unless a second-trace run shows a gain. Run the second-trace check before crediting any per-op gain from this cohort.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
