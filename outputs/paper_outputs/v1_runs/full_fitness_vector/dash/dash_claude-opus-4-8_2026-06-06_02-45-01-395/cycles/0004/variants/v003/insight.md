# Variant v003 Insight

## Helped

- per-op-time: get/put p50 both dropped, but the change only *adds* work — an extra `probing2_index` lookup and at most one more bucket scan on hits — so it cannot causally speed `get` up; read this as run-to-run benchmark noise, not a real win.

## Hurt

- (none — no dimension regressed)

## Takeaway

Widening the neighborhood from 2-wide (`target`, `target+1`) to 3-wide by adding the non-wrapping secondary probing slot `target+2` left hit-rate **bit-for-bit identical** to the parent (0.6794 unchanged, not even noise-level movement). Raising set-associativity bought exactly nothing, which cleanly falsifies the hypothesis that conflict misses are a real component of the remaining miss mass on this trace — colliding warm keys were not actually thrashing, so giving them a third bucket to coexist in changed no eviction decisions. Associativity is a dead lever here; the remaining misses are compulsory/capacity, not conflict.
