# Variant v002 Insight

## Helped

- hit-rate: Stacking the `STASH_PROMOTE_THRESHOLD = 2` frequency gate in `promotion_get` on top of the inherited even 16/16 split held one-hit-wonder stash residents in place instead of promoting them, sparing a few hot target keys from displacement — a marginal nudge above the parent's even-split-only score.

## Hurt

- (none): The gate keeps the held item in its legal stash bucket and only promotes once the counter reaches the threshold, so capacity conservation and slot/uniqueness invariants stayed intact and no dimension regressed.

## Takeaway

On this trace the frequency-gated admission filter barely moves hit-rate once the even normal/stash split is already in place: the two cycle-0001 stash-protection levers are largely redundant, not additive. Enlarging the protected tier (capacity reallocation) already captures most of the benefit that selective admission would otherwise provide, so layering the gate on top yields steeply diminishing returns.
