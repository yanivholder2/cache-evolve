# Variant v001 Insight

> Note: I could not read the overlay source or manifest.toml in this session. I had no directory listing, and the paths I guessed did not exist. This insight is based on hypothesis H01, the parent's insight, and both fitness vectors. I have not checked the diff against H01's Changes list.

## Helped

- hit-rate (web_0, size 131072): This is the target cell, and it recovered most of the way toward the level H01 aimed for. Letting a ghost-hit returner displace a counter-1 slot in a window with few warm slots does rescue the large-size web_0 returners that the strict counter-0 gate turned away. This confirms the parent insight: the loss came from the admission rule, not from a lack of placement targets.
- hit-rate (hm_0, size 131072): This cell was flat to marginally up. At the large size, hm_0 windows rarely reach the relaxed branch, or only cheap residents are displaced when they do.

## Hurt

- hit-rate (hm_0, size 16384): This cell regressed, which is the loss H01 said the warm-count guard would prevent. Contended small-size hm_0 windows evidently also hold at most 2 slots with counter ≥ 2. Their residents are mostly counter-1 too, so the guard does not tell them apart from web_0's large-size windows. The relaxed branch fires there and evicts counter-1 residents that would have been re-hit.
- hit-rate (web_0, size 16384): This cell dipped slightly, for the same reason. Under small-cache contention, a counter-1 resident is worth more than an unproven returner with a credit of 1.

## Takeaway

Counting counter-≥2 slots in the 8-slot tail window does not separate the two workloads. Both web_0 at 131072 and hm_0 at 16384 have windows made up mostly of counter-1 slots. So the gate brings back the same size trade-off as the global threshold it replaced: web_0 large gains and hm_0 small loses, and the mean does not improve. Lowering `WARM_TOLERANCE` to 1 will most likely just move along that same trade-off. Fixing both cells needs a signal the window's counter histogram does not hold, for example something about the returner itself or how recently its ghost entry was recorded.
