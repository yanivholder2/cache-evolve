# Variant v003 Insight

## Helped

- hit-rate: Replacing the flat LRU with ARC — two resident lists (T1 seen-once, T2 frequent) plus B1/B2 ghost lists driving a self-tuning target `p` — let the cache protect repeatedly-reused keys from one-shot recency scans, lifting hit-rate on workload-a over the single-list baseline.

## Hurt

(no other fitness dimension moved)

## Takeaway

On this trace, the win came from frequency-awareness plus ghost-driven adaptation: promoting a second reference into T2 and letting B1/B2 hits nudge `p` toward the workload's actual recency/frequency mix beats the baseline's single global LRU ordering. The gain is paid for with a second HashMap, a ghost arena, and four-list bookkeeping that the flat baseline never carried.
