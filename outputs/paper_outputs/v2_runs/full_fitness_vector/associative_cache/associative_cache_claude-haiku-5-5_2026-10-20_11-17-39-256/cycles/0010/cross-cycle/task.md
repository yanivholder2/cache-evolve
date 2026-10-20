# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three structural changes: wider buckets via the geometry layout, midpoint promotion of front-half hits, and bucket-local ghost memory for reinsertion. None of the three landed as a Winner. The one with a modest hit-rate gain (wider buckets) paid a small get-path cost, and the other two lost hit-rate on workload-a.

## Patterns

- Changes that move where items sit within a bucket (midpoint promotion, midpoint reinsertion) hurt hit-rate on this workload, and both add shifting work to the hot path.
- Widening buckets was the only change that improved hit-rate, but the gain was small, and the get path paid for the longer linear scan.
- Both bucket-local changes ran into the same suspected limit: their memory windows (back-half protection, the small ghost ring) may be too short to reach the reuse distances in workload-a.
- Put-side and get-side latency moved in ways the diffs do not fully explain in two variants. Put-side measurements should be re-verified before any variant is judged on them.
- Metadata-size did not register any change, so the size probe may be missing the new per-bucket state.

## Open questions

- Does the wider-bucket hit-rate gain hold across other traces, or is it specific to workload-a's conflict pattern?
- Is the ghost ring too small to matter, or is the reinsertion idea itself weak on this trace?
- What is driving the unexplained put and get latency shifts, and are they noise or layout effects?

## Continuation

Next cycle should keep the bucket-width direction and re-measure it before building on it. Drop midpoint promotion as currently specified. Before dropping ghost memory, test it with a longer ghost window or a hit-count gate, and make sure the size probe covers the new state.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried a fingerprint prefilter on bucket lookups, a longer bucket-local ghost ring, and frequency-based victim selection using per-slot access counters. Only the frequency-based victim selection (v003) landed as a Winner, and it gained hit-rate on workload-a at a cost in per-op time.

## Patterns

- Frequency signals helped hit-rate where recency-only ordering had not. Re-referenced keys survived bursts of one-shot inserts, which is the pollution pattern the bucket-local work has been chasing.
- The gain came with a victim search that scans the full bucket on put, so the cost sits on the hot path. The per-op regression is the main thing the Winner has to fix.
- Lengthening the ghost ring did not lift hit-rate. This closes the question cycle 0001 left open about whether the ring was simply too short, and points to the reinsertion idea itself as the weak part.
- Adding a fingerprint prefilter to 32-slot buckets added a second linear scan on get and extra write work on every put. Hit-rate was unchanged, so the regression is isolated to the layer itself.
- Metadata-size reported no change in any variant, including ones that clearly add per-bucket or per-slot state. The size probe does not appear to count the new state, so it should not be used to judge storage cost this cycle.
- Get and put latency movements were single-run and in some cases not explained by the diffs. Treat the size of any timing gain as uncertain until re-measured.

## Open questions

- Can the frequency victim search be made cheaper (tracking a running minimum, or fewer ways) without losing the hit-rate gain?
- Does the frequency gain hold on traces other than workload-a, or is it tied to its one-shot insert pattern?
- What does the size probe need to count so that per-bucket and per-slot state shows up in metadata-size?

## Continuation

Next cycle should build on frequency-based victim selection and attack its put-side cost first, keeping hit-rate fixed. Drop the fingerprint prefilter and ghost-ring length as axes. Do not tune ghost-ring length further.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes against the frequency-based parent from Cycle 0002: caching the minimum-counter slot for faster victim lookup, merging the victim search with a per-eviction counter halving, and a second-chance reference-bit scheme for victim selection. Only the second-chance variant (v003) landed as a Winner, gaining hit-rate on workload-a and lowering put cost, at the price of slightly higher get cost and larger per-item metadata.

## Patterns

- Both frequency-line variants that tried to speed up the put-side victim search (cached minimum, merged scan with halving) failed to help. Each one either slowed the put path or lost hit-rate, so the put-side cost from Cycle 0002 is not solved by these mechanisms.
- The cached-minimum shortcut was undone by its own maintenance: counter increments on get hits keep invalidating the cached slot, so puts still fall back to the full scan and pay the bookkeeping cost on top.
- Adding a miss-driven decay on top of the existing periodic halving erased the frequency protection too quickly. Aging is already bounded by the periodic schedule, so extra decay is the wrong lever.
- Re-reference-aware victim selection through a second-chance bit retained re-referenced keys through one-shot bursts, the same pollution pattern the frequency work targeted. It gained hit-rate and reduced put cost, which is the first cohort result that improved both at once.
- The Winner's per-item metadata grew, and the get path picked up a small extra write per hit. Metadata growth suggests more state than the single bit the hypothesis specified.
- The Winner's mechanism is inferred, not confirmed: its source overlay was not readable this cycle, so the attribution rests on the hypothesis, fitness vectors, and metadata delta.

## Open questions

- Is the second-chance gain real and mechanism-driven, or does it come from something else in the overlay? The overlay needs to be checked before the attribution is relied on.
- How many bits of per-item state does the Winner actually use, and can the metadata growth be cut back to a single bit without losing the hit-rate gain?
- Does the second-chance gain hold on traces other than workload-a?
- Can the get-side write on every hit be avoided, for example by setting the reference state only on some hits?

## Continuation

Next cycle should build on the second-chance reference scheme: confirm the mechanism, tighten the per-item state toward one bit, and cut the get-side write before adding anything else. Drop the cached-minimum shortcut and the per-eviction halving as axes; neither has a path to a win on the current policy. Keep the periodic halving schedule as is.
</content>


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes on the second-chance and frequency line: moving the reference bit from each item into a per-bucket mask (v001), narrowing the frequency victim search to a rotating window (v002), and replacing the reference bit with a 2-bit re-reference value per slot (v003). All three landed as Winners, but they win on different axes: v001 saves metadata at a time cost, v002 buys put latency at a hit-rate cost, and v003 gains hit-rate with a put-tail cost.

## Patterns

- Retention signals keep helping hit-rate on workload-a when they separate re-referenced items from one-shot inserts. Both the per-bucket mask (v001, same hit-rate as parent) and the multi-level re-reference value (v003) preserve or extend that effect.
- Put-side cost is the common thread. Shrinking the victim scan (v002) reduced put latency but weakened the frequency protection, which suggests the full-bucket view on eviction is what carries the hit-rate gain.
- Moving reference state out of the item and into the bucket (v001) traded memory for time: get and put both slowed, so per-bucket state is not free on either path.
- The multi-level re-reference value (v003) pushed the put tail up through its aging rounds, so the policy's cost has shifted from the get-side write toward the bounded aging scan on full-bucket puts.
- Taken together with Cycle 0003, reference state is moving in two directions at once: v001 pushes toward fewer bits per item, while v003 adds bits per slot for more levels. The cohort has not yet shown which direction is worth its cost.
- The mechanisms for the put and get timing shifts are inferred from diffs and fitness vectors, not confirmed from overlay code, because the overlays could not be read this cycle. Treat the timing attributions as provisional.

## Open questions

- Is the per-bucket mask's time regression a fixed cost of the bucket-word write, or can the bit-scan in the clock walk be made cheap enough to keep the metadata saving?
- Does the 2-bit re-reference gain hold on traces other than workload-a and at other cache sizes?
- Can the put-tail cost of the aging rounds be cut by capping rounds or by tracking the aging state, without giving back the hit-rate?
- Does narrowing the victim search lose hit-rate on other traces too, or is the loss specific to workload-a's one-shot pattern?

## Continuation

Next cycle should pursue the re-reference value (v003) and attack its put-tail cost first, keeping hit-rate fixed. Keep the per-bucket mask (v001) only as a metadata option, and only if a follow-up can recover its get and put time. Drop the rotating-window victim search (v002) as a put-side fix; cheaper bookkeeping for the true global minimum is the better direction.


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


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

The cohort tried three changes: a rotating start cursor for the equal-minimum victim scan (v001), halving the slots per bucket while doubling the bucket count (v002), and lowering the saturation ceiling of the access counter to 3 (v003). Only v002 shows a clear, direction-consistent gain, on the put path. Both listed Winners carry weaknesses: v001 lost hit-rate, and v002's get-side claim is not supported by its median.

## Patterns

- Narrower buckets is the first put-side change in several cycles with a clear effect. Halving the victim scan width lowered both put median and put tail with hit-rate flat. This sits uneasily with Cycles 0007–0009, where shortcuts to the scan and aging did not move the put tail. The likely reading is that the put cost scales with scan width itself, and earlier changes did not reduce that width.
- Tie-break order among slots with equal minimum counters does not protect hit-rate. Rotating the start of the scan (v001) lowered hit-rate, so index-order bias is not the cost. The exact-minimum rule is what matters, and the order in which equal slots are visited is not a lever.
- The frequency ceiling can be lowered without losing hit-rate (v003), so the victim search does not need the full counter range at this size. Both the get and put paths got slower, though, and the insight cannot attribute that to a mechanism.
- Get-tail movement again appears without a get-path change (v001) or with a change that did not target get (v002, v003). The narrow-bucket get-tail tightening in v002 is the one get-side effect here with a plausible mechanism, and it is not confirmed by the median.
- Hit-rate did not move favorably in any variant this cycle. Shrinking buckets did not cost hit-rate on this trace, which is a useful negative result for the associativity trade-off.
- Mechanisms are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Does the put-side gain from narrower buckets hold across repeated runs, and how far can width drop before hit-rate pays?
- Why did shortcuts to the scan fail in earlier cycles while a narrower scan worked here? Is the difference the width itself or the mechanism?
- What drives the slower get and put paths under the ceiling-3 counter, and does it persist when the ceiling is kept but the counter update is unchanged?
- Is the get-tail tightening from narrower buckets real, given the flat get median?

## Continuation

Next cycle should pursue reducing victim scan width as the put-side timing lever, re-measured over three runs with hit-rate checked alongside, and should not chase get-side claims from it until the get median moves. Drop the rotating equal-minimum cursor (v001) and stop treating tie-break order as a hit-rate lever. Do not keep the ceiling-3 counter (v003) on its Winner label without repeated timing runs, since both of its timing paths regressed.
</content>


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
