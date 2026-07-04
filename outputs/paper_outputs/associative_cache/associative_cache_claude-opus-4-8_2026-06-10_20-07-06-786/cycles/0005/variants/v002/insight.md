# Variant v002 Insight

## Helped

- hit-rate: No gain — the ghost-ring re-entry did not raise hit-rate on the returning keys it was designed to recover; the measured dimension moved the wrong way.

## Hurt

- hit-rate: Admitting any key found in `ghost_keys` straight to the protected MRU back with `lfu_counter == 1` is too coarse a "proven" signal — with the ring bounded to `bucket_size`, a key that merely bounced out and back within one lap (not a genuinely hot key) skips probation and seizes protected space, pushing eviction to fall back onto the protected LRU end and discard truly-proven keys, so hit-rate fell below the parent.

## Takeaway

On this collision-bound LIRS trace, recent eviction is not a reliable proxy for "hot": treating every ghost-remembered return as already-proven floods the protected segment with weakly-warranted keys and starves it of capacity for keys that earned promotion through a real second hit. The parent's earn-it-each-time FIFO probation filters churn better than accelerated re-entry does — the discriminating power of probation, not the latency of re-admission, is what carries hit-rate here.
