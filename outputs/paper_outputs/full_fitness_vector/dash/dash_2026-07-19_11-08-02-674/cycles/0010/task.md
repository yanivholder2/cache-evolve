# Task: cycle-insight-writer

Cycle: 0010
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010

## Winners

- v001
- v002
- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\hypotheses\H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
