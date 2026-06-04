# Variant v001 Insight

## Helped

- hit-rate: Quadrupling the ghost ring from `capacity/4` to full `capacity` extends the evicted-key memory across the entire CLOCK ring, so cold evictions whose re-reference falls in the (capacity/4, capacity] distance band — previously overwritten in v002's quarter-length ring — survive long enough to be promoted Hot on re-put.

## Hurt

## Takeaway

Stretching the ghost ring to full capacity gives only a marginal lift over v002, which means the re-reference distribution on lirs:multi1 has a thin tail past the quarter-capacity mark — most of the recoverable evicted-then-re-touched keys were already inside v002's 128-slot window, and the extra 384 slots only catch a sliver more. The ghost-sizing knob is in strongly diminishing returns on this trace; future gains on the CLOCK-Pro-lite skeleton likely live in policy (how ghost-hits seed Hot state, Hot-pool budgeting) rather than in pushing ghost length further.
