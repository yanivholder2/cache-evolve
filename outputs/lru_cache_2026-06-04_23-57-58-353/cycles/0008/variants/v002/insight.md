# Variant v002 Insight

## Helped

_(none)_

## Hurt

- hit-rate: Widening probationary from 20% to 30% (shrinking protected from 80% to 70%) reduced hit-rate, because the hot working set in workload-a is large enough that demoting the protected boundary forces genuinely warm entries to compete inside the probationary segment more often — even though IRR timestamps are preserved, more hot entries are displaced before they can win re-promotion contests.

## Takeaway

The hypothesis assumed that IRR timestamp preservation would let warm entries self-protect inside a larger probationary zone, but workload-a's hot working set is dense enough to fill more than 70% of capacity: reducing the protected segment size triggers excess demotion churn that cancels the larger cold-filter benefit. This confirms the hypothesis's own warning condition — the 80/20 split appears to be closer to the minimum protected fraction needed to hold this trace's active set without performance loss.
