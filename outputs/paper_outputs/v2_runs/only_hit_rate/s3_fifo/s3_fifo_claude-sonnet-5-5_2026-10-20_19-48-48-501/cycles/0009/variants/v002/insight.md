# Variant v002 Insight

## Helped

- (none): No dimension improved over the parent. hit-rate is the only reported dimension, and it came out identical to the parent's.

## Hurt

- (none): hit-rate did not regress. per-op-time and metadata-size were not reported, so any cost of the changed promotion path is unmeasured.

## Takeaway

Raising the hit-path promotion threshold from freq 2 to 3 and resetting freq to 1 on promotion left hit-rate exactly tied with the parent on the single measured trace and size. On this workload, the promotion gate and the carried-over count seem not to decide which keys end up in main. The gate may rarely separate keys at this cache size, or the freq-1 reset may be undone by later hits. This continues the pattern that tuning promotion parameters ties, while the parent's structural changes (hit-path action and sizing) were what moved the result. I could not diff the overlay source in this session, so this reading rests on the hypothesis text and the matching fitness vectors.
