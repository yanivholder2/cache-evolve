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
