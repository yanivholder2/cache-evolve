# Variant v001 Insight

## Helped

- hit-rate: Per the hypothesis, victim choice now uses a one-bit referenced flag. The flag is set on hit and cleared on insert, and the evictor takes the first unreferenced slot instead of the position-based victim. This keeps entries that have actually been reused. Hit-rate on workload-a rose over the parent, a clear step up compared with the parent's own marginal gain. It comes from only one trace, though, so the size of the gain is not established.

## Hurt

- None measured. hit-rate is the only reported dimension. The hypothesis says the flag lives in spare metadata bits, so memory should not grow. The extra bucket and stash scan on eviction could cost CPU time, and no dimension captures that. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Evicting by a real reuse signal (a hit since insertion) moved hit-rate where the earlier position-based victim tweaks reportedly could not. That suggests the eviction side, not the admission filter, held the remaining headroom. The change is cheap in metadata, so it is worth keeping and checking on more traces.
