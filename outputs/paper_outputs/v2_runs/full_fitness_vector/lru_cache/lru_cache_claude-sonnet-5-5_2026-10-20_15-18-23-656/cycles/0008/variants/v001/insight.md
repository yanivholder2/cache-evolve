# Variant v001 Insight

## Helped

- per-op-time: put tail latency dropped, consistent with the halved ghost table (`ghost_len` = next_pow2(capacity)/2) touching fewer cache lines on the lookup/clear/write-on-eviction path; put p50 was essentially unchanged.
- hit-rate: it did not fall as the hypothesis feared and ticked up slightly, so the extra ghost collisions did not cost admission quality. The gain is within noise, so treat it as "no loss".

## Hurt

- per-op-time: get latency moved up by about a nanosecond. `get` never touches the ghost table, so this is measurement noise and not an effect of the change.

## Takeaway

The ghost table in v002 was oversized. Halving it kept the fingerprint-based hit-rate gain and trimmed put tail latency. Fingerprint collisions at roughly capacity/2 slots are tolerable for this workload. metadata-size did not move because it measures only the static struct sizes, and the ghost table lives on the heap, so the claimed side-memory saving is invisible to this fitness dimension.
