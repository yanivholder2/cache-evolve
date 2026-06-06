# Variant v002 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Super-linear age weighting (age^1.5) did not improve hit-rate compared to the linear age term in v001. The workload's temporal clustering is already adequately captured by the linear age component in the frequency-recency hybrid score; exponentially penalizing older items does not provide additional discriminative power for victim selection on this trace.
