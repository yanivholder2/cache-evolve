# Variant v003 Insight

## Helped

- None measured. The hypothesis targeted per-op-time by returning early on resident hits before the ghost-membership lookup, but this variant's fitness vector reports only hit-rate, so the claimed speedup is unverified.

## Hurt

- None measured. Hit-rate is unchanged from the parent (v001), which is the control the hypothesis required: the early return on resident hits should leave eviction order untouched.

## Takeaway

Skipping the ghost lookup on resident hits is hit-rate neutral, as the hypothesis predicted, but this variant's fitness vector omits per-op-time and metadata-size, the two dimensions the change was meant to move. The cost benefit is therefore still open, and a rerun that reports those dimensions is needed before v003 can be judged on the change it made.
