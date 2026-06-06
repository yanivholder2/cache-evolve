# Variant v002 Insight

## Helped

(none)

## Hurt

- hit-rate: Raising the small-queue promotion threshold from freq > 1 to freq > 2 filters out items with only one reuse signal, but this eliminates candidates that would have contributed meaningfully to main-queue performance.

## Takeaway

Increasing selectivity at the promotion gate is not a free win. The stricter freq > 2 threshold prevents some items from graduating to the main queue, and these excluded items—despite having only one or two accesses—apparently provide enough utility that their loss outweighs the benefit of filtering noise. The original freq > 1 threshold appears better calibrated: it admits more items, but the signal-to-noise tradeoff favors the broader admission policy on this workload.
