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
