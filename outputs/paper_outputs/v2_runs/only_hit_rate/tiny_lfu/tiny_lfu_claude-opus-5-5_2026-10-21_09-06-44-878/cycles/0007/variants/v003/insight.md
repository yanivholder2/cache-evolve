# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Moving the SLRU split from 80/20 to 50/50 (`PROTECTED_NUM/PROTECTED_DEN`) left workload-a hit-rate flat; it moved by a rounding-level amount. So the length of the probation trial is not what limits newcomers. Two things explain this. First, the extra probation slots are not reserved for new arrivals. Shrinking protected sends more overflow demotions to the probation MRU end, so admitted newcomers share the larger segment with demoted keys and wait about as long as before to reach the head. Second, reaching the head does not decide eviction anyway. `select_victim` takes the min-estimate key among the 8 oldest, and the sketch never ages its counts, so survival depends on lifetime frequency rank and hardly at all on where the protected/probation boundary sits. Total capacity is unchanged, and both segments only hold keys that already passed the frequency gate. As long as eviction is driven by unaged min-frequency sampling, the segment ratio is not a useful lever. The next thing to test is count aging (periodic halving), which would make recency affect the estimates directly.
