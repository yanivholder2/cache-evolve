# Variant v001 Insight

## Helped

- per-op-time: trimming the ghost cap from `(main_size / 2)` to `(main_size * 3 / 8)` shortens the linear `ghost.iter().any(...)` membership scan on the `put` path, cutting both put p50 and put p99 below the parent's 50%-ghost cost (get is untouched — it never scans the ghost).

## Hurt

- hit-rate: shrinking the ghost window to ~37.5% of main dropped hit-rate fractionally below the parent's 50% window, meaning the shorter window evicted some near-reuse keys that the 50% cap still recovered.

## Takeaway

Pushing the ghost cap to 3/8 of main bought a real put-latency win but cost a sliver of hit-rate, disproving the hypothesis that the hit-rate peak sits below 50%. On workload-a the reuse-distance knee is at the parent's 50% cap, not under it: the 25–50% band trades recoverable near-reuse hits for a shorter membership scan, so 3/8 is a latency-vs-quality move, not a strict improvement.
