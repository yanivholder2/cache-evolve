# Variant v003 Insight

## Helped

- hit-rate: Raising the small-queue share of capacity to 20% lets the small FIFO hold more one-hit wonders before they can reach the main queue, so more cold keys are evicted without displacing reused ones on workload-a at capacity 2048.

## Hurt

## Takeaway

The change is a pure partition-constant move: the small-queue fraction goes up and the main queue and ghost queue follow, with no change to promotion, eviction, or metadata layout. The hit-rate gain is small and consistent with that mechanism, and metadata-size is unchanged as the hypothesis required. The get and put latency drops are much larger than a split constant should cause, since the get path is untouched, so they most likely reflect measurement noise or environment drift and should not be credited to this change. I could not read the overlay source in this session, so I have not confirmed that the diff matches the hypothesis.
