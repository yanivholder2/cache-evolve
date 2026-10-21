# Variant v002 Insight

## Helped

- hit-rate: Tie-admitting repeat keys (estimate >= 2) in the window-to-probation duel, plus using the protected LRU as victim when probation is empty, gave a very small gain over the parent. Equal-frequency repeat keys now displace stale probation entries instead of being dropped. The gain is near the noise level for a single trace and size.

## Hurt

- No fitness dimension regressed. This vector reports only hit-rate, so any per-op or metadata cost of the extra victim lookup is unmeasured.

## Takeaway

Loosening the admission duel to `>=` for repeat keys, while keeping strict `>` for one-hit keys, is safe: it preserves scan resistance and nudges hit-rate up. The gain is tiny, so the duel's tie-break and empty-probation fallback are not a significant bottleneck on workload-a at size 2048. Larger gains would have to come from elsewhere, such as segment sizing or the sketch.
