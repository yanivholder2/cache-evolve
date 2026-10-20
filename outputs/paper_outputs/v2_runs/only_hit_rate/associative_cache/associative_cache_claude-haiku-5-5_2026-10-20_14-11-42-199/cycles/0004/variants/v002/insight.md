# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Scanning only 8 rotating probationary slots per eviction can pick a victim that is not the lowest-ranked probationary entry, so a slightly hotter line may be displaced; the measured effect on workload-a is a negligible dip that misses the hypothesis's "at or above incumbent" guardrail.

## Takeaway

Bounding the probationary victim scan to a rotating window of 8 slots left hit-rate essentially unchanged from v003, so exact-minimum victim selection buys little on workload-a at this size. The hypothesis's real target, per-op time, is not in the fitness vector, and eval/ shows no evidence the bounded path was active, so this variant neither confirms the speedup nor rules it out.
