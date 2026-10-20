# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Lowering the `evict_small` promotion threshold from `freq > 1` to `freq > 0` admitted entries reused only once into the main queue, where they displaced hotter resident items and increased main-queue churn, slightly reducing hit-rate on this LIRS workload.

## Takeaway

On this trace the baseline's `freq > 1` gate is doing useful filtering: requiring two reaccesses before graduation keeps warm-but-not-hot small-queue entries out of the main queue. Promote-on-any-reuse (the paper's stated behavior) does not beat the stricter threshold here — the extra admissions cost more in evicted hot keys than they save in re-admission round-trips.
