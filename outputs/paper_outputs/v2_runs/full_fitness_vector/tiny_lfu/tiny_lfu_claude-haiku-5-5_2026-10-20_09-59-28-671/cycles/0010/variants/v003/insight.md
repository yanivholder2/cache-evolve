# Variant v003 Insight

## Helped

- per-op-time: The get hit branch now returns early when the hit node is already the MRU head of its segment, skipping the unlink and relink, and the hash is derived once per get instead of twice; both remove pointer and hashing work from the get path.

## Hurt

- None measured. Hit-rate and metadata-size are unchanged from the parent, consistent with the hypothesis's claim that eviction order and metadata layout were left alone.

## Takeaway

The change is a pure per-op optimization: the already-MRU short-circuit and single hash computation cut get-path cost while leaving eviction behavior intact, as the unchanged hit-rate confirms. The put-path timing also improved even though the hypothesis only touched get, so that gain is not explained by this change and should be confirmed with the repeated runs the hypothesis calls for before it is credited to the edit.
