# Variant v001 Insight

## Helped

- hit-rate: Switching `promotion_get` to `InsertPosition::Back` causes stash-promoted items to land at the MRU end of their target bucket, so a reactivated cold item qualifies for probing-tier promotion after just one more access rather than needing to bubble through the entire bucket — this collapsed re-entry penalty translates into faster occupation of the expanded normal-bucket pool inherited from v003's 50/50 geometry.

## Hurt

## Takeaway

Stacking MRU re-entry onto v003's 50/50 normal/stash split produced a further hit-rate improvement over v003 alone, confirming that hot-tier capacity (geometry) and re-entry speed (insertion position) are distinct, non-interfering axes. The gain above v003 is real but modest, which suggests that once the hot tier is enlarged the marginal value of faster re-entry shrinks — items already have more normal-bucket dwell time, so collapsing the re-entry penalty from O(bucket-size) to one access matters less than it does against a smaller hot tier.
