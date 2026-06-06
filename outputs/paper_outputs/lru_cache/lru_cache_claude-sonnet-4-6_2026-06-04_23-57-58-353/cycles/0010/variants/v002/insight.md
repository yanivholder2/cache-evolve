# Variant v002 Insight

## Helped

- hit-rate: Restoring the bounded ghost set re-routes recently evicted probationary keys directly into protected on their next insertion, eliminating the extra probationary miss that the one-hit-then-promote path requires without ghost.

## Hurt

## Takeaway

Ghost admission and IRR are structurally orthogonal: ghost acts at insertion time to decide which segment a re-arriving key enters, while IRR acts at eviction time to rank candidates within a segment. Adding the ghost set back on top of the cycle-9 IRR base recovered the hit-rate that cycle 9 lost when ghost was removed, confirming that workload-a contains enough recurring key churn that the single-step re-admission shortcut carries measurable positive value independent of how eviction victims are ranked.
