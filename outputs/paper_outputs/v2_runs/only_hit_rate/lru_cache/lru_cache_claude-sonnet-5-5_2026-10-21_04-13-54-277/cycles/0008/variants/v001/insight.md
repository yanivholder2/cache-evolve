# Variant v001 Insight

## Helped

- hit-rate: Only a negligible, noise-level gain over the parent. The adaptive `protected_cap`, driven by ghost-list hits and protected-tail hits, occasionally shifted the segment split in a useful direction.

## Hurt

- hit-rate: No regression on the single measured dimension. The ghost FIFO and the demotion path add bookkeeping and complexity (not captured by this fitness vector) for essentially no return.

## Takeaway

On this trace the fixed protected share was already near its optimum, as the hypothesis's own rationale suspected. Letting the ARC-style feedback loop resize the segments found almost no trace-phase variance to exploit. Further gains will probably need to come from something other than tuning the probation/protected split, such as the admission gate or the eviction signal.
