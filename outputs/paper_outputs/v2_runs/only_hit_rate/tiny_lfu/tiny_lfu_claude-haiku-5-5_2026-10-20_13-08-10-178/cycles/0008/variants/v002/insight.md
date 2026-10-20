# Variant v002 Insight

## Helped

- hit-rate: Scanning the two LRU-tail entries of probation and evicting the one with the lower sketch estimate keeps frequently re-referenced residents from being displaced by whichever entry is oldest, which is the mechanism H02 proposed for workload-a at size 2048.

## Hurt

## Takeaway

Choosing the colder of the two probation-tail entries is a small positive lever on this one trace and size, and the gain points the same way H02 predicted. The effect is modest and comes from a single trace/size point, so it does not establish a broad win. The variant's source overlay could not be inspected from these paths, so the attribution to the victim rule rests on the hypothesis and the fitness delta, not on confirmed code; a second trace or cache size is needed before treating it as the cause.
