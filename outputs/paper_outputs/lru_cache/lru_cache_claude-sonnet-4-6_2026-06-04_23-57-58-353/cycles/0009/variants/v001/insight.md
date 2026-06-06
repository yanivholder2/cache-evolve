# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Removing ghost admission hurt performance — the ghost set was routing recently evicted probationary keys directly into protected on re-arrival, and eliminating that shortcut forces them through the normal one-hit-then-promote path, costing at least one additional probationary miss per re-arriving key on workload-a.

## Takeaway

The IRR timestamp tie-breaker adds no measurable lift because frequency ordering already differentiates candidates within the narrow scan window almost all the time, just as the parent's `referenced` bit did not — the tie-breaker only activates on frequency ties, which are rare. The net effect is therefore determined almost entirely by the ghost-set removal: workload-a contains enough recurring key churn that ghost admission was actively recovering re-arriving warm keys back into protected in a single step, and discarding that mechanism cost hit rate. The hypothesis correctly identified ghost admission as an exhausted *improvement direction*, but it was still carrying a positive baseline contribution that IRR cannot substitute for.
