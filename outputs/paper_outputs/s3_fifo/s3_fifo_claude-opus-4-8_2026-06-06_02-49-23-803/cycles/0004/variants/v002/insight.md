# Variant v002 Insight

## Helped

- hit-rate: raising `MAX_FREQ` from 3 to 7 lets a proven hot key absorb more `evict_main` second-chance decrements before it can be dropped, so on top of v001's `capacity/32` split the durable keys survive slightly longer and reaccesses land marginally more often.

## Hurt

- (none) metadata-size is unchanged because `freq` is already a `u8` — a ceiling of 7 still fits the same byte, so neither `Entry` nor `S3Fifo` layout moved. per-op-time is flat within noise: the saturating-add-and-clamp in `get` and the decrement in `evict_main` cost the same regardless of the constant's value.

## Takeaway

Combining the two confirmed-orthogonal levers (`capacity/32` split + `MAX_FREQ=7`) does compound, but the gain is the smallest hit-rate move yet — both axes were already deep in diminishing returns, so stacking them is additive only at the margin rather than unlocking a new regime. The last untested split×freq-ceiling cell confirms the grid is essentially saturated for workload-a at capacity 2048.
