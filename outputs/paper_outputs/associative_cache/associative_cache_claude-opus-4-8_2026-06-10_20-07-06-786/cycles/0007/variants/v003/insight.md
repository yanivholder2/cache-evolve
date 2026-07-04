# Variant v003 Insight

## Helped

- (none): no fitness dimension improved over the parent — the absolute-target factoring conserved capacity but did not unlock a better collision-set width.

## Hurt

- hit-rate: Replacing the parent's relative halving with an absolute `TARGET_ASSOCIATIVITY = 8` divisor search pinned `bucket_size` to the divisor of `total_capacity` nearest 8; at the power-of-two eval size 8 divides exactly, so the cache locked to a fixed 8-way split that is narrower than the wider buckets the parent's single halving produced, evicting hot keys the parent retained.

## Takeaway

The hypothesis's whole premise — reaching non-power-of-two widths between halving steps — never engages on a power-of-two budget, where the nearest divisor to 8 is just 8 itself; so the change degenerates into hard-coding an 8-way set that sits *below* the parent's associativity sweet spot rather than triangulating around it. Absolute targeting only buys resolution the relative-halving strategy lacks when the budget has non-power-of-two divisors, and a target of 8 was too narrow regardless.
