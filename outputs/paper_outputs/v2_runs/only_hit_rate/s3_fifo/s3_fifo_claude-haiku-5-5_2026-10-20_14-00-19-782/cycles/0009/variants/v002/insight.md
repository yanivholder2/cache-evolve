# Variant v002 Insight

## Helped

- hit-rate: Adding a bounded ghost push in `evict_main` (keys that leave main at zero frequency now enter the ghost queue) lets a key that was recently main-evicted take the ghost-hit path on its next access and re-enter `main` directly, instead of restarting in `small`. This was the only dimension measured, and it moved in the predicted direction.

## Hurt

## Takeaway

Feeding main-queue evictions into the ghost queue is a small, positive change on lirs workload-a at capacity 2048, and the mechanism matches the hypothesis: re-references to main-evicted keys now skip the small-queue admission path. The gain is marginal and comes from one trace and one size, so it is weak evidence on its own. Note too that the `ghost` `retain` and `iter().any` scans on the miss path are unchanged, so this variant adds ghost entries without changing the cost of ghost lookups.
