# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The change swapped `Option<usize>` link and head/tail fields for a `u32` sentinel with a conversion helper, and was meant to shrink node metadata without altering eviction. The only recorded fitness dimension, hit-rate, did not move from the parent, which is consistent with the conversion being behaviour-preserving on this trace. The intended size reduction is not in the fitness vector, so this variant teaches nothing measured about node size.
