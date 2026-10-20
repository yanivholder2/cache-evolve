# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort made three small, mostly put-side changes: an early exit in the frequency victim scan (v001), a word-level bucket scan that replaces the per-slot clock walk on top of the per-bucket mask (v002), and a bulk-add rewrite of the 2-bit re-reference aging (v003). All three kept hit-rate and metadata unchanged; the only movement was in timing, and it was mixed.

## Patterns

- Hit-rate held across the cohort, so these changes are behavior-preserving. Each one is an implementation-level rewrite of an existing mechanism, not a new retention policy.
- The per-bucket mask follow-up (v002) recovered put median time that the bit-by-bit clock walk had lost in Cycle 0004, which supports the idea that the scan form, not the mask itself, was the source of that regression. Put tail did not improve, though.
- The frequency early exit (v001) gave only a small put gain, which suggests a zero-counter slot is rarely present when a put scans the bucket. The scan usually runs to completion, so this shortcut is not where the put-side cost lives.
- The aging rewrite (v003) made the put path slower. The scan-then-bulk-add form appears to add full-bucket passes on the common full-bucket put, so the aging rounds were not the dominant put-tail cost. Cycle 0004 pointed at them; this cycle's result argues against that diagnosis.
- Get-side timing moved in v002 with no change to the get path, and the same kind of unexplained get movement appeared in earlier cycles. Treat get-side gains as noise until re-measured.
- Mechanisms here are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Where does the put tail actually come from on full-bucket puts, if neither the aging rounds nor the zero-counter scan dominates it?
- How often is a zero-counter slot present at put time, measured directly? This decides whether the early exit is worth keeping at all.
- Does the word-level scan's put-median gain hold across runs, and can its p99 rise be traced to the same code path?
- Is the get-side movement in v002 real or layout-related?

## Continuation

Next cycle should keep the per-bucket mask and the word-level scan (v002) as the put-side base, and profile the victim search on full-bucket puts before changing it again. Drop the aging rewrite (v003) as a put-tail fix; the scan-then-bulk-add form is a net loss. Drop the zero-counter early exit (v001) as an axis unless a measurement shows it fires often enough to matter.
</content>


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
