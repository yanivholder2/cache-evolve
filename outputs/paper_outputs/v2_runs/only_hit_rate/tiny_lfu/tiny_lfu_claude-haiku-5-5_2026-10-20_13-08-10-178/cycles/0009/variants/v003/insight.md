# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The new rule in `CountMinSketch::increment()` halves every counter in every row whenever any one of the key's counters reaches `SAT_THRESHOLD`. Because the halving is table-wide rather than per key, one hot key saturating erases admission history for all cold keys, and `put()`'s `incoming_freq > victim_freq` comparison loses the ordering it depended on.

## Takeaway

Making the aging trigger frequency-based instead of access-count-based does not avoid the hit-rate loss, because the decay itself is global: the hypothesis's claim that only hot keys are reshaped does not hold once one saturated counter halves the whole sketch. Any future aging rule that keeps the admission comparison intact needs to decay per key or per saturated row, not the entire `counts` table.
