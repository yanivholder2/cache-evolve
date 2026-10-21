# Variant v001 Insight

Note: I could not list or diff the overlay source in this session (only Read and Write were usable). This insight rests on the hypothesis and the two fitness vectors, not on a line-level diff.

## Helped

- per-op-time: The change deleted write-only bookkeeping from the get and put paths, which removes a few stores per operation. The summary and the get p50 moved down by about one nanosecond, which is within plausible measurement noise.

## Hurt

- per-op-time: The put p99 tail got slightly worse than the parent's. Pure deletions should not lengthen a tail, so this is most likely noise.

## Takeaway

Deleting dead stores from an already-minimal DASH hot path left hit-rate and metadata-size identical, as intended. That confirms the removed fields were never read. Little write-only bookkeeping was left to remove, so the latency gain is at the noise floor. Further latency gains probably need structural changes to the probing or bucket operations, not more deletions.
