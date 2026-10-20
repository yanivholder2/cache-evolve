# Variant v001 Insight

## Helped

## Hurt

- per-op-time (put): The hypothesis predicted that stopping the `evict_one()` victim scan at the first zero-counter slot would make puts cheaper, but put latency moved the wrong way. Per the hypothesis the early exit only changes how far the scan runs in a full bucket, so the added per-slot zero check cost more than the shorter scan saved on this workload.
- per-op-time (get): The change was confined to the eviction scan, so get should not have moved at all. It did get slower, which the change as described does not explain. This is either noise or a code-layout effect outside the eviction path, and it should be checked before treating the get regression as a property of this change.

## Takeaway

An exact early exit on the lowest-counter scan preserved victim choice, since hit rate and metadata size matched the parent exactly, but it did not make puts cheaper. Cutting the scan short by checking for a zero counter is therefore not a useful speedup on this workload, and the put cost is not explained by scan length alone. The result is measured on workload-a only.
