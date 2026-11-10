# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): This was the target cell and it moved up, but only slightly. Cutting small from 25% to 10% gives roughly 20k more slots to main and to the ghost bound (`ghost_live <= main_size`). That keeps a few more proven keys resident and remembers a few more small victims. The gain is again marginal. The split is a real lever for web_0, but it is a weak one, as the cycle-0007 main-victim ghosting was.

## Hurt

- hit-rate (hm_0 @ 131072): This cell regressed by more than web_0 gained. A 10% small queue gives a new key only about 40% of its former small residence to collect the reuses it needs to graduate (two once main is full). So hm_0 keys with medium reuse distances, which used to qualify in small, are now ghosted at freq 0 instead. The pinned `recent_window` (32768 ghost insertions) still lets some of them straight back into main, but the ones that return later now need a second pass through small.

The small-size cells (both traces @ 16384) are bit-identical to the parent. This is what the hypothesis predicted, and it confirms that the size gate, the pinned window and the `Reference` oracle changes are isolated to capacities >= `LARGE_CAPACITY`.

## Takeaway

Shrinking small only at the large size, with the age window held fixed, cleanly separates the split's effect. That effect pulls the two traces in opposite directions: web_0 gains a little from the bigger main, and hm_0 loses more from the shorter small-queue probation. The summary ends up slightly below the parent. The pinned window is now about 2.5× small's turnover rather than 1×, so the "strong returnee" gate is looser here than in the parent. If the split is probed again, a midpoint (about 15–20%) or a window rescaled to the new `small_size` would show whether the hm_0 loss comes from the shorter probation or from the mismatched window. Either way, web_0 @ 131072's plateau does not seem to be explained by the small/main ratio either.
