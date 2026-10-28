# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384): The hypothesis's target cell recovered and ended above the level it had before cycle 0007's halving change. Main is only 64 slots in 128-slot buckets. Going back to linear decrement there, and ranking by `recent + counter`, gives 9 survival tiers in place of the 5 that bit-length allowed. Counters 2 vs 3 and 4–7 are separated again, so hm_0's warm-but-not-hottest blocks outlive its moderately-hit ones. The ranking-resolution diagnosis in cycle 0007's insight is confirmed.
- hit-rate (web_0 @ 16384): This cell moved up slightly instead of falling below the refutation bar. Under linear decay, a stale block at the ceiling now takes up to 7 visits to drain, not 3. That did not keep stale web_0 blocks too long, because the short 64-slot sweep wraps often enough that the slower per-visit decay is still fast in wall-clock terms. The recent bit also still protects blocks that are actually live, so the extra tiers mostly sharpen the ordering.

## Hurt

(none)

## Takeaway

The size gate on the decay rule worked as designed. Both 131072 cells are bit-identical to the parent, so the 768-slot main keeps recency-gated halving untouched, and only the 64-slot main changed. Halving and linear decrement turn out to suit different scales. In large main regions, faster draining of once-hot blocks is what matters. In small main regions, the sweep already drains them fast enough, so finer frequency resolution is what matters, and combining the recent bit with linear counters gives the finest ranking yet. Later decay or tiering changes should be judged per main-region size, not once for all sizes.
