# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Applying back insertion to target→probing promotion alongside demotion back insertion creates interference—promoted items landing at probing's MRU end compete poorly with the tier's eviction cadence, losing the benefit of forced re-earning that Front insertion provides.

## Takeaway

Universalizing MRU positioning across both demotion and promotion paths does not synergize as expected. While v003's demotion back insertion (restoring demoted items to MRU) preserves value for items proven by tier advancement, v001's addition of promotion back insertion (placing newly promoted items at MRU in probing) destabilizes the probing tier's effectiveness. The asymmetry matters: the MRU principle works when rewarding items with proven history, but placing freshly promoted items at MRU in a higher tier costs them the discipline of earning residency before competing for eviction, reducing overall cache hit rate. The two strategies saturate at v003's level rather than amplifying each other.
