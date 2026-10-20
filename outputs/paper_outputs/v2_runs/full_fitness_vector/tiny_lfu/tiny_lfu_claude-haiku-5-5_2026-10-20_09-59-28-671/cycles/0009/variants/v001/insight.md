# Variant v001 Insight

## Helped

- per-op-time: Tail latency for both get and put dropped relative to the parent, consistent with the hypothesis removing the separate read-all-rows pass before the conditional write, which was the extra per-increment work the parent added.

## Hurt

- per-op-time: The put median rose instead of falling, so the reuse of admission-check counter values did not offset the cost of the restructured single loop on the common put path.

## Takeaway

Collapsing the parent's two-pass conservative update into one loop helped the tail of both operations, but it did not deliver the median put win the hypothesis predicted, and hit-rate stayed flat, which is the expected result since admission decisions were meant to be preserved. The source diff could not be inspected in this review (no shell, file listing, or search access was available), so the cause of the put-median regression is unconfirmed, and the per-op-time reading rests on a single run at nanosecond scale and should be treated as provisional.
