# Variant v002 Insight

## Helped

- hit-rate: Full 8-bit frequency tracking preserves finer discrimination between moderately-hot items compared to 3-bit bucketing, enabling more precise victim selection when multiple items share the minimum frequency level.

## Hurt

(None observed)

## Takeaway

Removing quantization in favor of full 8-bit frequency precision demonstrates that finer gradient information is valuable for eviction decisions in frequency-based caching. The multiplicative decay strategy (0.75× per cycle) successfully maintains discriminative power across the expanded frequency range, allowing the cache to separate items with 4-7 accesses from those with 8-15 accesses and make better victim selection decisions.
