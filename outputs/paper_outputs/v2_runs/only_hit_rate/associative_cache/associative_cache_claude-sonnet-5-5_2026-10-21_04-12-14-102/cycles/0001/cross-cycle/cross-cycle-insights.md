## Recurring wins

- Geometry change (wider buckets) is the only approach that has won so far, and only by a small hit-rate margin. Hash imbalance across sets is a minor miss source.
- Not yet confirmed across multiple cycles (one cycle of history). The win is unproven until per-op-time is measured.

## Recurring failures

- Weakening hit-path promotion in short buckets (one-step promotion, 2Q-style half-promotion) lowered hit rate. Full move-to-back recency is hard to beat at small associativity on the LIRS trace.
- Evaluation gap: per-op-time and metadata-size were missing from every fitness vector. Speed and size claims were therefore untestable and cannot support a win.

## Open questions

- Can per-op-time and metadata-size be reliably measured, so speed-targeting hypotheses can be judged?
- Does the wider-bucket hit-rate gain survive once get/remove/push cost of long scans and shifts is counted?
- Does scan resistance work through a separate probationary segment or admission control, rather than shallower promotion within one bucket?
- Does frequency-aware victim choice help, given that most misses look like capacity or policy misses?

## Per-policy notes

- Plain move-to-back LRU: strong baseline. Promotion-weakening variants (transposition, midpoint, 2Q-style half-promotion) did not beat it.
- Wider-bucket LRU: slight hit-rate gain, cost unverified.
- Untried: probationary-segment or admission-based policies, and frequency-aware eviction.
