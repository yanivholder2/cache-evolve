# Variant v003 Insight

## Helped

## Hurt

- per-op-time (get): Moving the access counters from the item into a cache-owned parallel array means each get hit now touches a second memory location to read and write its counter, instead of one co-located with the key and value. That extra indirection is the likely source of the get tail regression, and it undoes the tail gain v002 showed.
- per-op-time (put): The put path now initialises and scans counters in the separate array. The shift is small and in the same direction as the get regression, so it is plausibly caused by the layout change, but it is within the run-to-run spread that the parent insight flagged.

## Takeaway

This variant tested whether the get-tail gain from v002 came from the struct layout or from the skipped counter store itself. Moving the counters out of the item without changing the retention policy removed the gain, which supports the layout-refutation branch of H03 and points to the store-skip as the cause. Hit-rate and item and cache sizes were unchanged, as expected, since the item size did not shrink when the counter left it. The fitness vector gives one measurement per dimension, so the hypothesis's requirement of three runs against v002 measured in the same session is not confirmed here and should be checked before the result is treated as settled.
