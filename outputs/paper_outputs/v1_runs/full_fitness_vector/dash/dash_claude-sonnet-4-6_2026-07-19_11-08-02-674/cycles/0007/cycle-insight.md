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
