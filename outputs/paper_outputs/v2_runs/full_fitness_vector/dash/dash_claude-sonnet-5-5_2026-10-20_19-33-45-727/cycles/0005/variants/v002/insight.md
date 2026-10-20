# Variant v002 Insight

## Helped

- hit-rate: Unchanged against the parent, so skipping promotion on a flagged hit did not change which items get evicted on this trace. The "dropping promotion is cheap in hit-rate" premise held.
- metadata-size: Unchanged. The edit only touched the get-hit branch, and the flag bit stays packed in the existing item layout.

## Hurt

- per-op-time: The get p50 and the put p50 and p99 all got slower than the parent. Only the get p99 improved. The hypothesis expected a lower per-op-time from removing the promotion work, but the median cost rose instead. A plausible cause is that the promotion move was not the dominant cost of a hit. Another is that the in-place flag write changed memory access or branch behavior. Run-to-run noise at this nanosecond scale could also account for it. I could not inspect the overlay diff, so this is inferred from the fitness vectors only.

## Takeaway

Removing the promote-on-second-hit move was neutral for hit-rate and metadata size, but it did not deliver the expected speedup. It made the median get and put latency slightly worse. On this workload, the second-hit promotion is therefore not where hit-path time goes. Later per-op-time work should look at other costs, and should repeat measurements to separate real effects from noise.
