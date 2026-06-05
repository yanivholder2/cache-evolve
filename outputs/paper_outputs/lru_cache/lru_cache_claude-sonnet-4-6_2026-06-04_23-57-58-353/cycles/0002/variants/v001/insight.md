# Variant v001 Insight

## Helped

_(none — no dimension improved)_

## Hurt

- hit-rate: Shrinking the protected segment from 80% to 70% of capacity caused more hot entries to be demoted back to probationary before they could absorb a second access, outweighing any benefit from the wider probationary buffer.

## Takeaway

Widening the probationary segment from 20% to 30% hurt rather than helped on workload-a: the hot working set is large enough that the 10-percentage-point reduction in protected capacity triggers more demotion churn, and that churn costs more hits than the extra probationary room saves in scan filtering. The 80/20 split established by v001 appears closer to the right balance for this workload's access pattern than a 70/30 split.
