# Variant v003 Insight

## Helped

- hit-rate: Gating target-to-probing promotion on `lfu_counter >= 2` prevented items with only a single in-target access from occupying the probing tier, keeping its slots available for items that had demonstrated genuine recurrence rather than a lucky one-time hotness coincidence.

## Hurt

## Takeaway

Adding the `lfu_counter >= 2` guard to the `is_hottest` promotion condition produced a real but marginal lift: single-access items do occasionally rise to the top of their target bucket and displace better probing candidates, but the effect is small on this workload, suggesting the hot set is mostly confirmed across multiple accesses before hitting the promotion gate anyway. The intra-bucket fallback path — which increments the counter and moves the item to MRU — correctly accumulates the evidence without evicting the item, so the gate adds filtering without creating a residency gap.
