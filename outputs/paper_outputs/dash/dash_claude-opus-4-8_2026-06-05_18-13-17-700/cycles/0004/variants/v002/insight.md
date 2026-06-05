# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Admitting new keys straight into their target bucket (`Slot::Target`, `Front`) instead of the stash let every cold one-hit-wonder immediately displace resident keys, cascading warm items out through `demote_from_normal_bucket` and dropping hit-rate well below baseline.

## Takeaway

The stash-admission step is not just probation delay — it is a filter that absorbs cold/scan traffic before it can touch the target/probing tiers. Removing admission probation and keeping the stash only as a demotion sink means the hot tiers eat the full churn of first-time keys, so on this trace the "probation cost" was actually warm-set protection that this change destroyed.
