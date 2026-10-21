# Variant v002 Insight

## Helped

- (none) No dimension improved. The only reported dimension, hit-rate, moved the wrong way.

## Hurt

- hit-rate: Growing the window from ~1% to ~10% of capacity slightly lowered hit-rate versus the parent. Keys now sit in the window longer and take capacity from the probation/protected main region. The sketch-gated window-to-probation duel is also applied to a larger pool of candidates. Together these dilute the frequency-based protection that the parent's main region gave to re-referenced keys. The loss is small.

## Takeaway

On workload-a at size 2048, the trace is not recency-biased in a way a bigger window can use. The extra window space is worth less than the main-region space it replaces. Window size looks like a weak or negative lever here, so later cycles should try something other than enlarging it, such as a better admission or aging policy.
