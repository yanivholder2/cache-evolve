# Variant v001 Insight

## Helped

- None. hit-rate is the only reported dimension, and it is identical to the parent's on workload-a.

## Hurt

- None measurable. hit-rate is unchanged, and no other dimensions were reported, so metadata-size and per-op-time were not evaluated. I could not read the overlay sources or diff them against the parent, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Changing the insert path to place new keys at the MRU end (`InsertPosition::Back`) had no effect on hit-rate for this trace and cache size. Either the new-key placement is already effectively MRU, or insert-time position is not what drives hits here, unlike the hit-promotion position in `get` that the parent changed. The exact match with the parent also suggests the overlay may not have altered the path that matters. The parent's promotion gain does not carry over to insertion by analogy.
