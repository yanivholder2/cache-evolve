# Variant v001 Insight

## Helped

- hit-rate: Halving the ghost queue to max(1, capacity/2) gives a slightly stricter admission test, because only recently evicted small-queue keys can bypass probation. The gain over the parent is marginal.

## Hurt

- None observed. Only hit-rate was measured. A shorter ghost memory could miss legitimate re-references with long reuse distances, but this trace and cache size didn't show it. I could not read the overlay source, so this comes from the hypothesis and fitness vectors only.

## Takeaway

The small-queue shrink and the shorter ghost are not meaningfully additive. Each tied or barely beat its parent on its own, and combining them gave only a tiny gain. Admission-side sizing (small ratio, ghost length) looks saturated on this trace, so later cycles should change the promotion or eviction logic, not these capacity ratios.
