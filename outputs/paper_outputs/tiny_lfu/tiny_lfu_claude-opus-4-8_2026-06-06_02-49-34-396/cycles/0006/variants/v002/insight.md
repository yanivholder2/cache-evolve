# Variant v002 Insight

## Helped

- hit-rate: Conservative update in `increment` — scanning for the per-key minimum and raising only the rows that equal it — cuts the systematic over-count that collisions inject into the narrow factor-4 sketch, tightening the frequency estimate that drives both the admission test and victim choice, so eviction keeps genuinely hotter keys.

## Hurt

- per-op-time: The added per-key min-scan plus the conditional `== min` write in `increment` runs on every access (both get and put record frequency), so it lifts get p50/p99 and put p50/p99 across the board — a larger, hot-path cost than the hypothesis's "marginal" prediction, since the scan now precedes the write instead of being amortized only inside `estimate`.

## Takeaway

Conservative update is a real signal-sharpening win on the narrowest sketch: it raises hit-rate without touching geometry or metadata size. But it is not free — moving the min-scan onto the write path taxes every operation, not just eviction, so the estimate-quality gain comes at a broad per-op latency cost rather than the predicted marginal one.
