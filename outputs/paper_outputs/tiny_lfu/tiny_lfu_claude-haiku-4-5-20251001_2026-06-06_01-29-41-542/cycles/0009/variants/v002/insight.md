# Variant v002 Insight

## Helped

(none)

## Hurt

- hit-rate: Age-based partitioning (targeting the oldest 25% of items) is less effective than frequency-based partitioning at identifying victims, suggesting that temporal staleness alone is a weaker discriminator than access frequency for this workload.

## Takeaway

Swapping the primary partitioning criterion from frequency (v001) to age (v002) produces a small regression in hit-rate. This teaches that although age and frequency are conceptually independent signals, frequency is the stronger primary filter for victim selection on this workload. Age-first hierarchical partitioning does not compound the two signals more effectively than frequency-first filtering, contrary to the hypothesis' expectation.
