# Variant v003 Insight

## Helped

- hit-rate: Frequency-gating target→probing promotion prevents occasionally-accessed items from claiming space in the expensive probing tier, reserving it for genuinely hot keys.
- per-op-time (get): Fewer items in the probing bucket reduces lookup contention and cascading demotions, making reads faster.
- per-op-time (put): Selective promotion reduces overall cache churn and eviction-cascade overhead, lowering operation latency.

## Hurt

## Takeaway

Requiring items to prove sustained access (counter ≥ 2) before entering the probing tier effectively filters out noisy, transient hotness. This simple gating mechanism substantially reduced operation latency by eliminating false-positive promotions that would trigger expensive cascading demotions, while simultaneously improving hit-rate by keeping probing capacity focused on items with persistent access patterns.
