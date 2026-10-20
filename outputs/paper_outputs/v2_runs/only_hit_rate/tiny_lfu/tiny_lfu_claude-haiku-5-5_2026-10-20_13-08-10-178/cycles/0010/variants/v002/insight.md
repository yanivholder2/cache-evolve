# Variant v002 Insight

## Helped

- hit-rate: Raising `TAIL_SCAN` from three to four lets `select_victim()` consider one more probation entry from the LRU end, so a re-referenced resident sitting fourth from the head can be passed over when the three older entries are colder; the gain is very small and comes from a single trace and size.

## Hurt

## Takeaway

Extending the probation victim scan from three to four entries moved hit-rate in the same direction as the prior step, but by a smaller margin, which suggests the benefit of comparing additional tail candidates is tapering rather than still compounding. The change touched only the constant and its doc comments, with the tie rule and admission comparison unchanged, so the result isolates the scan width. A single trace at one size cannot separate a real plateau from noise, so the open question is whether the scan width keeps helping on a second workload or cache size.
