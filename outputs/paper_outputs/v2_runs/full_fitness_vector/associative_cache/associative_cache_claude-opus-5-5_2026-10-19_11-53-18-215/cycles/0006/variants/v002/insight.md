# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Making protected hits lazy changed no dimension. Hit rate and metadata-size are identical to the parent's, get latency is identical at both p50 and p99, and put moved by about 1 ns in opposite directions at p50 and p99, which is noise. The likely reason is that the change does almost nothing at these bucket sizes. `hot_zone = (prot_len / 4).max(1)` is just the MRU slot whenever protected holds fewer than 8 items, and hitting the MRU slot was already free in the parent, because `rotate_left(1)` on a one-element slice moves nothing. Even with a larger protected segment, the skipped rotate would have moved only one or two 16-byte items, which is negligible next to the linear `find` scan that every get pays. So the earlier finding that "memmove dominates" does not apply to hits that are already near MRU. A latency gain from lazy promotion would have to come from hits deep in protected or in probation, and skipping those rotates is exactly what would start to cost hit rate.
