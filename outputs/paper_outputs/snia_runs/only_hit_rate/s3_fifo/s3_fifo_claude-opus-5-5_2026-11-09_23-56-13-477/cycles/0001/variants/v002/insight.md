# Variant v002 Insight

## Helped

## Hurt

## Takeaway

Moving to a slab plus `HashMap<K, Loc>` index, with lazy-deletion stamps in the ghost queue, left hit-rate identical to the baseline on every trace and size. That confirms the claim that the refactor keeps the policy intact: second-chance in main, graduation at `freq > 1`, a ghost bound of `main_size` live entries, and re-admission into main on a ghost hit. The goal of the change was per-op-time, but this variant's fitness vector has only the hit-rate dimension. Neither per-op-time nor metadata-size was measured, so the speedup is still unconfirmed. The next evaluation should re-run per-op-time, and also metadata-size: `Entry` is still `{key, value, freq}`, but `S3Fifo` itself now holds the index, slab, free list and stamp counters.
