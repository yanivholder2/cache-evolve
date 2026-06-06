# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Making `put()` symmetric with `get()` for probationary promotion is logically sound — the full `was_protected || sketch.estimate > PROMO_THRESHOLD` gate with overflow demotion now mirrors `get()` exactly — but it moved the needle on `hit-rate` by exactly zero. The workload-a trace is read-dominant enough that hot items accumulate the promotable frequency count via `get()` calls long before any `put()` update touches them; the promotion gap this change was designed to close (an item updated repeatedly without a subsequent `get()`) simply does not appear in this trace at any meaningful rate. Future gains from promotion-path changes will require either a write-heavy trace or a different axis of attack altogether.
