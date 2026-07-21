# Task: cycle-insight-writer

Cycle: 0007
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007

## Winners

- v001
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0007\hypotheses\H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle attempted three refinements on top of v003's vacancy-gated frequency promotion: a stricter ≥ 3 frequency threshold, a vacancy-gated stash bypass, and an lfu_counter reset at stash-to-target promotion. No variant moved any measured dimension positively — every regression was either structural (overlay coverage gap silently dropping the parent's Item-struct compaction) or workload-dependent (the target scenarios simply do not occur often enough on this trace).

## Patterns

- **Overlay coverage gaps are a first-class failure mode.** Two of three variants touched only `dash_segment.rs` and silently reverted the parent's `item.rs` struct reduction, re-inflating per-item footprint and worsening tail latency. The regression did not come from the hypothesis under test — it came from incomplete inheritance of the parent's accumulated changes. Any multi-file parent change must be fully re-applied in descendant overlays, or the measurement conflates intent with omission.
- **The vacancy gate's ≥ 2 threshold appears close to optimal for this workload.** Raising to ≥ 3 produced a fractional hit-rate loss rather than a gain, indicating that a real cohort of genuinely reusable items sits between the two thresholds. The prior cycle's threshold-only (no vacancy guard) ≥ 3 result had a different base, so the comparison was confounded; on the clean vacancy-gated base, stricter filtering is net harmful.
- **The joint stash-bypass condition fires too rarely to be measurable.** Requiring both target-full and probing-vacant simultaneously at stash-hit time is apparently an uncommon joint event on this trace, making the bypass mechanism inert. The hypothesis was structurally sound but the workload does not generate the precondition frequently enough.
- **Counter-carrying oscillation is not a live pattern in this workload.** The lfu_counter reset was designed to eliminate premature re-promotion of stash-re-entrants with stale counts, but no hit-rate signal emerged, implying the oscillation scenario is too rare on workload-a to be detectable at this cache size.

## Open questions

- Is there a mechanism to verify at hypothesis-generation time that a child overlay carries forward all files modified by its parent, preventing silent structural regressions from coverage gaps?
- At what cache-to-working-set ratio does stash traffic increase enough that the target-full + probing-vacant joint condition fires at a detectable rate?
- If the ≥ 2 threshold is near-optimal, are there other promotion axes (e.g., recency combined with frequency, or a decay on the counter over time) that could improve selectivity without the crude threshold knob?

## Continuation

Next cycle should not further tune the frequency threshold — the ≥ 2 vacancy-gated gate from Cycle 3's v003 appears to be the local optimum for this workload. The overlay-coverage-gap finding is the most actionable output of this cycle: future hypotheses that descend from multi-file parents must explicitly enumerate all inherited file changes, not just the files the new hypothesis modifies. Rather than continuing to refine promotion sensitivity, the next cycle should explore an orthogonal axis — for example, eviction ordering within a tier (which item gets displaced when a full bucket must eject), or the promotion path for items accessed in the probing tier rather than the target tier. The vacancy-gate and MRU-end insertion are now stable anchors; the remaining headroom is likely in eviction selection rather than promotion gating.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle explored eviction-side improvements — replacing positional LRU victim selection with frequency-minimum selection — alongside a symmetric application of the vacancy gate to the probing re-entry path. All three variants won: the two LFU-minimum eviction variants improved hit-rate at the cost of per-op tail latency, while the symmetric probing re-entry variant added a small regression-free hit-rate lift.

## Patterns

- **LFU-minimum eviction is an independently validated lever.** v003, which applied frequency-minimum victim selection without any inter-tier promotion gate, confirmed that eviction selection alone improves hit-rate meaningfully over the baseline — isolating it as a distinct mechanism rather than an auxiliary to promotion gating. v001 then showed it also stacks additively on top of the vacancy-gated promotion base.
- **The O(n) eviction scan's latency cost is consistent and non-trivial.** Both LFU-minimum variants (v001 and v003) inflated put p99 regardless of what else was present, confirming the scan overhead is the dominant latency regression source this cycle — not promotion cascades or insertion-position interactions.
- **Symmetric vacancy-gated probing re-entry is safe but thin.** Applying the same vacancy gate to the probing-hit branch (v002) added a fractional hit-rate gain with zero regressions, confirming the pattern is structurally sound. The joint condition (counter threshold met AND target has space) fires infrequently enough on this workload that the gain remains incremental rather than transformative.
- **Eviction selection and promotion gating appear complementary, not redundant.** The ordering of hit-rate lifts — baseline < eviction-only (v003) < eviction + promotion gate (v001) — suggests the two mechanisms protect different item populations.

## Open questions

- Can the LFU-minimum scan's latency cost be eliminated or amortized — for example, by maintaining a per-bucket minimum-counter index or a small frequency-sorted auxiliary structure — so the hit-rate gain can be captured without the tail-latency regression?
- Does stacking all three mechanisms (LFU-min eviction + vacancy-gated target promotion + vacancy-gated probing re-entry) yield additive hit-rate gains, and what is the combined latency footprint once the scan is the only regressor?
- Would the probing re-entry path contribute more on a workload with higher probing residency or a smaller cache-to-working-set ratio, where demoted-but-reused items accumulate in probing at higher rates?

## Continuation

Next cycle's highest-priority direction is replacing the O(n) eviction scan with an O(1) or near-O(1) alternative — a maintained per-bucket minimum-counter slot or a small frequency-ordered auxiliary — to preserve the confirmed hit-rate gain without the tail-latency cost. Until the scan overhead is resolved, stacking further mechanisms on top obscures their individual contributions. v002's probing re-entry is regression-free and should be carried forward in any multi-mechanism stack. The full three-way combination (LFU-min eviction + both vacancy gates) is the natural convergence target once the scan latency problem is solved.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three directions: completing the three-way mechanism stack (v001), adding post-eviction counter decay (v002), and replacing the O(n) LFU-minimum scan with a two-chance clock algorithm from baseline (v003). The cleanest wins came from v001's additive stack and v003's orthogonal clock approach; v002's decay conflicted with the existing frequency-promotion machinery and regressed both hit-rate and latency.

## Patterns

- **Two-chance clock eviction solves the O(n) scan latency problem.** v003 achieved frequency-aware eviction — sparing warm items from displacement — without scanning the full bucket for a global minimum. By rotating items with non-zero counters to the MRU end and stopping at the first cold item, the clock terminates early on Zipfian workloads where cold-tail items accumulate, delivering a hit-rate lift alongside latency and metadata-size improvements rather than the latency regression that followed the LFU-minimum scan in prior cycles.
- **Timestamp removal multiplies other gains.** Dropping the `Instant` field from the item struct reduced per-item footprint by 40 %, improving cache-line utilization during bucket traversal and eliminating clock reads on the eviction path — both of which contributed to v003's latency improvement independent of the clock algorithm itself.
- **Counter decay is antagonistic to frequency-gated promotion on a stationary workload.** v002's post-eviction halving of `lfu_counter` directly undermined the ≥2 threshold gate inherited from v001: items that had legitimately earned promotion had their counters suppressed below the threshold, and the decay pass added a second O(n) scan at eviction time, doubling median latency. Decay addresses stale-popularity drift, a problem that does not arise on a stationary distribution.
- **The three-way mechanism stack (v001) confirmed additive stacking holds.** Probing re-entry combined with LFU-minimum eviction and vacancy-gated target promotion delivered the expected incremental hit-rate gain with no new regressions, validating the Cycle 0005 prediction. The two-chance clock (v003) now makes this stack's LFU-minimum component the obvious candidate for replacement.

## Open questions

- Does stacking v003's two-chance clock with the vacancy-gated promotion and probing re-entry mechanisms from v001 yield additive hit-rate gains, and does it preserve the latency advantage the clock provides over LFU-minimum?
- On a non-stationary workload where popularity shifts over time, does counter decay (v002's idea) become beneficial — and is there a guard condition (e.g., decay only when eviction rate exceeds a threshold) that makes it safe to combine with frequency-gated promotion?
- Does the clock's early-exit advantage over LFU-minimum grow with cache-to-working-set ratio, where a larger fraction of bucket items are cold-tail and the clock terminates sooner?

## Continuation

v003 is the new structural anchor: its combination of timestamp removal, MRU-end insertion, and two-chance clock eviction improves all three measured dimensions simultaneously from baseline, and it avoids the latency regression that has followed every O(n) scan variant since Cycle 0005. The highest-value next experiment is grafting v001's vacancy-gated target promotion and probing re-entry onto v003's base — this is the natural convergence of the two winning lineages and the most direct path to a variant that improves every tracked dimension without a latency trade-off. Counter decay should be deprioritized until a non-stationary trace is available to test it against.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
