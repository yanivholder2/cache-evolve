# Variant v001 Insight

## Helped

- hit-rate: `select_victim` now walks only the probation list (via `on_hit`, any key hit while in probation moves to the protected list). So a newcomer is compared against the oldest probation entries, which have not been hit since they entered or were demoted, rather than against residents that drifted to the head of the single LRU. The min-estimate scan (`VICTIM_SAMPLE = 8`, strict `<`) already skipped most hot keys at the LRU head in the parent, so the extra shielding only matters when all 8 sampled entries had been re-referenced. That is why the gain is real but marginal.

## Hurt

## Takeaway

On clean, dedup'd sketch estimates, adding an SLRU split (20% probation / 80% protected) on top of an 8-way min-frequency victim sample adds very little on workload-a. The sampling scan was already acting as a soft protected segment, so the explicit split mostly duplicates that work. The split also adds per-hit list work. On overflow, `on_hit` demotes the protected LRU entry to the probation MRU end, which is a second-chance path. Each promotion that overflows the protected list moves two nodes, not one. Only hit-rate was scored, so that cost is unscored. The hypothesis was right that structure "pays off" on honest estimates, but only slightly, so more gain will need more than a recency split. Options include changing the split ratio, adding a separate admission window, or aging the sketch.
