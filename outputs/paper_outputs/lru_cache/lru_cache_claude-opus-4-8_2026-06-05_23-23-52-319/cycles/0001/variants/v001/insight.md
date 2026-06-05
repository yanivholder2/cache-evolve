# Variant v001 Insight

## Helped

- hit-rate: Splitting the single list into probationary/protected segments and promoting a key on its second access shields reused keys from eviction, since `put`-after-miss always evicts the probationary LRU first — exactly the reuse-resistance the LIRS trace rewards.

## Hurt

## Takeaway

On this reuse-heavy LIRS trace, gating eviction on a second access (SLRU) beats pure recency: promoting hit keys to a protected tier and draining one-hit-wonders from probationary first is what lifts hit-rate. The 80/20 protected split sized that protected tier without changing the P1 capacity bound (segments sum to `capacity`), so the gain came purely from eviction ordering, not extra residency.
