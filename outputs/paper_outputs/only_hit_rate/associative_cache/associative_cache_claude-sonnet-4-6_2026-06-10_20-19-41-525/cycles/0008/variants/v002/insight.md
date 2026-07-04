# Variant v002 Insight

## Helped

- hit-rate: Partitioning each bucket into probationary and protected segments prevented scan traffic from displacing confirmed-hot items — items accessed more than once graduate to the protected segment and can only be evicted when the protected segment is itself full, keeping the working set stable under workload-a's access pattern.

## Hurt

## Takeaway

The SLRU two-queue design confirms that explicitly separating "seen once" items from "seen more than once" items is the right structural move for workload-a: the probationary segment absorbs scans and one-hit wonders without polluting the protected segment, which holds the genuine working set. This result opens SLRU as a viable evolutionary branch and closes out the hypothesis that co-encoding position + counter in a single queue is the only workable model for this workload.
