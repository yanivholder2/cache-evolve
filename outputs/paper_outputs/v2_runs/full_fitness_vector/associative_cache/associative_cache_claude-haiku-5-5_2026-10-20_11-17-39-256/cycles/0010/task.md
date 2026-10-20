# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010

## Winners

- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three changes: skipping the access-counter store on saturated get hits (v001), replacing the periodic global halving with lazy per-bucket halving (v002), and gating put-miss admission on the victim's counter at threshold T=1 (v003). None produced a clear hit-rate gain, and only v001 has any favorable timing movement, which is small. The listed Winners (v002, v003) have no helped dimensions in their own insights.

## Patterns

- Admission gating again showed no hit-rate effect. v003 refuses inserts only when every slot in a full bucket already has a counter of at least 2, so it touches few buckets. This matches Cycles 0006–0008: the admission gate has not produced a dependable hit-rate change.
- Decay scheduling is not the put-tail source. v002 kept hit-rate and metadata flat, yet moved the halving cost into ordinary put and get calls and made both tails worse. The whole-structure pass was not what drove the put tail, and the cost of decay is paid per operation wherever it runs.
- The saturated-counter store skip (v001) again shows a small get-tail gain, but its get median ticked up, so it does not beat its parent on the headline number. Its put-side movement is not explained by its edit and is probably run-to-run spread.
- Across the last several cycles, get-tail effects appear with no get-path change or with a change that adds per-op work. Single-run tails remain the dominant evidence, so these effects are not settled.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Does the saturated-counter store skip's get-tail gain survive repeated runs, and does it hold when the get median is also considered?
- How often does the v003 refusal branch fire on workload-a? A near-zero rate would mean the victim choice already captures the frequency signal.
- What is the put-tail source on full-bucket puts? Six cycles of shortcuts to the scan, aging, and decay have not moved it.

## Continuation

Next cycle should not pursue admission gating or decay-schedule changes as hit-rate levers. If the saturated-counter store skip is retried, measure it with repeated runs and judge it on the get median as well as the tail. Drop the lazy per-bucket halving (v002) and the T=1 admission gate (v003), and treat the Winner labels on both as unsupported by this cycle's evidence.
</content>


## Write

Exactly one file: `cycle-insight.md` in the output dir.
