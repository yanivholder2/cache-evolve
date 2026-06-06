# Variant v002 Insight

## Helped

- hit-rate: essentially unchanged (a hair up, within noise) — relocating a saturated entry to the front of `main` while leaving eviction-from-the-back and the small/ghost paths untouched preserves the protected key set exactly as the hypothesis predicted.

## Hurt

- per-op-time: get p99 rose because the `MAX_FREQ-1 → MAX_FREQ` transition now fires an O(position) `main.remove(idx)` + `push_front` on the hottest keys, so the gets that trip the saturation boundary pay a real walk-and-shift cost in the tail.

## Takeaway

Front-loading proven-hot keys at the freq-saturation transition did **not** move get-p50, its stated target — the median get already lands shallow in `main`, so physically clustering Zipfian-hot keys at the head gave no median scan savings while the rare O(position) relocation only widened the get tail. The get-p50 floor here is not caused by hot-key scatter within `main`, so in-place reordering can't lower it.
