# Variant v002 Insight

## Helped

- hit-rate: No movement, which is the intended outcome. Swapping the MRU move from `remove(position)+push` to a single `self.items[position..].rotate_left(1)` produces a byte-identical final vector, so the protected suffix, probationary prefix, and eviction order are all unchanged.

## Hurt

- hit-rate: No regression. The rotate slides the hit element to the back and shifts `[position+1, len)` down by one — exactly the reorder remove+push performed — with no length change, so the `protected_count` invariant and every promotion/admission decision behave identically.

## Takeaway

This variant proves the get-hit promotion can be expressed as one contiguous suffix rotation rather than a remove followed by a capacity-checked push, with provably no effect on cache state. The intended payoff — shedding the redundant `push` capacity check and separate re-insertion on the `get` p50 path — lives on a per-op-time axis this hit-rate-only fitness vector cannot observe, so the rewrite is provably free here but not provably beneficial under the current measurement.
