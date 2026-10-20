# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_11-18-16-635/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried to narrow or cheapen the frequency-aware victim selection from cycle 0004: a two-choice sampled victim search (v001), the same counter restricted to stash-bucket victims on top of the deferred promotion (v002), and a record repack that was meant to shrink the segment (v003). None of the three reproduces cycle 0004's hit-rate gain, and the repack did not change the segment size it was meant to change. All three are labelled Winners, but each records a hit-rate or per-op regression.

## Patterns

- Narrowing the frequency-aware victim choice gave back its gain. Sampling two slots instead of scanning the bucket (v001) and restricting the counter to stash victims (v002) both left hit rate below the full-scan version from cycle 0004, so the retention effect depends on seeing the true lowest-counter resident.
- Cutting the victim search short did not make puts cheaper. v001's two-choice search raised put latency rather than bringing it toward the parent's level, which suggests scan length was not the main put-path cost. This partly answers cycle 0004's open question about put-side cost; the cost source is still unidentified.
- The frequency signal is not a cheap add-on to the deferred promotion. Stacking the counter on stash victims (v002) hurt both hit rate and both get and put tails, so the cycle 0004 combination with v001's promotion is not safe to extend without re-measurement.
- The repack did not land (v003). Segment and metadata size are unchanged, and hit rate is identical to the parent. Cycle 0004's per-op gain from the same layout idea therefore still has no mechanism behind it, and the size-reduction explanation offered there is not supported.
- Per-op movements remain unexplained. Get and put tails move in both directions across the cohort, and the attributions given are inferred rather than confirmed in source. This is the same pattern flagged in cycles 0001 to 0004.
- Every hit-rate signal again comes from workload-a. The second-trace checks requested in cycles 0003 and 0004 were not run here.

## Open questions

- Does full-scan lowest-counter eviction (cycle 0004 v002) keep its hit-rate gain on a second trace, and how much of its put cost is the scan itself?
- Is there a cheaper way to find the lowest-counter resident that keeps the full-scan ranking, such as maintaining the minimum incrementally?
- Why are all three variants Winners given their recorded regressions? The Winner basis is still not visible in the insights.
- Does the cycle 0004 per-op gain from the repack survive a layout change that actually reduces segment size?

## Continuation

Next cycle should go back to full-scan frequency-aware eviction from cycle 0004 (v002) rather than sampled or stash-only variants, and test it on a second, non-workload-a trace; focus on a cheaper way to find the victim. Keep the deferred promotion as the base. Drop the two-choice sampling (v001) and stash-only restriction (v002). Do not build on the repack (v003) until a variant verifiably shrinks the segment, and do not credit any per-op gain from this cohort until it is re-measured.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
