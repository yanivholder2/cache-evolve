# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants were single-point tweaks to baseline S3-FIFO admission and ghost policy: a looser small-to-main gate (v001), ghost tracking of main-queue victims (v002), and initial freq=1 on ghost hits (v003). Only v002 won, and its hit-rate gain was marginal. The other two were slightly worse than baseline.

## Patterns

- The two losers both made main more permissive (looser promotion, pre-protected ghost re-admits). Both added pollution that displaced useful residents. Baseline's strict filtering looks well tuned for this LIRS workload.
- The only winner added information (more ghost coverage) without changing admission or protection rules. It was safe but nearly neutral, since baseline main eviction rarely drops soon-to-be-reused items.
- Metadata-size and per-op-time were missing from every fitness vector. Cost claims for all variants are unverified.
- All three variants edited a single branch or condition, so none explored structural levers.

## Open questions

- Why were the metadata and timing dimensions not reported, and can they be restored so ghost-size costs can be judged?
- Would a stricter gate or a different small/main split help, given that looser gating hurt?
- Is the v002 gain stable across traces, or is it noise on LIRS?

## Continuation

Build on v002 (main-victim ghosting) and drop further loosening of promotion or ghost-hit freq, which both hurt. Next hypotheses should probe other levers: small/main size ratio, ghost capacity, stricter promotion, or main-eviction policy. They should also be combined with v002. Make sure metadata and per-op-time are measured.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three single-knob tweaks were stacked on cycle 0001's v002 (main-victim ghosting): a smaller small queue (v001), a longer ghost queue (v002), and faster freq decay in main eviction (v003). Only v001, the small-queue shrink, beat the parent. The other two were slightly worse.

## Patterns

- The one winner changed capacity allocation (more room for main). It added no new admission or eviction logic. Both losers changed a filter or aging rule, and each made the cache slightly worse.
- The longer ghost loosened admission, which repeats cycle 0001's lesson. Anything that lets weakly-reused keys into main hurts, so ghost length acts as a filter.
- Faster main aging (freq halving) hurt a little. Main residents with high freq are usually reused, so the existing linear decrement looks about right.
- The gain was modest, and it was measured on one trace (LIRS) at one cache size. Metadata-size and per-op-time were again unmeasured, so cost claims remain unverified.
- The structural ratio knob (small/main split) moved the result more cleanly than the rule-level tweaks. Rule-level tweaks have now lost in both cycles.

## Open questions

- Is capacity/10 near the optimum, or does the gain keep growing as the small queue shrinks further? Where does it turn over?
- Does the smaller probation window hurt scan-heavy or other traces, where a larger small queue filters better?
- Would a shorter ghost (the opposite of v002), or a stricter ghost-hit rule, help?
- Why are metadata and timing still missing from the fitness vectors?

## Continuation

Build on v001 (the smaller small queue), and sweep the ratio around it, for example capacity/20 and capacity/8, to find the optimum. Drop the longer ghost and faster freq decay, and don't loosen admission or age main faster. Next, try the opposite direction on ghost size or a stricter ghost-hit rule, and check robustness on a second trace. Restore the metadata and per-op-time measurements.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

Three single-knob tweaks were tried on the cycle 0002 line: a further smaller small queue (v001), a shorter ghost queue (v002), and a lower freq cap in main eviction (v003). Only v001 won, with a small gain. v002 reported a small gain over its parent but is not among the Winners. v003 was slightly worse.

## Patterns

- The small-queue ratio gain is flattening. This halving gave much less than earlier steps, so the lever is close to exhausted on this trace.
- A shorter ghost helped, or at least did not hurt. That is the opposite of cycle 0002's longer ghost, which hurt. Ghost length now looks like a real admission filter, and shorter means stricter.
- Another attempt to age main entries faster (a lower freq cap) lost a little. Across cycles, ageing and decay changes have lost every time. This workload rewards keeping reuse history.
- Evidence still comes from one trace at one cache size, and every gain is small and may be within noise. Metadata-size and per-op-time were unmeasured for the fourth cycle running.

## Open questions

- Is the v002 shorter ghost additive with the v001 small queue, or does it just overlap with the stricter-admission effect?
- Where does the ghost-length gain turn over? Does it hurt when reuse distances are long?
- Does the very small probation window weaken filtering on scan-heavy or other traces?
- Are the small gains real, or noise from a single trace?

## Continuation

Keep v001's smaller small queue as the base and stop tuning the ratio. Pursue stricter admission instead: a shorter ghost (combined with v001), or a stricter ghost-hit or promotion rule. Drop all ageing and decay variants, including freq cap and halving. Validate on a second trace and cache size, and restore the metadata and per-op-time measurements.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

Three stricter-admission tweaks were tried: a shorter ghost combined with the small-queue shrink (v001), a promotion gate one freq level stricter (v002), and a ghost-hit fast path into main only if freq > 0 (v003). Only v001 won, by a marginal gain. v002 tied its parent and v003 was clearly worse.

## Patterns

- Admission-side sizing (small ratio, ghost length) now looks saturated. The shorter ghost and the smaller small queue are not additive, and the combined gain is tiny.
- Making the promotion gate stricter had no measurable effect. Either the gate is not a binding lever on this trace, or the change did not alter behaviour.
- Filtering ghost hits hurt the most. A ghost hit is strong reuse evidence, and with a very short small queue, keys sent back through small are evicted before they can prove themselves again.
- "Stricter admission wins" holds only for ghost length. It fails for the ghost-hit path, and it is untested for promotion.
- Only hit-rate was measured, again. Evidence is still one trace at one cache size, and the variant authors could not inspect the overlay source.

## Open questions

- Did the v002 gate change actually alter which keys get promoted, or was it a no-op?
- Is any remaining admission or capacity lever worth tuning, or is the S3-FIFO structure at its local optimum on this trace?
- Do these small gains hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored?

## Continuation

Stop tuning capacity ratios, ghost length, and promotion thresholds, and keep v001 as the base. Keep the ghost-hit fast path permissive, and consider loosening it only for ghost hits, not for general promotion. Explore structurally different levers, such as main-eviction policy or reinsertion rules, and verify that each diff changes behaviour. Validate on a second trace and cache size, and restore the metadata and per-op-time measurements.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three structurally different main-queue and ghost-path tweaks were tried: a one-time reinsertion grace for ghost-admitted entries (v001), protected-end insertion of high-freq small-queue victims into main (v002), and a per-key freq flag in the ghost, with re-admission at freq=2 (v003). None improved hit-rate. v002 is the Winner only because it tied its parent, while v001 and v003 were marginally worse.

## Patterns

- Nothing moved hit-rate. This is the second cycle running in which the best variant merely tied the parent. The S3-FIFO structure looks close to a local optimum on this trace.
- Both variants that gave ghost-admitted keys extra credit (a grace pass, a boosted freq) lost slightly. Ghost-hit information is already used well by plain admission, and extra protection keeps entries that stop being reused.
- The main insertion position (v002) looks like a weak lever. Either few victims reach the threshold, or main's reinsertion logic washes out the position difference.
- Variant authors still could not read the overlay source, so we cannot tell whether v002's change was a no-op. The tie is ambiguous.
- Metadata-size and per-op-time were again missing, and v003 added ghost state with no signal. Evidence is still one trace at one cache size.

## Open questions

- Did v002 alter behaviour at all, or is it equivalent to its parent?
- Is any remaining lever on this trace worth the effort, or are we measuring noise?
- Do these null results hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored so cost can be compared?

## Continuation

Treat v002 as a neutral base; prefer the simplest parent if it is cleaner. Drop ghost-hit bonuses, per-key ghost metadata, and main insertion-position tweaks. The next cycle should validate on a second trace and cache size, and restore the metadata and per-op-time measurements. Only try new levers (such as main-eviction scanning or reinsertion rules) if the diff is checked to change behaviour.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Three variants were tried. Two changed main-queue eviction: a bounded lookahead that picks the lowest-freq entry (v001), and a cap on reinsertions per eviction (v002). The third (v003) combined known admission settings: main evictees go to the ghost, a very small small queue, and a shorter ghost. Only v003 won, with a hit-rate gain. Both eviction variants lost a little.

## Patterns

- Every change to main-eviction mechanics lost hit-rate. Replacing reinsertion with lookahead lost, and capping reinsertion depth also lost. The unbounded second-chance reinsertion scan is doing real protective work.
- The one Winner is a consolidation of earlier admission gains (small queue shrink, shorter ghost), plus ghost insertion of main evictees. It adds no per-key state. This again points to admission sizing, not eviction, as the lever that moves this trace.
- Ageing freq in place for skipped entries (v001) fits the older finding that ageing and decay changes lose here.
- Cost was still unmeasured. Variant authors still could not read overlay source. Evidence is one trace at one cache size, so small regressions may be noise.

## Open questions

- Which part of v003 carries the gain: ghost insertion of main evictees, or the re-bundled sizing?
- Does v003's extra ghost work on `put` cost meaningful per-op time?
- Does the gain hold on a second trace or cache size?

## Continuation

Adopt v003 as the base. Drop main-eviction rewrites (lookahead, reinsertion caps, freq ageing) and keep the parent's reinsertion semantics. Next, isolate whether ghost insertion of main evictees helps, and validate on a second trace and cache size. Restore metadata-size and per-op-time measurements so cost can be compared.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three probes of the sizing-plus-ghost configuration were tried: a stricter small-to-main promotion test (freq>=2, v001), ghosting only main evictees that showed reuse (v002), and an ablation of main-evictee ghosting from the small-queue/short-ghost sizing (v003). v001 is the Winner only because it tied its parent. v002 lost a very small amount, and v003 gained over baseline.

## Patterns

- Sizing alone (small queue ~5%, shorter ghost) delivers the hit-rate gain over baseline. Main-evictee ghosting is not needed for it, so the cycle 0006 gain was mostly sizing.
- Filtering main evictees out of the ghost lost slightly. Evictees that look unreused still carry some re-admission value, and no ghost pollution showed up.
- A stricter promotion gate (freq>=2) again made no measurable difference. This is the second cycle with a tie on promotion strictness, so the gate is either not binding on this trace or the change is a no-op.
- Ghost content and ghost admission selection, like capacity ratios, look saturated. Every cycle's best variant has been a tie or a marginal gain.
- Variant authors still could not read overlay source. Cost dimensions (metadata-size, per-op-time) are still unmeasured. Evidence is one trace at one cache size.

## Open questions

- Does the freq counter ever reach 2 while an item sits in small? If not, the v001 tie is a no-op rather than evidence.
- How much does main-evictee ghosting add on top of sizing? v003 shows it is not required, but not whether it helps or hurts.
- Do the sizing gains hold on a second trace or cache size?

## Continuation

Treat the simplest sizing-only configuration (v003) as the base; it is as good as the more complex 0006 winner, so prefer it unless ghosting main evictees is shown to add something. Drop promotion-threshold and ghost-content filtering variants. Next, validate the sizing on a second trace and cache size, restore metadata-size and per-op-time measurements, and only try new levers if the diff is verified to change behaviour.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three levers were tried: adaptive small-queue sizing from ghost-hit and promotion feedback (v001), a higher frequency saturation cap (v002), and promotion from small to main on the hit path once freq reaches 2, bundled with sizing (v003). v002 is the Winner with a very small gain, likely within noise. v001 lost slightly. v003 improved on its parent, but the gain is confounded with the sizing change.

## Patterns

- Feedback-driven sizing lost to the fixed ~5% small queue. The fixed ratio looks like a real local optimum, and window-based signals were too coarse or noisy to improve on it.
- Raising the freq cap is the untested opposite of earlier cap reductions. It gives at most a marginal gain, so the cap is close to saturated as a lever.
- Hit-path promotion is the first structurally new lever in several cycles. It changes behaviour for certain, unlike the eviction-time gate tweaks that tied. Its gain cannot be separated from the sizing it was bundled with.
- Adaptive control and counter tuning are not paying off. Sizing and promotion timing are where signal has appeared.
- Cost dimensions (per-op-time, metadata-size) are still unmeasured, and hit-path promotion adds work to `get`. Evidence is still one trace at one cache size, and variant authors still could not read overlay source.

## Open questions

- Is v003's gain from hit-path promotion or from the sizing it was bundled with? Parent choice in this cycle makes the two inseparable.
- Is v002's gain real or noise?
- What does promotion on the hit path cost in per-op time?
- Do any of these results hold on a second trace or cache size?

## Continuation

Pursue hit-path actions, using the sizing-only configuration as the explicit parent so the effect of promotion is isolated. Variants to try include a different promotion threshold and a different insertion position in main. Drop adaptive sizing and further freq-cap tuning. Validate on a second trace and cache size, and restore per-op-time and metadata-size measurements.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Three probes were tried. Two isolated hit-path promotion from small to main (freq>=2 on the sizing parent, v001; a stricter freq>=3 gate with freq reset, v002). The third shrank the small queue to ~3% on baseline with a shortened ghost (v003). The promotion probes tied their parents exactly. v001 is the Winner only because of that tie. v003 gained over baseline.

## Patterns

- With sizing held fixed, hit-path promotion made no measurable difference. The cycle 0008 gain from promotion was therefore most likely the sizing it was bundled with.
- Promotion threshold and carried-over freq tie again. This is now the third cycle where promotion strictness or timing leaves hit-rate unchanged. Either eviction-time promotion already catches the same entries, or the gate is rarely binding on this trace.
- A smaller small queue (~3%) still beats baseline when paired with the shortened ghost. Sizing remains the only lever with consistent signal. No 5% comparison exists, so the optimum is not located.
- Exact ties suggest that diffs may not change behaviour. Variant authors still could not read overlay source, so this is unconfirmed.
- Per-op-time and metadata-size are still unmeasured. Evidence is one trace at one cache size.

## Open questions

- Is 3% better than 5%, or just another point on a flat curve? Both were never compared head to head.
- Is hit-path promotion a true no-op on this trace, or a no-op in the code?
- Does any sizing gain hold on a second trace or cache size?

## Continuation

Drop hit-path promotion, promotion thresholds and freq-reset variants; they have tied repeatedly. Next, sweep the small-queue ratio (for example 3%, 5%, 8%) with the same ghost and a shared parent, so the optimum is identified directly. Validate the best sizing on a second trace and cache size, and restore per-op-time and metadata-size measurements.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

Three single-constant probes were tried on separate parents: small queue to ~5% (v001), small queue to ~2% (v002), and a ghost 1.5x longer (v003). v002 is the Winner with a slight gain. v001 tied its parent exactly. v003 lost a very small amount.

## Patterns

- The small-queue sweep is flat: 2% gave a slight gain, 5% was indistinguishable from its parent. Sizing sensitivity has largely run out within the range of ~2-5%.
- Lengthening the ghost slightly hurt, which fits earlier notes that the shortened ghost is near its optimum. Ghost length is a low-sensitivity, tuned constant.
- The sweep was not head-to-head. The three probes used different parents (0008/v002, 0009/v001, 0008/v002), so 2%, 3% and 5% are still not compared on one parent.
- The v001 exact tie may mean the parent already rounds to the same effective queue size, or the diff does not change behaviour. This repeats the earlier suspicion about no-op diffs.
- Variant authors still could not read overlay source. Per-op-time and metadata-size are still unmeasured, and all evidence is one trace at one cache size.

## Open questions

- Is v002's gain real or noise? It is very small and has no replication.
- Do the effective queue lengths for 2%, 3% and 5% actually differ at this capacity?
- Does the near-flat sizing curve hold on a second trace or cache size?

## Continuation

Treat small-queue ratio and ghost length as saturated and stop tuning them; keep the ~2% configuration (v002) as the base. Next cycle should move to a structurally different lever, or validate the current best on a second trace and cache size. Any further sizing probe must share one parent and confirm the effective queue lengths differ. Restore per-op-time and metadata-size measurements.


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
