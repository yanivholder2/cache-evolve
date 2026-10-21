# Variant v002 Insight

## Helped

- hit-rate: Barely moved. The first hit after insertion now sends a key to the bucket midpoint instead of the MRU tail, so a key that is hit only once is evicted after about half as many misses as under v001. It also stops pushing keys that were hit repeatedly toward eviction. The hit rate cleared the hypothesis threshold only in the last reported digit. That is too small a change to count as confirming H02, but it is not a clear refutation either.

## Hurt

(None measured. This evaluation recorded only hit-rate. The diff reuses the existing `lfu_counter` field as a 0/1 flag, so metadata-size should not change. A first hit now does a `Vec::remove` plus an `insert` at the midpoint instead of the policy's move-to-tail. That costs about the same in element shifts, so any per-op-time change should be small, but it was not measured.)

## Takeaway

On workload-a, two-step promotion adds almost nothing on top of v001's 1/8 insertion depth. The insertion depth already removes one-off keys before they are ever hit, so few keys are hit exactly once and then never again, and those are the only keys the midpoint step targets. A single hit predicts reuse about as well as two hits here. This implementation is also weaker than a true SLRU. `lfu_counter` is never reset after a key ages out of the MRU region, so a key promoted once goes straight back to MRU on every later hit. Value updates in `put` also count as hits. A follow-up that resets the flag when a key is demoted below the midpoint would test frequency-aware promotion properly before the idea is dropped.
