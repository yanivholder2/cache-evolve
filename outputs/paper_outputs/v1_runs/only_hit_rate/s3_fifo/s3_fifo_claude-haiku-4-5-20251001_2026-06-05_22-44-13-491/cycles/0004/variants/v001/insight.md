# Variant v001 Insight

## Helped

- **hit-rate**: Reducing the small-queue admission threshold from 1/6 to 1/7 of capacity strengthens frequency selectivity even further, filtering out more items that lack strong reuse signals before they consume main-queue space, concentrating capacity on the highest-confidence repeaters.

## Hurt

(none)

## Takeaway

The trajectory toward tighter selectivity continues to yield gains. Narrowing the small queue from 1/6 to 1/7 demonstrates that the workload still sits comfortably below the utilization cliff—stricter admission gates remain beneficial without constraining the cache so severely that working-set diversity suffers. Each incremental tightening removes more transient items during the evaluation phase, leaving the main queue with denser, more reliable reuse patterns.
