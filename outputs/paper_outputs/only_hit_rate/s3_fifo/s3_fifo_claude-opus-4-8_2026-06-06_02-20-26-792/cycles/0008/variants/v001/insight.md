# Variant v001 Insight

## Helped

- hit-rate: Making the split adaptive — nudging `small_size` ±1 on every ghost re-admission according to the matched record's `from_main` tag, then recomputing `main_size = capacity - small_size` — lets the partition drift toward whichever side is currently being re-referenced from its ghost history, recovering hits the frozen `capacity/24` ratio left on the table on this trace.

## Hurt

## Takeaway

Adaptivity is the first mechanism that beats static sizing rather than collapsing back into it: reusing the already-computed `from_main` steering signal to shift future eviction thresholds (no entries moved, just the bound) turns the ghost queue into a feedback controller. The gain is small because the `[capacity/48 .. capacity/12]` clamp keeps the orbit tight around the proven center and workload-a's balance only wanders modestly — but it moved the right direction, confirming the partition was slightly main-starved and the split should be allowed to track the workload instead of being pinned.
