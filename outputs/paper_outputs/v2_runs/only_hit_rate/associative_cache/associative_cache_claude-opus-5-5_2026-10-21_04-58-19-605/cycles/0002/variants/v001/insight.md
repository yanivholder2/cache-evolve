# Variant v001 Insight

## Helped

- hit-rate: New keys now enter a full 16-slot bucket at index 3 instead of index 7, so a key that is never reused is evicted after about 4 misses in its bucket instead of about 8. That frees slots for reused keys sooner. The grace window is still wide enough that keys with short re-reference distances get their second hit and are promoted to the MRU tail before they are evicted.

## Hurt

(None measured. This evaluation recorded only hit-rate, so per-op-time and metadata-size are unknown. The diff changes only `INSERT_DEPTH_DEN` from 2 to 4, so metadata cannot have changed. But each miss's `Vec::insert` now shifts about 12 elements instead of about 8, so a small per-op-time regression on the miss path is expected but not confirmed.)

## Takeaway

On workload-a, the midpoint was not the best insert depth. Pushing insertion deeper toward the eviction end still improved hit rate, which confirms H01's prediction. This means workload-a's one-off keys cost more than the reused keys that a shorter probation window might lose. The best depth is therefore at 1/4 or below, between here and pure LIP (index 0). The next step is to bracket it with depths like 1/8 or 0 and to measure per-op-time, since every step deeper costs more element shifts per miss.
