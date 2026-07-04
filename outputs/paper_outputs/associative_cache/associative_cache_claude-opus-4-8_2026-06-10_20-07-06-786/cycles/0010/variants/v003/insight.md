# Variant v003 Insight

## Helped

- hit-rate: Admitting fresh keys at the eviction front (`insert(0)`) instead of pushing to the protected back makes each newcomer the immediate next victim, so the LIRS trace's scan/loop bursts churn through one probationary slot while older keys in `[1, len)` survive to earn their promoting second hit.

## Hurt

- per-op-time: Not re-measured here, but `insert(0)` shifts the whole bucket vector on every miss (O(bucket_size)) where baseline's `push` was amortized O(1) — a latent cost the front-admission change introduces on the put path.

## Takeaway

Scan-resistant MRU probation works on this workload: discarding the most-recent unproven key first protects keys one hit away from promotion and is what closed the gap to the second-hit ceiling. The win came from front *admission* (one churning slot) rather than the hypothesis's literal eviction-index rewrite, but both realize the same "evict newest unpromoted" discipline.
