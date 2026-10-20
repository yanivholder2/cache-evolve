# Variant v002 Insight

## Helped

- None observed. Only hit-rate was measured, and it did not improve over the parent.

## Hurt

- hit-rate: Main evictees that skipped the ghost were apparently still useful as ghost-hit candidates. Dropping them gave the ghost fewer re-admission opportunities, which cost a very small amount of hit-rate. The drop is within noise of the parent. I could not read the overlay source, so this comes from the hypothesis and the fitness vectors.

## Takeaway

Filtering which main-queue evictees enter the ghost did not make the shortened ghost any more useful on this trace and size. Main evictees that reached freq 0 still carried enough reuse signal that discarding them lost a little, and none of the pollution the hypothesis expected showed up. Ghost-content selection looks like a dead end here, along with the capacity ratios the parent already found saturated. Later cycles should look at the promotion or main-queue scan logic.
