# Variant v001 Insight

## Helped

- None. hit-rate is the only dimension reported, and it is identical to the parent's.

## Hurt

- None measurable. hit-rate is unchanged, and metadata-size and per-op-time were not evaluated. I could not list or diff the overlay sources (no directory-listing tool was available), so this rests on the hypothesis text and the fitness vectors.

## Takeaway

Replacing the victim choice with a "furthest from MRU" scan over the target bucket, probing bucket and stash left hit-rate on workload-a exactly where the parent's MRU promotion put it. That suggests the scan either never ran, or picked the same victim the parent's ordering already implied. The hypothesis admitted the victim-selection function had not been located, so the change may not have reached a live code path. The result shows that victim selection is not a lever on this trace, at least not in the form tried. Before attributing this to eviction policy, confirm the edited function is actually executed during the evaluation.
