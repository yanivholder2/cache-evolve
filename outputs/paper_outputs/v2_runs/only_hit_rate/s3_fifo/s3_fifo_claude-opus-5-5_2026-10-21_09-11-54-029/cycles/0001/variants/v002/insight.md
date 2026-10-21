# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Graduating on a single small-queue hit pushes many once-reused keys straight into main. Because their frequency is reset to 0, these keys are evicted on the next `evict_main` pass unless they are hit again in main. Promotion therefore churns main and displaces established residents instead of protecting reused keys. On top of that, the reset removes the extra passes through main that baseline gave to keys hit twice or more in the small queue, so the hottest keys lost protection too. The result is a slight regression on workload-a rather than the hoped-for gain.

## Takeaway

On workload-a, a single small-queue reuse does not reliably signal a key worth keeping in main. Under baseline, those keys already reach main one miss later through the ghost queue, so promoting them earlier saves little and adds pressure on main. Combining the lower threshold with a frequency reset on graduation also removes main-queue protection for the keys that actually showed repeated reuse. The two changes landed together, so their effects can't be separated here. A follow-up should test the threshold change alone, or keep the small-queue frequency on graduation, to see which part caused the loss. Only hit-rate was evaluated for this variant, so its effect on metadata-size and per-op-time is unknown.
