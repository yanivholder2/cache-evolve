# Variant v002 Insight

## Helped

## Hurt

- hit-rate: The admission gate compares two estimates on different scales. Incoming keys are scored by 4-bit counters incremented on every miss and get, while resident keys are scored by the bucket's hit counter, which counts only hits and resets on eviction. A newly inserted resident starts near zero, so the gate rarely rejects anything and the admission check mostly fails to filter one-hit wonders on workload-a.

## Takeaway

Gating inserts on frequency did not raise hit-rate on workload-a; it lowered it slightly relative to v001. The hypothesis assumed the two frequency estimates were comparable, but the spec scores residents by hits and newcomers by misses plus gets, so the gate is biased toward admitting and a frequency-based admission policy needs a shared counter for both sides before it can be judged. I could not inspect the overlay source in this session, so the mechanism above is inferred from the hypothesis's definition of the estimates rather than confirmed in code.
