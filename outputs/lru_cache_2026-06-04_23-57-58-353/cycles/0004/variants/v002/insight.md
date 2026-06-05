# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Expanding `ghost_cap` from `prob_cap` (~410) to `capacity` (2048) caused more evicted keys to qualify for ghost-hit fast-path admission into the protected segment, but those additional hits correspond to keys with long reuse distances that lack the recurrence pattern to justify protected placement — polluting the protected segment and accelerating demotion of genuinely hot items.

## Takeaway

Setting `ghost_cap = capacity` tests the full ceiling of reuse-distance tracking, and the hit-rate regression shows that reuse events at distances between `prob_cap` and `capacity` on workload-a are not reliable locality signals: admitting those keys directly into the protected segment displaces items with stronger recurrence. The ghost window appears to have an effective upper bound somewhere below total capacity for this trace, meaning the right next step is a proportional step (e.g., 50% of capacity) rather than the full expansion.
