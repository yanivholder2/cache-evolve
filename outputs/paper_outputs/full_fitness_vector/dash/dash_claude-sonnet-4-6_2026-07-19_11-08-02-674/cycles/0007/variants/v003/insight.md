# Variant v003 Insight

## Helped

- hit-rate: Incrementing `lfu_counter` on every non-hottest target hit and promoting to probing when the counter reaches 2 with a vacancy allows frequently-accessed items to bypass the hottest-only gate, placing them in the probing tier where they are shielded from target eviction.

## Hurt

- per-op-time: Every non-hottest target hit now executes a counter write, a counter read-back, an `is_full()` check on the probing bucket, and a conditional branch — even when the promotion threshold is not reached — tripling median latency compared to the parent's unconditional `get_from_position` rotation.

## Takeaway

Vacancy-gated frequency promotion successfully raises hit rate, but it does so by adding overhead to the common non-hottest-target path that runs on every such access regardless of whether a promotion fires. The per-access cost of the counter increment plus vacancy check dominates on this workload, showing that gating promotion on a frequency counter is more expensive per operation than gating it solely on position (the hottest-only gate), even when the gate itself is rarely triggered.
