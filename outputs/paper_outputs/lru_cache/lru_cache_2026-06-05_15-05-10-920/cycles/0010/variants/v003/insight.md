# Variant v003 Insight

## Helped

- hit-rate: Threshold-based amnesia creates a sharp boundary that cleanly separates frequently reused items (freq ≥ 2) from one-time accesses (freq < 2), enabling the cache to better protect the working set and more aggressively evict ephemeral items.

## Hurt

## Takeaway

Threshold-based frequency amnesia outperforms smooth multiplicative decay by using discontinuous forgetting rather than gradual proportional decay. The sharp cutoff at freq=2 allows working-set items that demonstrate sustained reuse to retain their frequency signals entirely while aggressively discarding the frequency history of transient accesses. This creates a clearer decision boundary for victim selection, resulting in significantly higher hit-rate. The gap to the 0.7334 target suggests the threshold value itself could still be tuned for further gain.
