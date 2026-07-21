# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle explored three independent axes — promotion-insertion ordering, forced eviction-policy override, and per-item metadata pruning. Both survivors were clean wins with no regressions: one accelerating the warm-up path through the tier hierarchy, the other eliminating structural overhead from every cached item.

## Patterns

- **Orthogonal wins compound cleanly.** The two winners touch entirely separate subsystems (promotion mechanics vs. item struct layout) with no shared code paths, making them strong candidates for stacking in the next cycle.
- **Insertion position had outsized impact.** A single argument flip from LRU-end to MRU-end insertion improved both hit rate and per-op time simultaneously, revealing that the baseline's placement choice was actively working against the tier hierarchy's warm-up logic.
- **Syscall and struct costs are low-hanging fruit.** The timestamp removal produced a dual dividend — smaller memory footprint and eliminated syscalls — with zero hit-rate penalty, suggesting the baseline carries more dead-weight overhead worth auditing.
- **Operational timeout masked the LFU hypothesis.** v002 was rejected by simulator timeout rather than by performance, so the merit of forcing LFU eviction remains genuinely unknown.

## Open questions

- Are there other struct fields or per-operation syscalls in the baseline that provide no eviction advantage on this workload, similar to `timestamp`?
- Is the LFU hypothesis worth retesting with a lighter trace or smaller cache size to get a fitness signal before committing to a full run?
- What happens when MRU-end insertion and timestamp removal are combined — do they interact, or does the improvement simply add?

## Continuation

Next cycle should stack v001 and v003 together as a combined baseline to test whether their independently measured gains are additive. In parallel, the LFU eviction hypothesis from v002 deserves a retry under a configuration that avoids timeout — either a faster trace or a reduced policy surface — before the idea is discarded. Further metadata audits similar to the timestamp discovery are likely to yield more free wins.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle pursued three directions simultaneously: confirming that Cycle 1's two winners stack cleanly, probing frequency-based early promotion, and bypassing a saturated target tier on stash hits. All three variants won, making this an unusually productive cohort — though the two new ideas each extracted hit-rate gains at a tail-latency cost absent from Cycle 1.

## Patterns

- **Stacking was confirmed additive.** The combined MRU-end insertion + timestamp removal variant preserved the prior hit-rate gain and reduced struct size by exactly the expected width, with no interaction between the two subsystems. The stacking hypothesis was fully vindicated.
- **Cycle 2's new wins trade hit-rate for p99 latency.** Both the frequency gate and the stash-bypass variants improved hit-rate but raised tail latency (p99 get, p50 put), in contrast to Cycle 1's winners which had no regressions. The shared mechanism is cascade evictions triggered by conditional promotion logic — each gate fires often enough to generate a steady stream of downstream demotions.
- **The frequency gate has the right idea but the wrong threshold.** A threshold of ≥ 2 fires aggressively enough that cascade churn negates much of the hit-rate value; the signal is real (the workload does contain re-accessed non-hottest items) but the sensitivity is too high.
- **Stash-bypass is the cleaner hit-rate win.** Skipping a saturated target on stash hits removes one unnecessary eviction per stash-hit and has a clear rationale; its p99 regression is narrower in scope than the frequency gate's broader per-op overhead, suggesting it has a better hit-rate-to-latency ratio.

## Open questions

- Can the frequency gate's threshold be raised (≥ 3 or ≥ 4) or made fullness-conditional to preserve the hit-rate signal while reducing cascade frequency?
- Do the stash-bypass and frequency gate improve overlapping or complementary item populations — and can they coexist without amplifying each other's cascade costs?
- Are the p99 regressions from both new variants workload-size-dependent, or would they disappear at a different cache-to-working-set ratio?

## Continuation

Next cycle should treat v001 (the stacked Cycle 1 baseline) as the new starting point and layer v003's stash-bypass on top of it, since stash-bypass is the higher-quality hit-rate win and has not yet been tested on the combined baseline. In parallel, the frequency gate from v002 deserves a retry at a higher threshold (≥ 3 or guarded by target fullness) to determine whether the idea is viable once the threshold is calibrated. Combining all three mechanisms — stacking, stash-bypass, and a tuned frequency gate — is the natural end-state to aim for, but the components should be validated pairwise first to detect interactions before committing to a three-way stack.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle stress-tested the MRU-end + timestamp-free base by layering three independent ideas on top of it: stash saturation bypass, a recalibrated frequency gate, and a vacancy-gated frequency promotion. The clearest win was structural — vacancy-gating gate-fires to open probing slots eliminated cascade evictions entirely and delivered a regression-free hit-rate lift. The threshold-tuning approach improved on the prior cycle's calibration but could not fully suppress tail-latency inflation. The stash-bypass, which had succeeded on the LRU-end base, reversed on the MRU-end base, revealing a previously hidden interaction between insertion position and bypass-fire rate.

## Patterns

- **Structural cascade elimination beats threshold tuning.** Vacancy-gating promotion fires so that no cascade can occur (v003) produced a clean two-dimensional outcome, while raising the frequency threshold to ≥ 3 (v002) only softened the latency regression without curing it. The root problem is architectural — a gate that fires into a full tier will always trigger a cascade regardless of how rarely it fires.
- **MRU-end insertion changes tier pressure dynamics across the board.** Stash-bypass was a hit-rate win on the LRU-end base but a regression on the MRU-end base because back-insertion makes target saturation the common case, so the bypass fires far more often, converting the probing tier into a spillover sink for stash traffic.
- **All three strategies can be ordered by cascade risk.** From safest to most aggressive: vacancy gate (zero cascades by construction) → threshold ≥ 3 (cascades suppressed by rate) → stash-bypass on MRU base (cascades amplified by insertion position). The hit-rate benefit and the latency cost track this ordering.

## Open questions

- Can the vacancy gate and the higher threshold be combined — gate fires only when probing has space AND counter ≥ 3 — to see whether precision and safety together push hit-rate higher than either alone?
- Is stash-bypass salvageable on the MRU-end base if guarded by a probing-vacancy check, analogous to what v003 did for frequency promotion?
- Does the vacancy gate's marginal hit-rate lift grow at higher cache occupancy (smaller cache relative to working set), where probing vacancies are rarer and gate-fires are naturally throttled more aggressively?

## Continuation

Next cycle should use v003's vacancy-gated promotion as the new starting point — it is the only variant this cycle that improved hit-rate with no regressions. The highest-value experiment is combining the vacancy gate with a threshold of ≥ 3 to determine whether restricting gate-fires to items with stronger reuse evidence amplifies the hit-rate gain. The stash-bypass direction should be retried, but gated on probing vacancy in addition to target fullness, mirroring the structural fix that made v003 succeed where v002 fell short. The unguarded frequency-gate idea (threshold-only calibration) appears to have a fundamental latency ceiling and should be deprioritized unless combined with the vacancy constraint.


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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle attempted to merge the two winning lineages from Cycle 0006 — the clock-eviction base (v003) and the three-way mechanism stack (v001) — by grafting them together in three combinations. No variant improved all dimensions: the full convergence attempt regressed on both hit-rate and latency, the isolated probing re-entry graft was regression-free but inert, and the target-path counter increment improved hit-rate at the cost of tripling median latency. Winners v001 and v002 were selected as the less harmful forward bases over v003's severe latency regression.

## Patterns

- **The two-chance clock and frequency-gated promotion share `lfu_counter` in a conflicting way.** The clock's early-exit advantage depends on cold items sitting at counter zero; its counter-decrement pass corrupts the frequency signal that promotion gates rely on. When the bucket is warm throughout (as it is on v001's counter-rich base), the clock's rotation becomes O(k·n) rather than amortized O(1), erasing the latency benefit that motivated the convergence attempt.
- **Counter increment on the common non-hottest target path is too expensive to be unconditional.** H03's mechanism fires on every qualifying get regardless of whether the promotion threshold is reached; on a Zipfian trace this is the most frequent access path, so even a small per-access overhead compounds into a large latency regression despite the genuine hit-rate gain.
- **Vacancy-gated probing re-entry is safe but persistently thin on a saturated cache.** The joint condition (probing item at threshold AND target has vacancy) rarely fires when the cache operates at high occupancy — the precise steady-state where Zipfian hot items cycle through probing. The mechanism has now failed to contribute meaningfully in two consecutive cycles under these conditions.

## Open questions

- Can the clock's early-exit mechanism be decoupled from `lfu_counter` — for example, with a separate boolean "chance bit" — so it can coexist with counter-based promotion gates without corrupting the frequency signal?
- Is there a condition under which the target counter increment (H03's mechanism) can be gated to avoid the common-case overhead, firing only when the item is near eviction risk rather than on every non-hottest access?
- At what cache occupancy level does the probing re-entry vacancy gate become effective enough to produce a measurable hit-rate gain?

## Continuation

The main structural finding is a conflict between the two lineages: the clock assumes counter-zero cold items while the promotion machinery requires non-zero counters for warm items, and these invariants cannot coexist when the same `lfu_counter` field serves both purposes. Next cycle should explore decoupling these signals — a separate "clock chance" bit (or a narrower decrement scope) that does not touch the promotion counter — as the direct path to composing clock eviction with frequency-gated promotion. The counter-increment-on-target-access idea (H03) has a confirmed hit-rate signal and should be revisited with a conditional guard that limits the overhead to the promotion-relevant minority of accesses. Probing re-entry on the clock base should not be pursued further until a workload or cache-size condition is found where the vacancy gate fires at a useful rate.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle explored three independent directions: bit-packed clock/frequency decoupling on the target tier (v001), frequency-gated stash bypass to probing (v002), and cold-preferring stash eviction on a prior base (v003). All three variants won — not because they improved hit-rate, but because none introduced severe regressions; v003 produced a meaningful latency improvement while v001 and v002 were neutral-to-slight-regression and fully neutral, respectively.

## Patterns

- **The CLOCK_BIT fix is correct but insufficient.** v001 resolved the counter-corruption bug from Cycle 0007 by encoding the clock signal in the high bit of `lfu_counter`, preserving the frequency gate. However, because CLOCK_BIT is set on *every* non-hottest target access, nearly all resident target items are perpetually "warm" — leaving the clock with no discriminating power over what LFU-minimum eviction already provides on this counter-rich base. The bit-packing technique works as designed; the problem is that the warmth signal fires too broadly.
- **The stash tier is a consistently weak lever for hit-rate on this workload.** Both v002 (warm-stash bypass to probing) and v003 (cold-preferring stash eviction) left hit-rate unchanged. The stash represents too small a fraction of total cache footprint, and the conditions that move items into stash (target-full cascades) rarely produce items with high frequency counts, so gate-based stash interventions fire too infrequently to shift the aggregate miss curve.
- **Stash cold eviction is latency-cheap and amortized safely.** v003's scan runs only at full-stash-bucket insertions — far less often than target-tier scans — confirming that the stash tier can absorb frequency-aware eviction logic without a tail-latency cost. This disproves the concern that any O(n) stash scan would replicate Cycle 0005's latency regression.

## Open questions

- Can CLOCK_BIT be set only at a higher frequency threshold (e.g., `lfu_counter >= 3`) rather than on every non-hottest access, so that the clock retains discriminating power without corrupting the promotion gate?
- With eviction policy (target), promotion gating, probing re-entry, stash eviction, and stash bypass all explored, what structural axes remain — alternative tier layouts, adaptive thresholds, or workload-specific tuning — that could move hit-rate?
- Would stash cold eviction (v003) contribute on a workload or cache size where the stash tier holds a larger share of total capacity?

## Continuation

The clock integration problem is now well-characterized: the warmth signal needs to be sparse, but the existing promotion machinery makes it dense. Next cycle should test a higher CLOCK_BIT threshold (set only when `lfu_counter` crosses a second boundary, not on every increment) or replace CLOCK_BIT with a genuinely independent boolean "chance bit" that is set only when an item is rescued from eviction, not on ordinary accesses. The stash tier has been explored from multiple angles without a hit-rate payoff and should be deprioritized. v003 is the cleanest forward base (latency improvement, no regressions on its parent lineage), and v002 is safe but adds nothing; future work should build on v003 if targeting latency, or revisit the clock idea with a sparser warmth-set condition if targeting hit-rate.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle probed three targeted changes: CLOCK_BIT sparsification at freq≥3 on the clock base (v001), max-frequency promotion-candidate selection replacing MRU-position (v002), and position-0 eviction bundled with the same freq≥3 sparsification (v003). All three variants were named winners by elimination rather than dominance — v001 recovered a genuine hit-rate lift at the cost of elevated put-path tail latency, while v002 and v003 both regressed on hit-rate with v003 also regressing on latency.

## Patterns

- **Workload-a's hot set is concentrated at freq 1–2, making freq≥3 a destructive clock threshold.** Both v001 and v003 applied the same sparsification boundary; v001 survived because LFU-minimum eviction still gave cold victims to expel, while v003's additional position-0 switch removed that fallback entirely, compounding the regression. The principle is confirmed: freq≥3 leaves too much of the working set permanently cold for this trace.
- **Sparsifying CLOCK_BIT creates a latency tension in hot buckets.** v001 successfully restored hit-rate discrimination, but the same boundary that makes CLOCK_BIT sparse globally makes it *dense* within the hottest buckets — items there accumulate three hits quickly, causing `clock_evict_prepare` to rotate more protected entries before finding a cold victim. The threshold trades uniform saturation for hot-bucket saturation, shifting but not eliminating the rotation overhead.
- **MRU-position promotion is already a reliable frequency proxy on Zipfian workloads.** v002 confirmed that substituting a max-lfu_counter scan for the MRU-back position erodes hit-rate marginally. On a stationary Zipfian trace, the most recently accessed item at the back of the bucket is consistently the item with the highest access evidence; cumulative frequency occasionally diverges for cooling-but-historically-hot items, and those divergences are the wrong direction to promote.
- **Position-0 eviction is not a safe drop-in for LFU-minimum when CLOCK_BIT is sparse.** v003's double change — O(1) position-0 eviction plus freq≥3 threshold — produced the worst outcome: the sparser threshold evicted more freq 1–2 items as cold, the position-0 rule offered no frequency-based recovery, and promotion churn under increased miss pressure inflated both hit-rate and tail-latency metrics simultaneously.

## Open questions

- Does freq≥2 as the CLOCK_BIT threshold recover the hit-rate gain of v001 while reducing hot-bucket rotation depth enough to contain the latency regression?
- Can a consecutive-rotation cap (abort `clock_evict_prepare` after k cleared bits and fall back to position-0) bound the tail latency without losing the discriminating power the sparser threshold provides?
- Is position-0 eviction worth retesting in isolation — keeping CLOCK_BIT at freq≥1 or freq≥2 — to assess the O(n) scan's contribution to latency independently of the threshold choice?

## Continuation

The immediate next experiment is freq≥2 as the sole change: apply it to the clock base exactly as v001 applied freq≥3, with no position-0 substitution. v001's insight and v003's double-regression together bracket the answer — the working threshold almost certainly lies at freq≥2 — and isolating the variable will confirm whether the hit-rate gain survives at a lower rotation depth. If the latency regression persists at freq≥2, the rotation-cap idea becomes the natural follow-on. Max-frequency promotion (v002's direction) should be deprioritized; MRU-position is effective on this trace. Position-0 eviction should not be bundled with threshold changes again; test it independently only after the threshold question is settled.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

This cycle probed three targeted adjustments: lowering the CLOCK_BIT warm-set threshold from freq≥3 to freq≥2 (v001), extending the two-chance clock pre-pass symmetrically to the probing tier (v002), and lowering the probing re-entry counter gate from ≥2 to ≥1 (v003). All three were named winners by elimination rather than by improvement — none produced a clear hit-rate gain, and two introduced hit-rate regressions, while individual latency improvements (v001 on put-path tail, v002 on put-path tail) were offset by hit-rate costs.

## Patterns

- **The CLOCK_BIT threshold search space is now exhausted by elimination.** freq≥1 saturated the warm set (Cycle 8); freq≥2 over-extended it because Zipfian access distributions concentrate heavily at exactly two accesses, reproducing the saturation problem in milder form (v001); freq≥3 remains the least-bad discriminator despite hot-bucket rotation overhead (Cycle 9). Simple arithmetic thresholds on the raw counter cannot simultaneously protect the Zipfian working set and maintain clock discriminating power — a structural change (rotation cap, density-adaptive gate, or independent bit) is needed.
- **Clock protection in the probing tier creates a hit-rate/latency trade-off rather than a win.** v002's probing clock pre-pass cuts put-path tail latency by front-loading cold-victim selection, but it stickifies warm probing residents — items that have earned CLOCK_BIT in probing are blocked from timely demotion, slowing the target→probing→target circulation that drives steady-state hit rate on this workload. The mechanism is directionally useful for latency but structurally antagonistic to promotion throughput unless gated by bucket occupancy.
- **The probing re-entry counter threshold does real filtering work.** v003 confirmed that ≥1 is too aggressive: items recycled to target on their very first probing hit bounce before accumulating enough tenure to justify the slot, increasing churn and degrading hit rate and get-path tail latency simultaneously. The vacancy condition alone is not the binding constraint — the counter gate contributes independent signal on this trace.

## Open questions

- Can a consecutive-rotation cap on `clock_evict_prepare` (abort after k cleared bits and fall back to LFU-minimum) bound hot-bucket tail latency without sacrificing the freq≥3 threshold's discriminating power?
- Can the probing clock pre-pass (v002's mechanism) be gated on bucket occupancy — firing only when the probing bucket is above some fill threshold — to preserve the latency saving while avoiding the promotion-stickiness that hurts hit rate?
- Are there alternative warm-set signals — such as a decayed counter or a per-item "recently eviction-rescued" boolean — that could provide sparsity without depending on absolute frequency counts that cluster problematically on Zipfian traces?

## Continuation

The threshold-tuning axis for CLOCK_BIT is closed: all integer boundaries (≥1, ≥2, ≥3) have been tested and none dominates. Next cycle should move orthogonally — either introduce a rotation cap as a pure latency fix on top of the freq≥3 base, or replace the threshold criterion entirely with a mechanism that fires on eviction rescue rather than access count. The probing clock idea (v002) has genuine latency value and deserves a second test with occupancy-gating to suppress the admission-blocking effect; this is a narrowly-scoped change to `promote_to_probing`. Probing re-entry at ≥1 is falsified and should not be revisited.


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
