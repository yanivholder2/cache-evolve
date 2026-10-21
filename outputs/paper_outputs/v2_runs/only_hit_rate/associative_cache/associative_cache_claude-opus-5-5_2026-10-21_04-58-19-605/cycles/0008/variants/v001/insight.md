# Variant v001 Insight

## Helped

(No measured dimension improved. The fitness vector contains only `hit-rate`.)

## Hurt

- hit-rate: H01's refutation clause applies: hit rate dropped slightly below the parent's. Lifting a resident's first hit from counter 0 to `FIRST_HIT_COUNTER = 2` means a key that is hit once and then goes cold now holds its slot for two sweep passes instead of one. In a small bucket, every extra pass a stale key survives makes the hand decrement and evict someone else, often a ghost-admitted or freshly admitted newcomer that would have been reused. The jump also uses up most of the `CLOCK_MAX = 3` range in one hit, so a key now saturates on its second hit instead of its third. That leaves less room to tell "hit once" apart from "hit repeatedly", which is exactly the signal GCLOCK victim choice depends on.

## Takeaway

On this GCLOCK/LIP line, with a saturation cap of 3 and a 4-entry fingerprint ghost ring, one pass of protection after a first hit is already enough on workload-a at 2048. Making the first-hit reward bigger mostly shields once-touched keys and flattens the counter scale; it does not reward real reuse. The small, uniform +1 step (the same as the ghost-admission reward) is the better setting, so cycle/0007/v001 remains the stronger base. The regression is small, though. A future probe of hit-side magnitude should not raise the jump again. It should first raise `CLOCK_MAX`, so that a larger first-hit reward does not eat into the frequency range.
