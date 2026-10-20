# Variant v001 Insight

## Helped

- hit-rate: `select_victim()` now compares the three oldest probation entries instead of two and evicts the lowest sketch estimate, so a re-referenced resident that sits third from the LRU head can be passed over when the two older entries are colder; the gain is marginal and comes from a single trace and size.

## Hurt

## Takeaway

Widening the probation victim scan from two to three entries, with strict-less-than selection so exact ties still keep the older entry, is a very small positive change on workload-a at size 2048 and matches the direction H01 predicted. The movement is within what one trace/size point can distinguish, so it does not show that a wider scan is a broad improvement; a second trace or cache size is needed before treating the third candidate as a real lever.
