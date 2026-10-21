# Variant v001 Insight

## Helped

- hit-rate: Two changes were made together. `evict_one()` now lowers a promoted key's `freq` by one instead of resetting it to zero, and `MAX_FREQ` went from 3 to 7. As a result, a key hit two or more times while in `small` now arrives in `main` with credits. Those credits buy it extra trips through `main`'s lazy-reinsertion loop before it can be evicted, while keys hit only once still arrive with `freq = 0`. The gain over the parent is real in direction but tiny. That fits the fact that few keys build up multiple hits during the short stay in `small` (`small_target` is 51 at size 2048).

## Hurt

(No measured dimension regressed. The change's reach is limited by the parent's own layout. Per the parent's insight, more keys now reach `main` through a ghost hit. That path still sets `freq = 0`, so carryover never reaches the admission route that is growing.)

## Takeaway

Carrying surplus credits from `small` into `main` is safe: there was no regression, the ordering rule is clean, and it terminates. But on workload-a at size 2048 it barely matters, because with a 51-slot `small` few keys pick up more than one resident hit before promotion. This variant changed `MAX_FREQ` and carryover together, and its parent has neither. So this comparison cannot separate carryover's share of the small gain from the higher cap's. The hypothesis's own decision rule needs cycle 7 v001's result (`MAX_FREQ = 7` alone) to answer that. On this variant's evidence alone, adding more credit to keys promoted from `small` is close to exhausted while `small` stays this small.
