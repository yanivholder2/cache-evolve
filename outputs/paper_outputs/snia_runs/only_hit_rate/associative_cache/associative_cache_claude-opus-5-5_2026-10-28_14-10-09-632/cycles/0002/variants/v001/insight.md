# Variant v001 Insight

## Helped

- hit-rate: At size 16384 the 64-slot floor applies to a full 127-slot bucket, so a new item now goes in at index 63 instead of 31. That roughly doubles how many misses it can survive before reaching the eviction end. Hit-rate went up at 16384 on both `web_0` and `hm_0`, as the hypothesis predicted. More of the second references the parent's ~32-slot window missed now arrive while the block is still in the bucket. The gain was larger on `hm_0` than on `web_0`, which suggests `hm_0` has more reuse at distances between ~32 and ~64 bucket-misses.

## Hurt

- hit-rate: At 131072 both traces dropped very slightly. For a full 1023-slot bucket `insertion_index` returns 255, the same as the parent, so steady-state behavior did not change. The drift can only come from the fill phase. While a bucket holds roughly 128–255 items the floor puts new items deeper than `len/4`. That is a small transient, effectively noise, and it confirms the "keep v001's gain at 131072" half of the claim.

(`metadata-size` and `per-op-time` were not measured. A deeper insertion point in a 127-slot bucket shifts fewer trailing elements per `insert`, so the change should not cost speed. This run does not show that, though.)

## Takeaway

A probation floor set in absolute slots works where a fixed fraction did not. The parent's small-bucket shortfall was caused by too short a probation window, and lengthening only that window recovered hits at 16384 without changing large-bucket behavior. The gain at 16384 is modest even after doubling the window, though. Probation depth alone does not explain the gap between small and large buckets. Two levers come next: a floor above 64 (which nearly reaches the `len/2` cap at 127 slots), or protecting re-referenced items better, since the protected region above the insertion point shrank from ~96 to ~64 slots.
