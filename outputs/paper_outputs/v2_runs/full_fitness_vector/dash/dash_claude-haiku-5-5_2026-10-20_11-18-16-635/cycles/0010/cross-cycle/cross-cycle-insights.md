## Recurring wins

- Deferred stash-to-target promotion keeps a hit-rate gain across cycles 0003 and 0004, as long as the first-hit state stays cheap on stash hits.
- Full-scan lowest-counter victim choice is the only victim policy that has produced a hit-rate gain. Its base is a saturating counter that does not decay.
- Retention depends on hits accumulated after insertion. Changes to the starting value or the per-hit increment were flat, and sampling the increment cost hit rate.
- Layout and replacement are separable. Record repacking and packed layouts moved metadata size without changing eviction order, so layout work can be judged on footprint alone.

## Recurring failures

- Narrowing the victim search lost the gain. Two-choice sampling and stash-only restriction both fell below the full-scan result, so the retention effect needs the true lowest-counter resident.
- Keeping the minimum up to date costs more than the scan it replaces. The incremental victim cache raised put and get tails.
- Counter decay lowered hit rate, and a decay pass was the worst put result in the cohort.
- Protecting residents at insert time has not helped. Gating promotion position on fullness cost hit rate, and blocking first-touch admission also lost hit rate.
- Ghost-ring re-admission and per-slot tag filters lost hit rate or added put-path bookkeeping without a gain.
- Cutting the victim scan short or sampling it did not reduce put latency in three cohorts in a row. Put cost is not explained by scan length.
- Recency tie-breaks, residency-class tie-breaks, and overwrite counting moved nothing that helps.

## Open questions

- The second-trace check requested since cycle 0004 has never been run. Every hit-rate result so far is workload-a only, so no directional claim generalises yet.
- The get-path regression appears across many variants and may be shared. Re-measure the parent before judging any further get or put result.
- The stash-hit tail is not split out in the fitness vector, so the effect of reordering the `get` lookup is unknown on that path.
- Why are variants labelled Winners when they record regressions and no helped dimension? The Winner basis is not visible in any insight.
- Can the packed record keep its footprint saving while the hit-path counter stays unpacked?
- The overlay source has not been inspected in any cycle, so per-op attributions remain inferred and should not be credited.

## Per-policy notes

- Full-scan lowest-counter (saturating, non-decaying): the working base. Keep the full scan and avoid sampling, caching the minimum, or decaying counts.
- Promotion and admission: the deferred stash-to-target promotion is the candidate to build on. Admission delays lost hit rate on workload-a, and insert-position changes trade hit rate for protection.
- Tie-breaking among equal counters: age and residency-class tie-breaks were marginal or flat, so this is a low-priority lever.
- Ghost ring and stash-only variants: not carried forward.
