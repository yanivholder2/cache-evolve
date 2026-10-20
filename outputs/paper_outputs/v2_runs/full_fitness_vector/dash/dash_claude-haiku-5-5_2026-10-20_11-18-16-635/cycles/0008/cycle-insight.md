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
