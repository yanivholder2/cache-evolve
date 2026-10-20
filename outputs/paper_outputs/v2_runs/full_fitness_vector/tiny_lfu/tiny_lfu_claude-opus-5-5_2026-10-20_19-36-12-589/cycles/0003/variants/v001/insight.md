# Variant v001 Insight

## Helped

- per-op-time: get and put both got faster at p50 and p99, but the diff can't explain it. `get` is byte-for-byte the same as v002. The only change to `put` is `>` → `>=`, which makes more tie contests take the admit path, and that path is the heavier one (key clone, `map.remove`, unlink/push_back, `map.insert`) compared with an early reject. Its being faster on every op and percentile at once points to the benchmark environment again. v002's insight already suspected its own regression was environmental, and that fits this. It should not count as a speed win for tie-admission.

## Hurt

- hit-rate: Letting newcomers win frequency ties slightly lowered hit-rate on workload-a. The incoming key is counted before the comparison, and the sketch never ages. So a one-shot arrival with an estimate of 1, or one inflated by count-min collisions, now pushes out an LRU-head victim of equal estimate. Under strict `>`, that victim was kept and could still be hit again. The tie-break turns cold churn into evictions of residents.

## Takeaway

Changing the admission operator from `>` to `>=` is a clean single-variable test. It moved hit-rate slightly in the wrong direction. Ties at the LRU head reward the incumbent on workload-a, not recency, so the strict rule was not over-rejecting. By the hypothesis's own test, the hit-rate ceiling lies elsewhere: sketch aging or saturation, collision noise, or victim selection, not the tie-break. This variant also shows that the per-op-time numbers aren't stable across runs. v002 and v001 should be benchmarked side by side before any speed differences between them are trusted.
