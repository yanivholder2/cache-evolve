# Variant v002 Insight

## Helped

- hit-rate: Adding `InsertPosition::Back` on top of the 75/25 topology produced a small additional gain, confirming that MRU re-entry (instant promotion eligibility after one access) is still directionally positive even when the hot tier already has more normal-bucket slots.

## Hurt

## Takeaway

Stacking MRU re-entry onto the 75/25 topology yields only a marginal further lift in hit-rate beyond the topology change alone, indicating the two changes are not strongly synergistic. The enlarged normal-bucket pool from 75/25 already keeps re-promoted items resident long enough to earn promotion naturally, so Back-insert adds little on top — the dwell-time benefit the hypothesis expected to amplify MRU re-entry has already been largely captured by the topology itself.
