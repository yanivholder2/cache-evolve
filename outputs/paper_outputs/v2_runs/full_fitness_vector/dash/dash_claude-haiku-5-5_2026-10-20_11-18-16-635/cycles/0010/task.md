# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three small edits around the full-scan, lowest-counter victim choice: sampling the counter increment on get hits (v002), reordering the `get` lookup to check target and probe buckets before the stash (v001), and seeding a new entry's counter at 1 instead of 0 (v003). None of the three moved a helped dimension. v002 lowered hit rate and did not improve get, and v001 and v003 left hit rate and metadata unchanged with small or unexplained per-op shifts. All three are labelled Winners.

## Patterns

- The victim choice is sensitive to how counts grow, not where they start. Seeding new counters at 1 (v003) was inert on hit rate and metadata, so the retention effect comes from hits accumulated after insertion. Sampling the increment on get hits (v002) did cost hit rate, which points the same way: hot keys need their counts to keep climbing.
- The premise that the per-hit counter write dominates get cost did not hold. v002 removed most of those writes and get latency rose instead. The get cost in the packed layout sits somewhere other than the counter update.
- The `get` reorder did not reproduce. Cycle 0008 v003 reported a get win on an earlier base, but v001 here moved get slower on both median and tail on the packed-counter parent. Hit rate and metadata were preserved, so the semantics held, but the latency claim is not carried forward.
- Put movements remain unexplained by their mechanisms. v001 and v002 both record put regressions on paths their edits do not describe, and v003's put shift is small enough to be noise. This is the same gap noted in cycles 0006 to 0008.
- Stash-hit gets are still not split out in the fitness vector, so the stash-hit tail that v001 flagged is still unmeasured.
- Every hit-rate signal in this cycle comes from workload-a. The second-trace check first requested in cycle 0004 is still outstanding.

## Open questions

- What drives get latency in the packed layout, if not the per-hit counter write that v002 removed?
- Does reordering the `get` lookup help or hurt stash-hit gets once that tail is reported separately?
- Why do the put regressions appear in variants whose edits do not touch put, and is the overlay diff doing something the hypotheses do not describe?
- Why are all three variants Winners when none records a helped dimension? The Winner basis is still not visible in the insights.

## Continuation

Next cycle should keep the full-scan, non-decaying counter and focus on how counts grow after insertion, not on their starting value or on sampling them. Drop the insert-time seed change (v003) and the on-hit sampling (v002). Do not carry the `get` reorder (v001) forward until the stash-hit tail is reported separately and the get regression is re-measured. Run the second-trace check before crediting any per-op or hit-rate result from this cohort.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
