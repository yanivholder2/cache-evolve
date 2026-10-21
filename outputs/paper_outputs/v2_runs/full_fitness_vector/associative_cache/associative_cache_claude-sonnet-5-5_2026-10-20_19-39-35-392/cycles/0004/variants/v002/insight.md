# Variant v002 Insight

## Helped

- per-op-time: Replacing the half-bucket shift on mid-insertion with an overwrite-victim-then-swap-with-mid step makes put O(1) instead of a memmove/rotate, lowering put latency at both median and tail.
- per-op-time (get): Get latency improved marginally even though the get path was untouched, most likely from measurement noise or layout/codegen effects, not a real mechanism.

## Hurt

- (none) hit-rate and metadata-size did not move: the approximate eviction order among displaced items had no measurable effect on this trace, and no fields were added.

## Takeaway

Mid-insertion's benefit comes from where the new entry lands, not from the exact order of the entries it displaces. Swapping into the mid slot kept the parent's hit rate while removing the shift cost from put. Exact ordering within the bucket is not worth paying for on this workload.
