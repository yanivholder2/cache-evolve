# Variant v003 Insight

Note: the overlay sources could not be listed or diffed in this session (no file-listing tool was available). This insight rests on the hypothesis (H03) and the two fitness vectors, not on a code diff.

## Helped

- hit-rate: Nothing moved, which is what the hypothesis required. The change reused precomputed indices without touching counters, aging or admission, so hit-rate matched the parent exactly.

## Hurt

- per-op-time: The hypothesis targeted this dimension, but the fitness vector has no entry for it. The claimed speedup from hashing once per access is still unmeasured.
- metadata-size: There is no entry for it either, so the effect of caching the per-call `Indices` array in the cache struct on footprint is unknown.

## Takeaway

Sharing the sketch indices across estimate and increment appears to be behavior-preserving. The evaluation only measured hit-rate, so it cannot show whether removing the redundant hashing paid off. A kept-or-dropped decision on this change needs a run that records per-op-time.
