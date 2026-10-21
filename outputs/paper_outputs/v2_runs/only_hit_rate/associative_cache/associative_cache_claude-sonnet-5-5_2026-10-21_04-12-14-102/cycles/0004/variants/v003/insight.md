# Variant v003 Insight

## Helped

- hit-rate: Inserting new entries into a small probation segment (1/16 of each bucket's slots) and promoting only on a re-hit stops one-touch scan entries from displacing re-referenced ones. This lifted hit-rate clearly above the plain baseline on the LIRS workload-a trace.

## Hurt

- metadata-size: Not measured for this variant, since the fitness vector has only hit-rate. The probation/protected split needs per-bucket ordering and segment membership, so any cost here is unconfirmed.
- per-op-time: Also not measured. Hits now reorder entries between segments and may demote the LRU protected entry, which adds work to `get`. The effect cannot be judged from the evidence.

## Takeaway

A segmented-LRU split with a minimal fixed probation share, and no ghost, counter or adaptive state, recovers a large hit-rate gain on the baseline geometry. The gain does not depend on the accumulated cycle 1-3 changes. Because only hit-rate was recorded, the cost side of the trade-off is still unknown, and the variant can't be judged against the baseline on size or latency.
