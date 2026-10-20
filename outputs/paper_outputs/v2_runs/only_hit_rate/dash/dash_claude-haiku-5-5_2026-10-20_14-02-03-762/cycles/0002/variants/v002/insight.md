# Variant v002 Insight

## Helped

- hit-rate: The hypothesis gates promotion on a stash item's second hit, so one-off stash hits stay in the stash instead of displacing resident items; on the LIRS workload-a trace at size 2048 this raised hit-rate over the baseline.

## Hurt

- None measured. The current fitness vector reports only hit-rate, so any cost to metadata-size (the one-bit "seen in stash" marker is the only place metadata could grow) or to per-op-time is unknown from this evaluation.

## Takeaway

Making stash promotion conditional on a second hit is a useful admission filter for the LIRS trace, which suggests one-off stash hits are a real source of displacement there. The evaluation does not yet show whether the marker stayed within the existing entry metadata, so the metadata-size and per-op-time cost of this change still needs measuring before the gain can be judged net.
