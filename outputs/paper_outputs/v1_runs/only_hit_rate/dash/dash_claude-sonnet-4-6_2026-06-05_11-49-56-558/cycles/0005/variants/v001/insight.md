# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Inserting ghost-re-admitted items at Back (MRU) of the target bucket moved hit-rate slightly below the parent, suggesting that placing historically-cold stash-evicted items at the hottest position prematurely displaces warmer residents from the target tier.

## Takeaway

The hypothesis that Front-insertion was neutralising ghost re-admission's benefit is not supported: switching to Back-insertion made hit-rate marginally worse, not better. Stash-evicted items are by definition the coldest recently-active keys, so granting them immediate MRU status — and thus immediate promotion candidacy — appears to harm the tier by crowding out legitimately hot items. The ghost set mechanism continues to deliver only noise-level movement on this workload regardless of re-admission insertion position, indicating the stash eviction boundary is not where meaningful hit-rate is lost.
