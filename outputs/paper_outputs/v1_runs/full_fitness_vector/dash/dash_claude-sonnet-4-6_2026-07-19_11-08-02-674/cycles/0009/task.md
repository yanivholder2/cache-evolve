# Task: cycle-insight-writer

Cycle: 0009
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009

## Winners

- v001
- v002
- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0009\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
