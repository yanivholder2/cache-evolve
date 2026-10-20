# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Freezing promotions once the protected segment hits a fixed half-capacity cap, and parking further probation hits at the MRU end of probation instead of demoting the protected tail, slightly reduced hit rate against v002, so the protected segment's size was not the limit on the segmentation gain.
- per-op-time: Get latency rose on both p50 and p99 against v002, which fits an extra count comparison on every probation hit in the hit path; this is a single run, so the size of the regression is uncertain.

## Takeaway

Capping the protected segment without demotion held metadata flat but cost hit rate, so the cap is the wrong lever: the gain from segmentation depends on letting the protected set turn over, not on bounding it. The metadata and admission/eviction paths were untouched, so any future size or cap experiment should expect the same trade-off unless it also allows protected-tail demotion.
