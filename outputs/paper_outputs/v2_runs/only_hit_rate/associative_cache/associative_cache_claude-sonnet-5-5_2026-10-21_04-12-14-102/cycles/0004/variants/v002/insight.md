# Variant v002 Insight

## Helped

- No dimension improved. The stricter promotion rule did not add a useful reuse signal on this trace.

## Hurt

- hit-rate: Requiring a second probation hit before promotion (a one-bit "seen once" flag) slightly lowered hit-rate versus the parent. Entries that were reused once and then kept being reused stayed in the small probation segment (8 slots) for longer. There they were exposed to eviction by new insertions, so some entries the parent would have protected were lost. Note: I could not read the overlay source, so this reading rests on the hypothesis and the fitness vectors. Only hit-rate was measured, so per-op-time and metadata cost are unobserved.

## Takeaway

With probation already shrunk to 8 slots, delaying promotion by one hit costs more than it gains. First-hit promotion looks to be the better trade on this LIRS trace, because the probation segment is too small to hold entries while they wait for a second hit. A stricter promotion filter would need either a larger probation segment or a ghost-based signal that does not hold the entry in probation.
