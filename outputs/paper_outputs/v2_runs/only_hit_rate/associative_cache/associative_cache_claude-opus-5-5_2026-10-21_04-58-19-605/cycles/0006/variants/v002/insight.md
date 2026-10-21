# Variant v002 Insight

## Helped

(No dimension improved. H02 expected hit rate on workload-a to rise above the parent's level, and it did not.)

## Hurt

- hit-rate: Hit rate came in slightly below the parent, so H02 is refuted. Strict leaders run exactly the parent's code, and ghost admission is the same under both policies. The only behaviour that changed is where the hand rests after a full-bucket replacement. Lenient placement moves the hand past the newcomer, so a one-shot key gets a full extra sweep before it can be evicted, and here that costs hits instead of saving them. The loss comes from one of two places. If PSEL settled at or below zero, followers stayed strict and the loss is the 1/32 of buckets that are permanently lenient leaders. If PSEL went positive at times, followers adopted lenient placement and lost hits while it was on. Either way, the extra grace does worse than the parent's strict hand-on-newcomer filter.

## Takeaway

This change compared strict and lenient placement under fixed policies, so neither policy could bias the evidence for itself. That removes the flaw in cycle 0005's self-referential duel, and lenient still did not win. On workload-a, the ghost ring's counter-1 admission is already enough protection for keys that return. Giving every newcomer a sweep of grace mostly shelters scan or one-shot keys, and dueling costs hits even when it settles on strict, because the lenient leaders keep running. This closes the newcomer-leniency question for this trace. "Strict hand-on-newcomer plus a capped ghost counter" should be treated as the settled insertion shape, and later variants should look at other levers such as ghost-ring depth or how promotion on a hit works.
