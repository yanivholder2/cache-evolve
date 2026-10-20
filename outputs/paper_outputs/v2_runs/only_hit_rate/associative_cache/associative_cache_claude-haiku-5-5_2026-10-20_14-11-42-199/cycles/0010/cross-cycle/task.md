# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three structural changes: bucket repartitioning, an aging LFU policy, and a static-dispatch refactor of the bucket policy. Only v001 (fixed-width buckets) won, and its hit-rate gain was marginal. v002 and v003 showed no hit-rate movement and carried no measurements on other dimensions, so neither is judged yet.

## Patterns

- Hit rate was the only dimension measured in this cycle. Every Winner-or-not decision rests on it, and the gain that earned v001 its place was small.
- The latency and metadata-size effects the hypotheses predicted were not measured in any variant, so they are untested rather than refuted.
- Two of three variants left hit rate unchanged. For v002 this may mean the evaluation never selected the new policy; for v003 the refactor was meant to be neutral on hit rate, so no change is expected.
- Smaller bucket-conflict misses appear to be a minor share of total misses at this cache size, which limits how much further geometry changes can gain.

## Open questions

- Does the evaluation path actually select the aging policy in v002, or is the fixed aging period too coarse to alter eviction order?
- Does the per-operation cost of scanning wider buckets (v001) show up once per-op time is measured?
- Does the static-dispatch change in v003 reduce get latency when measured separately from put?

## Continuation

Next cycle should pursue changes whose effect can be checked on per-op time and metadata size, and include those dimensions in the fitness vector. Re-test the aging policy only after confirming the evaluation selects it. Drop further hit-rate-only geometry tweaks unless they are paired with a cost measurement.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried a per-bucket fingerprint filter (v001, the sole Winner), a u8 saturating age counter (v002), and a cache-wide insert-count trigger for the AgedLfu halving (v003). Only hit-rate was measured for every variant, and it did not move in any of them, so the Winner's selection is not explained by the recorded evidence.

## Patterns

- Hit-rate was again the only dimension reported, repeating the Cycle 0001 gap. No variant recorded metadata-size or per-op-time, so the cost-side claims behind all three hypotheses remain untested.
- Hit-rate was flat across the cohort. The v001 insight itself says its fitness vector matches the parent's, which leaves the Winner label unsupported by the data it cites.
- Both aging-related variants (v002, v003) returned no measurable hit-rate gain, a second null result for the aging policy after Cycle 0001's v002. Whether the evaluation actually selects the aging policy is still unresolved.
- The v002 hit-rate drop is described only qualitatively as slight, so the size of any regression cannot be compared across variants.

## Open questions

- Why was v001 marked a Winner when its only measured dimension did not change? Does the evaluation check fingerprint code is in the build it runs?
- Does the AgedLfu halving fire at all under the evaluation harness? How many halvings occur per run?
- Do the fingerprint filter's per-op-time savings and the u8 counter's metadata-size savings show up once those dimensions are measured?

## Continuation

Next cycle should add metadata-size and per-op-time to the fitness vector before judging any cost-motivated change, and should log whether the aging policy is active and how often it halves. Do not pursue further aging-period or counter-width tweaks on hit-rate alone. Keep the fingerprint direction only if a per-op-time measurement is included.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried a sampled bucket-victim scan capped at 8 slots (v001), a frequency-gated admission check on inserts (v002), and a two-region bucket split that evicts from the probationary region first (v003). Only v003 improved hit-rate on workload-a and became the Winner; v001 held hit-rate flat and v002 lowered it slightly.

## Patterns

- The one Winner is a scan-resistance change that alters which entries are displaced, not how the existing ranking is computed. This is the first cohort in the digest where hit-rate moved in a positive direction on a mechanism that was actually tested.
- Hit-rate was again the only measured dimension. Per-op time and metadata size were not recorded for any variant, so the cost side of v001 and v003 is still unknown, as in Cycles 0001 and 0002.
- Exact-minimum eviction appears to buy little: capping the victim scan left hit-rate unchanged. Scan cost is therefore a candidate for savings, but only if it is measured.
- The frequency-gated admission in v002 appears to compare estimates on different scales, so its null-to-negative result may reflect a flawed comparison rather than a weak idea.
- Two insights could not confirm from the overlay that the change ran. For v001, an identical hit-rate is also what an unapplied overlay would produce. This leaves the flat result ambiguous.

## Open questions

- Does the sampled-victim change in v001 execute under the harness? An identical hit-rate cannot distinguish "no effect" from "not applied".
- Would a shared frequency counter for both incoming and resident keys make the admission gate in v002 informative?
- What do the second region's metadata and per-op costs in v003 amount to once measured?

## Continuation

Next cycle should build on the scan-resistance direction from v003, and include per-op time and metadata size in the fitness vector before accepting any cost-motivated change. Confirm that each variant's overlay is active before reading a flat hit-rate as a null result. Drop the admission-gate direction unless its two frequency estimates are put on a common scale first.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried a second-hit protected tier on top of probationary-first eviction (v001), a sampled probationary victim scan capped at 8 rotating slots (v002), and a single shared frequency counter gating eviction-time admission (v003). Only v003 showed a hit-rate gain on workload-a; v002 was flat to slightly negative and v001 was negative. The recorded Winner is v002, which does not match the hit-rate evidence in its own insight.

## Patterns

- The only positive hit-rate movement came from the shared-frequency admission gate (v003), which compares incoming and resident keys on one scale. This supports the Cycle 0002 suspicion that the earlier admission loss came from a scale mismatch rather than from admission itself.
- The Winner label and the evidence disagree. v002's insight reports a negligible hit-rate dip that misses its own guardrail, while v003 is the variant with a gain. Either the Winner selection used a dimension not shown in the insights, or the label was applied in error. This should be checked before the Winner is treated as a precedent.
- Bounding the victim scan (v002) left hit-rate essentially unchanged, consistent with Cycle 0003's finding that exact-minimum eviction buys little. Its per-op-time benefit remains unmeasured.
- Stacking a second reuse signal (v001) lowered hit-rate. The insight points at the protection cap and demote-on-promotion path rather than the promotion trigger, so the failure is plausibly in the mechanism, not the idea of a second tier.
- No variant measured per-op time or metadata size, and all three insights say the overlays could not be inspected in this run. Flat or negative hit-rate cannot yet be read as a null result about the mechanism, and the cost side is still unknown, as in Cycles 0001 to 0003.

## Open questions

- Which dimension made v002 the Winner, given that v003 carries the only hit-rate gain? Is the Winner rule using a metric missing from the insights?
- Is the v003 gain reproducible across more than workload-a, and does its shared counter add measurable metadata or per-op cost?
- Would a protected tier survive if the cap did not force demotion on each promotion?
- Are the v002 and v001 overlays active under the harness? Without that, their results are ambiguous.

## Continuation

Next cycle should build on the shared-frequency admission gate from v003, and add per-op time and metadata size to the fitness vector before accepting any change. Reconfirm the Winner selection for this cycle before treating v002 as a precedent. Drop the bounded-scan direction as a hit-rate lever, and retry the protected tier only with a changed cap or demotion rule rather than the same promotion path.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried a shared-frequency admission gate on top of probationary-first eviction (v001, the Winner), a removal of demote-on-promotion in the protected half-bucket (v002), and inserting misses at the eviction-next position instead of MRU (v003). The two variants that address one-shot inserts both reported a hit-rate gain on workload-a; the protected-tier variant ended slightly below its parent.

## Patterns

- Both positive results target the same problem: one-shot inserts displacing entries with proven reuse. The admission gate (v001) filters them at miss time, and the eviction-next insert position (v003) lets them leave first. Whether these gains overlap or add is not yet known.
- The Winner's gain is recorded only qualitatively. No magnitude is given, so v001 and v003 cannot be ranked against each other on the evidence in their insights.
- Removing the forced demotion (v002) did not recover the loss from the protected tier in Cycle 0004. The protected cap itself, or the second-hit signal, is the more likely cause, consistent with Cycle 0004's suspicion.
- Hit-rate is still the only measured dimension in all three insights. Metadata size and per-op time remain unknown for the shared counter (v001) and the miss-position flag (v003), even though v003's hypothesis claims no new per-entry metadata.
- v002's insight says its overlay could not be read in this run, so its result is not confirmed against the actual diff. The other two insights do not state this.

## Open questions

- Are the admission gate (v001) and the eviction-next insert position (v003) redundant, or do they stack? Testing each alone against the parent would settle this.
- Does the v003 gain hold beyond workload-a, given the Cycle 0004 concern about single-trace results?
- What does the shared counter's metadata and per-op cost amount to once measured?
- Is the protected-cap failure in v002 a cap-size problem or a second-hit-signal problem?

## Continuation

Next cycle should pursue one-shot-insert filtering, testing the admission gate (v001) and the eviction-next insert position (v003) separately or in combination, and confirm the gain on a second workload. Include metadata size and per-op time in the fitness vector before accepting either change. Drop the protected half-bucket with demote-skip as a hit-rate lever, and do not retry it without a changed cap or second-hit signal.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried a bounded direct-mapped frequency counter behind the admission gate (v001), placing newly admitted entries at the front of the probationary slice (v002), and shrinking the protected region from about 75% to 50% with demotion unchanged (v003). None of the three produced a hit-rate gain on workload-a. v001 and v002 both lowered it, and v003 was flat, yet v003 is recorded as the Winner.

## Patterns

- Every variant regressed or stayed flat on the single measured dimension. The Winner label again does not match the hit-rate evidence, as in Cycle 0004. The selection rule, or a dimension missing from the insights, needs to be checked before v003 is treated as a precedent.
- Shrinking the protected region (v003) changed nothing measurable. Protected capacity is therefore unlikely to be the cause of the protected-tier failures in Cycles 0004 and 0005, which points toward the demotion or second-hit signal.
- Bounding the admission gate's frequency metadata (v001) cost hit-rate. The insight attributes this to three simultaneous changes (slot aliasing, u8 saturation, periodic halving), so the cause is unresolved. The Cycle 0005 question about the shared counter's metadata cost is still open, and this result does not settle it.
- Front-of-probation placement (v002) hurt, while Cycle 0005's eviction-next insert reported a gain. The two may use different meanings of "next" or "front," so the gain from the one-shot-insert direction is not yet secure. Placement semantics need to be pinned down before the two results are compared.
- Per-op time and metadata size were still not measured in any variant, so the cost side remains unknown, as in Cycles 0001 to 0005.

## Open questions

- Why was v003 chosen as Winner when its insight reports no measurable change and its own prediction failed?
- Is the one-shot-insert gain from Cycle 0005 tied to inserting at the victim end of probation or at some other position, and does v002's front placement contradict it?
- Does aliasing or u8 saturation drive the v001 loss? Separating the three changes would show whether a cheaper frequency store is viable at all.
- Is the second-hit or demotion rule the remaining cause of the protected-tier failure?

## Continuation

Next cycle should reconfirm the Winner selection before using v003 as a precedent, and should not pursue protected-region capacity as a hit-rate lever. Pursue the demotion or second-hit signal instead, and state the insert position explicitly, testing victim-end placement against front placement. Drop the direct-mapped, saturating frequency store as a single change; retry metadata savings only with exact keys or with one change at a time, and include metadata size in the fitness vector.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried inserting admitted keys at the probationary victim end (v001), a second-miss ghost list that admits a key only on its second miss (v002), and a non-strict shared-frequency admission predicate that admits on counter ties (v003, the recorded Winner). Only v002 reported a hit-rate gain on workload-a; v001 and v003 both lowered it.

## Patterns

- The one positive result is a recurrence gate. The ghost list (v002) refuses first-time inserts without a frequency counter, which is the exact-key, no-counter form that Cycle 0006 asked to be retried. It also extends the one-shot-insert direction from Cycles 0005 and 0006.
- The recorded Winner (v003) has a hit-rate regression, and the only gain (v002) is not the Winner. This is the same Winner/evidence mismatch flagged in Cycles 0004 and 0006, and it now recurs. The selection rule needs checking before any of these results is used as a precedent.
- Insert position is a live confound. Placing admitted keys at the victim end (v001) lost hit-rate, while Cycle 0005's eviction-next insert reported a gain. The likely reading is that a newly admitted key needs some residency before it becomes eviction-eligible, but the Cycle 0005 and 0007 placements have not been shown to mean the same thing.
- Tightening the frequency predicate to admit ties (v003) gave no benefit. Strict comparison looks like the better default under unbounded counters.
- Cost is still unmeasured everywhere. v002's ghost list adds per-key metadata and v003 and v001 touch the shared counter, yet none of the three reports metadata size or per-op time. The hypothesis-level cost claims remain unverified, as in Cycles 0001 to 0006.

## Open questions

- Why was v003 chosen as Winner when its insight reports a slight hit-rate regression?
- Does the v002 ghost-list gain hold on a second workload, and what is its metadata size at the same cache size?
- Is the Cycle 0005 eviction-next gain the same mechanism as v001's victim-end loss, or is the placement semantics different?
- Does the v003 tie result reflect the tie rule itself, or a shift in victim selection pressure? The admitted-on-tie count was not checked.

## Continuation

Next cycle should pursue recurrence-gated admission (v002), testing its ghost-list window and checking it on a second workload with metadata size and per-op time in the fitness vector. Drop victim-end placement for newly admitted keys as a hit-rate lever, and do not pursue tie-admission in the shared-frequency predicate. Reconfirm the Winner selection before using v003 as a precedent.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried victim-end placement for admitted keys (v001), pruning singleton history from the shared frequency map (v002, the recorded Winner), and quadrupling the second-miss ghost-list window (v003). All three report a hit-rate regression on workload-a, so no variant produced a gain, and the Winner's own insight describes a decrease.

## Patterns

- No variant improved the measured dimension. The Winner label (v002) again conflicts with its evidence, as in Cycles 0004, 0006, and 0007. The selection rule should be checked before v002 is used as a precedent.
- Victim-end placement for admitted keys (v001) lost hit-rate again, now in a second cycle after Cycle 0007's version. The Cycle 0005 eviction-next gain has not reproduced under this placement. Its insight suggests the earlier gain depended on back-of-probation placement, so the two results may not be comparable.
- Widening the ghost-list window (v003) regressed. Together with Cycle 0007's recurrence gate, this suggests the one-window setting is about right and a longer first-miss memory admits stale one-shot keys.
- Pruning singleton frequency history (v002) produced a small regression, not the metadata saving its hypothesis predicted. The change also has no hysteresis, so a full prune could run on many accesses. That is a plausible per-op cost, but it is not measured.
- Metadata size and per-op time are still unmeasured in every variant, as in Cycles 0001 to 0007. The fitness vector still lacks these dimensions.

## Open questions

- Why was v002 chosen as Winner when its insight reports a hit-rate decrease?
- Does the Cycle 0007 recurrence-gate gain (ghost list at one-capacity window) hold when the window is kept at one capacity but the rest of the setup is unchanged, or was that gain specific to its run?
- Does singleton pruning change the admission gate's behavior enough to explain the decrease, or is the loss from some other effect of the prune?
- How often does the prune's full pass run under the current map size, and what does that cost per access?

## Continuation

Next cycle should not pursue victim-end placement, ghost-window widening, or singleton pruning as hit-rate levers. Keep the one-capacity ghost window from Cycle 0007 as the base for recurrence gating, and retest its gain before building on it. Reconfirm the Winner selection before using v002 as a precedent, and add metadata size and per-op time to the fitness vector before accepting any memory-saving change.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried a first-miss ghost refusal placed in front of the shared-frequency admission gate (v001, the recorded Winner), eviction-next placement for keys that pass the gate (v002), and periodic halving of shared frequency counters with zero-entry pruning (v003). All three report a hit-rate regression on workload-a, so no variant produced a gain, and the Winner's own insight describes a loss.

## Patterns

- No variant improved the measured dimension. The Winner label (v001) conflicts with its evidence, as in Cycles 0004, 0006, 0007, and 0008. The selection rule needs checking before v001 is used as a precedent.
- Stacking a recurrence gate in front of the counter gate (v001) lost hit-rate. The counter comparison already bypasses one-shot keys, so the ghost list mostly delays admission for keys that would have been admitted. This does not contradict Cycle 0007's ghost-list gain, but it shows that the recurrence signal did not add information on top of the counter gate.
- Eviction-next placement for admitted keys (v002) regressed again, now paired with the gate. Victim-end placement lost in Cycles 0007 and 0008 as well, so placement changes that make newly admitted keys eviction-eligible sooner have not helped on this workload.
- Halving counters with zero-pruning (v003) removed history for every key seen only once per window. This is the same singleton-history loss that Cycle 0008 found from pruning, so the loss now shows up under a second mechanism and is more likely a property of the removal than of the prune's trigger.
- Metadata size and per-op time are still unmeasured in every variant, as in Cycles 0001 to 0008. The cost side of the fitness vector remains empty.

## Open questions

- Why was v001 chosen as Winner when its insight reports a hit-rate loss?
- Would a recurrence signal that replaces the counter rejection, rather than preceding it, behave differently from the stacked form tested here?
- Does halving without removing entries preserve the one-hit information that the zero-pruning step discards?
- Was the v002 mechanism (eviction-next makes admitted keys the first victim candidate) confirmed against the diff? Its insight says the overlay source was not readable, so the mechanism is inferred from the hypothesis.

## Continuation

Next cycle should not stack a first-miss refusal in front of the shared-frequency gate, and should not pursue eviction-next or victim-end placement for admitted keys. Drop zero-pruning in any aging scheme; if aging is retested, halve without removing entries and test it separately from the admission rule. Reconfirm the Winner selection before using v001 as a precedent, and add metadata size and per-op time to the fitness vector.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

The cohort tried choosing the probationary victim by shared frequency (v001, the recorded Winner), periodic floor-halving of shared counters (v002), and a ghost-list override that admits recurring keys without a counter check (v003). Only v001 reports a hit-rate gain on workload-a, and it is marginal and from a single trace and size. v002 and v003 both lost hit-rate.

## Patterns

- The Winner's evidence matches its label this cycle: v001 is the only variant with a reported gain, unlike Cycles 0004 and 0006 through 0009. The gain is small and single-trace, so it is a candidate, not a result.
- Both losers fail on the admission path. v002 floors counts so one-hit residents drop to zero, and the `max(1)` floor on incoming keys then lets one-shot keys beat them. v003 lets recurrence bypass the `incoming > victim` comparison. In both, a loosened comparison admits keys the strict counter ordering would have kept out, and hit-rate falls.
- Aging is still negative. Cycle 0009 found that zero-pruning hurt; v002 shows plain floor-halving without removal also hurts, so the removal was not the only cause. Halving at small magnitudes erases the one-hit versus no-hit distinction the gate relies on.
- The ghost-list result differs from Cycle 0007's recurrence gain. There the ghost list refused first-time inserts; here it overrides a counter rejection. The mechanism is different, so the Cycle 0007 gain is not contradicted, but the recurrence signal has not yet helped when it can skip the counter check.
- Cost is unmeasured again. v001 adds a probationary scan on every miss, v003 writes to the ghost list on every bypass, and v002 runs a full halving pass. None of the three reports metadata size or per-op time, and the fitness vector still lacks these dimensions.

## Open questions

- Does v001's marginal gain hold on a second trace and a second cache size?
- Does the extra probationary scan per miss cost enough per-op time to offset a small hit-rate gain?
- Does v003 lose because the counter check is skipped, or because ghost-list writes from one-shot keys push out useful ghosts? The variant insight proposes separating these.
- Does a floor of 1 for previously seen keys (`max(1, v / 2)`) keep one-hit history without the admission flip seen in v002?

## Continuation

Next cycle should keep shared-frequency victim selection (v001) as the base, and confirm it on a second trace and size with per-op cost in the fitness vector. Drop the ghost override that skips the counter comparison, but keep recurrence as a signal to retest with the counter check kept on the ghost path, since Cycle 0007's gain is still open. Do not pursue plain halving or zero-pruning; if aging is retested, keep previously seen counts at or above 1. Note that Cycle 0009's advice to halve without removing entries is incomplete, since plain floor-halving also lost hit-rate here.


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
