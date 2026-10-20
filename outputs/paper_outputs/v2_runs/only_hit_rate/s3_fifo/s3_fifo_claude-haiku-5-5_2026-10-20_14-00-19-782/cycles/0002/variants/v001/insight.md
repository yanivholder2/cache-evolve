# Variant v001 Insight

## Helped

- None measured. The hypothesis targeted per-op-time (replacing the O(main_size) ghost scan in `put()` with a `HashSet` membership check), but this variant's fitness vector reports no per-op-time value, so the claimed speedup is unverified.

## Hurt

- None measured. Hit-rate is unchanged from the parent, which is the control the hypothesis required: the ghost set mirrors the existing ghost queue, so eviction order should be identical.

## Takeaway

The change is a pure cost refactor: it keeps hit-rate flat, which confirms that the ghost-membership lookup can move to a set without altering routing. Whether it actually reduces per-op-time or raises metadata-size is still open, because this variant's fitness vector omits the per-op-time and metadata-size dimensions the hypothesis asked to be reported. A rerun with those dimensions is needed before this variant can be judged on cost.
