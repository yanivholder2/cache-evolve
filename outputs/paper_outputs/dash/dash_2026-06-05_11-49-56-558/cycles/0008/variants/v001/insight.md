# Variant v001 Insight

## Helped

- hit-rate: Expanding the normal tier from 24 to 28 buckets (7/8 split) gave more hot-tier slots for the workload's active set, producing a modest but positive lift over the parent's 3/4 split.

## Hurt

## Takeaway

Shifting from a 3/4 to a 7/8 normal-bucket split continued the upward trend in hit-rate, but the gain is notably smaller than the earlier move from 16/16 to 24/8 — indicating diminishing returns as the normal tier grows. The second-chance recovery inherited from the parent absorbs the reduced stash depth (8 → 4 slots) without cost, confirming that stash overflow is no longer a significant loss site at this workload size; the bottleneck has shifted to the absolute count of hot-tier slots rather than stash headroom.
