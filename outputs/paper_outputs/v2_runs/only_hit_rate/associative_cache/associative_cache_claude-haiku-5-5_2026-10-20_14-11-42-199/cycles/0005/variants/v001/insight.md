# Variant v001 Insight

## Helped

- hit-rate: The shared frequency counter gates admission on a miss, so an incoming key displaces the probationary-first victim only when its counter exceeds the victim's. This filters out one-shot inserts that would otherwise evict entries with proven reuse, and that gain stacks on top of the parent's probationary-first eviction.

## Hurt

## Takeaway

Admission gating and victim selection act on different decisions, and in this variant they combine: the gate alone reached a lower hit-rate than the parent in cycle 0004, while adding it on top of the probationary-first rule improved workload-a. The fitness vector reports only hit-rate, so the cost of the counter's metadata and any admission-bypass overhead is still unmeasured.
