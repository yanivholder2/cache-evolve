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
