# Variant v002 Insight

## Helped

- hit-rate: Doubling `SKETCH_WIDTH_FACTOR` from 8 to 16 further reduced bucket collision rate in the count-min sketch, allowing a small additional accuracy improvement in frequency estimates — though the gain was roughly one-quarter the size of the prior 4×→8× step, suggesting collision noise was already largely suppressed at 8×.

## Hurt

- per-op-time: At 16× width the sketch heap at cap=1024 reached 262 144 bytes (256 KB), exceeding typical L2 cache boundaries and causing both p50 and p99 to double for get and put — the p50 regression (100 ns → 200 ns) signals the slowdown is now systematic rather than a tail-only event.
- metadata-size: The sketch heap doubled from 131 072 to 262 144 bytes at cap=1024, a direct consequence of the width factor change, making this variant materially more memory-hungry without a proportionate accuracy payoff.

## Takeaway

Widening the count-min sketch from 8× to 16× capacity delivers a diminishing hit-rate return compared to the previous doubling step while simultaneously pushing the sketch past L2 cache boundaries, converting what was a sporadic tail-latency penalty into a systematic per-operation slowdown. This indicates that 8× already reduced collision noise to near-saturation on this Zipfian trace, and that the L2-to-L3 boundary is a hard latency cliff for sketch-width scaling — future width increases are unlikely to be worthwhile unless the capacity or trace characteristics change substantially.
