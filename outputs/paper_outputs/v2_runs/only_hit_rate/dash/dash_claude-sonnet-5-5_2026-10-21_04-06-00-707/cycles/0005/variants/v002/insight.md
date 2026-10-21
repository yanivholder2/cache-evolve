# Variant v002 Insight

## Helped

- hit-rate: Essentially unchanged versus the parent, with only a negligible uptick at the noise level. Inserting proven (twice-seen) keys at MRU and unproven ones next to MRU slightly favours entries likely to be reused, but it barely changes which entries get victimised.

## Hurt

- None measured. hit-rate is the only reported dimension, so any cost of the extra "recently re-admitted" bit and insert-position branching in the insert path is unevaluated. I could not list or read the overlay sources (only Read/Write/Grep were available), so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

After the second-sighting filter has already screened out one-hit-wonders, the keys that reach insertion are mostly worth keeping, so ranking them within the bucket (MRU versus next-to-MRU) adds almost nothing. The parent's gain came from the admission decision, not from slot placement, and this refinement of placement on top of it did not move the metric.
