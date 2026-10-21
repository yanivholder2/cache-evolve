# Variant v001 Insight

## Helped

(None. `hit-rate` is the only measured dimension, and it regressed. `metadata-size` and `per-op-time` were not measured. The change does not affect `metadata-size`, and it swaps one O(1) `VecDeque` push for another, so `per-op-time` should not move either.)

## Hurt

- hit-rate: In the ghost-hit branch of `put`, the re-admitted entry now goes in with `main.push_back` (the tail) instead of `push_front`. That makes it the first candidate for the next `evict_main()`. In steady state main is full, so every later ghost hit or small→main promotion triggers that eviction. The probation window therefore shrinks from roughly one lap of main (~1946 insertions) to the gap until the next main insertion. That gap is much shorter than even the 102-slot small queue, so most re-admitted keys are evicted before their reuse comes back.

## Takeaway

Tail insertion is not a "second reuse signal" rule in this design. The deadline it sets is the next main insertion, not a fixed span of the access stream, and in steady state that deadline is very close, so a ghost hit buys almost no residency. The parent's insight says the ghost does most of the filtering on workload-a, and this result supports it: the ghost hit is itself the reuse signal worth honouring, and adding a near-immediate eviction test on top throws those keys away. Making re-admission stricter only pays if the probation window is long enough to see a real reuse. Pushing to the tail sits at the extreme where the window is close to zero.
