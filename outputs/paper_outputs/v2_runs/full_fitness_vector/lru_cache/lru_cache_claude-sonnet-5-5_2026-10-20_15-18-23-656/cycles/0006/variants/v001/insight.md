# Variant v001 Insight

## Helped

- hit-rate: Growing the protected segment from 3/4 to 7/8 of capacity leaves the probationary segment smaller, so re-referenced entries survive scan bursts better and hit-rate rose a little over the parent.

## Hurt

- per-op-time: The change was one constant in `try_new` and touched no hot path, so the slower get/put percentiles are most likely measurement noise and not a real cost. The per-op-time dimension still regressed against the parent.

## Takeaway

The 1/2 → 3/4 → 7/8 trend in protected share keeps improving hit-rate, but the gain from this step is small, which fits the earlier note that the split is a weak lever. Metadata-size did not move, as expected for a constant-only edit. Per-op-time deltas on a no-hot-path change show the benchmark noise floor, so they should not be read as a cost of the split. The next probe should look for the turning point or a different lever.
