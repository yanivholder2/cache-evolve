# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three different kinds of change: narrowing the full-bucket victim scan to unreferenced slots (v001), skipping the access-counter store on saturated get hits (v002), and adding a doorkeeper admission gate on the put-miss path (v003). Only the doorkeeper gate moved hit-rate; the other two changed timing alone, and none of the three improved the put tail.

## Patterns

- The admission gate (v003) is the first cohort change to act before eviction rather than inside it. It gains hit-rate on workload-a by keeping one-shot keys from reaching the victim scan, which is the pollution pattern the retention line has been chasing since Cycle 0003.
- The gate's costs sit outside what it targets: its storage, its periodic whole-structure clear, and an extra field on the cache struct all show up in metadata and in put tail and get timing, none of which the gate itself touches.
- Narrowing the victim scan (v001) did not shorten the put tail and slowed put. This is the second cycle in a row where a put-side shortcut to the scan has failed (see the rotating window in Cycle 0004), so the scan length is not where the put-side cost lives.
- The saturated-counter store skip (v002) showed a get-tail gain with hit-rate and metadata unchanged. The gain is concentrated in the tail, not the median, and put numbers moved with no put-path change, so single-run tails look noisy.
- The mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle. Treat the attributions as provisional.

## Open questions

- Does the doorkeeper's hit-rate gain hold on traces other than workload-a and at other cache sizes?
- Can the gate's storage be made smaller and the whole-structure clear be spread out, so its put-tail cost goes away without giving back the admission effect?
- Is the get-tail gain from skipping the saturated-counter store real across repeated runs, or is it run-to-run variance?
- What is the actual source of the put tail on full-bucket puts? Cycles 0004 and 0005 blamed the aging rounds and the scan, and this cycle's scan change did not help.

## Continuation

Next cycle should pursue the doorkeeper admission gate (v003) as the main hit-rate lever, and attack its put-tail and metadata cost first, holding hit-rate fixed. Keep the saturated-counter store skip (v002) as a get-side candidate, but re-measure it with repeated runs before building on it. Drop the narrowed victim scan (v001) as a put-side fix.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three changes: a one-shot doorkeeper admission gate on the put-miss path (v001), a cached two-candidate victim pair that skips most full-bucket scans (v002), and a Fibonacci multiplicative hash in place of the bucket index (v003). Only v003 is listed as a Winner, and it won on timing alone: hit-rate did not move, and its own insight treats the timing shift as probably noise.

## Patterns

- Both changes that touched the eviction or admission decision hurt hit-rate. The doorkeeper rejected keys that recur soon after, and the cached pair let stale candidates hide the true lowest-frequency slot. Together they say the cohort's frequency protection is fragile in both directions.
- The exact minimum victim search is doing real work. v002 is the second cycle in a row where a shortcut to the victim search cost hit-rate (the rotating window in Cycle 0004 was the first). Put-side shortcuts to the scan have not paid off.
- The doorkeeper result contradicts Cycle 0006. There, the same admission-gate idea gained hit-rate on workload-a; here it lost hit-rate against its parent. The gain is therefore not established. Either the parent or the exact reject rule changed the outcome, and the cohort cannot tell which.
- The doorkeeper also added put and get cost and grew the cache struct, so the gate's bookkeeping costs land on paths it does not target.
- Bucket spread is not the miss driver under this policy. The hash change left hit-rate unchanged, so the remaining misses are not mainly a collision problem.
- Get-side movement appeared in two of three variants with no get-path change. The struct-layout explanation is plausible and recurs across cycles, but it remains unconfirmed.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Why did a doorkeeper gain hit-rate in Cycle 0006 but lose it here? Is the parent or the reject rule responsible?
- Does the v003 timing movement survive repeated runs? If not, the Winners label rests on nothing measured.
- Is the get-side movement caused by struct size or layout? A layout-only probe would settle it.
- Can the exact global minimum be kept cheaply by refreshing the candidate on counter updates, rather than by scanning?

## Continuation

Next cycle should not treat the doorkeeper as the main hit-rate lever until Cycle 0006 and this cycle are reconciled; if it is retried, test a looser rule that admits on second sighting, as v001's takeaway suggests, and keep the bitset out of the inline struct. Drop the cached two-candidate victim pair (v002) and keep the exact minimum search. Do not build on v003's hash change: re-measure it before keeping its Winner status, since hit-rate is unchanged.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes: a second-sighting doorkeeper on the put-miss path (v001), a Fibonacci multiplicative hash in place of the bucket index (v002), and moving the access counters out of the item into a parallel array (v003). No variant produced a clear hit-rate gain, and the listed Winners rest on small or single-run timing movement; v003's own insight records regressions.

## Patterns

- The second-sighting admission rule barely moved hit-rate, and its cost landed on the put-miss path. Across Cycles 0006, 0007 and this one, the doorkeeper has gained, lost, and now barely moved hit-rate, so the admission gate is not a dependable lever.
- The hash change (v002) is the only variant with a plausible timing movement, and it cost a small amount of hit-rate that may be noise. It partly re-measures Cycle 0007's timing claim and suggests that claim was partly real and partly noise. Bucket placement looks like a modest timing lever, not a hit-rate one.
- Moving the counters out of the item (v003) removed the get-tail movement seen in Cycle 0006's store-skip variant, while leaving retention unchanged. That supports the skipped counter store as the cause, and points against the struct layout as the explanation. The cost is an extra memory touch per get hit and a small put-side shift.
- Winners status should be reconciled. v003 is listed as a Winner, but its insight records get-tail and put regressions and no helped dimensions. Its Winner label is not supported by this cycle's evidence.
- Single runs dominate the evidence here. Several effects are within the run-to-run spread the parent insights flag, so none of the timing or hit-rate movements should be treated as settled.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Does the hash change's timing gain hold across repeated runs, and is its small hit-rate drop real?
- Is the get-tail gain from skipping the saturated-counter store reproducible with the counter kept in the item?
- Does the doorkeeper's hit-rate effect depend on the parent or on the exact reject rule, given the conflicting results in Cycles 0006 and 0007?
- What is the put-tail source on full-bucket puts? Five cycles of shortcuts to the scan and aging have not moved it.

## Continuation

Next cycle should keep the counter in the item and retest the saturated-counter store skip from Cycle 0006 with repeated runs, rather than moving the counter. Treat the hash change (v002) as a timing candidate only, pending a three-run spread, and drop it as a hit-rate lever. Drop the second-sighting doorkeeper as a hit-rate lever, and drop the separate counter array (v003).
</content>


## Write

Exactly one file: `cycle-insight.md` in the output dir.
