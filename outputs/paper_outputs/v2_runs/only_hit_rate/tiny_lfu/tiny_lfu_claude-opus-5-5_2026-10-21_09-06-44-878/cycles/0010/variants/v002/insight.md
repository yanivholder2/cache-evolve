# Variant v002 Insight

## Helped

## Hurt

## Takeaway

The change is in place as specified. `select_victim` now also checks the protected LRU head after the 8-entry probation scan, and it takes that head only if its estimate is strictly lower. `put` correctly decrements `prot_len` and re-tags the slot to probation. Hit-rate on workload-a is the same as the parent, so on this trace the extra candidate almost never wins. The reason is how entries reach protected. Every protected entry got there through a hit, so it already has an estimate of at least 2. Once protected is full, an entry also has to tie or beat the demotion floor to get in. The 8 oldest probation entries, by contrast, are mostly newcomers and demoted keys that have estimates near 1. Estimates are non-aging lifetime counts, so a protected head can never fall back below that probation floor, and ties go to the probation candidate. A "stale-high" protected resident is stale in recency but not in count, and a contest based only on frequency cannot see recency. To reach stale protected keys, a change has to add a recency or aging signal, such as decaying counts or discounting by time since last access. Adding protected keys to the comparator set while comparing raw lifetime estimates will not do it.
