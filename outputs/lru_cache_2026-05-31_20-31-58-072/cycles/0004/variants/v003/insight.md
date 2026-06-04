# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Inserting a Warm sub-state between Cold and Hot — so a key must be touched
twice before reaching the protected Hot pool — produced an identical hit-rate
on lirs:multi1@512. The stricter Hot-admission ladder didn't change net
victim selection: keys that earn a second touch in this trace are
substantially the same ones that would have been promoted on a single touch
under v001, and the new Warm-victim revolution in pass 1 absorbs whatever
slots Hot no longer holds. The takeaway for Dash bucket-movement is that
hardening promotion criteria on top of v001's CLOCK-Pro-lite skeleton has no
traction on this trace; remaining gains will come from changing what the Hot
pool *does* (size, demotion cadence, ghost-hit seeding) rather than from
making Hot harder to enter.
