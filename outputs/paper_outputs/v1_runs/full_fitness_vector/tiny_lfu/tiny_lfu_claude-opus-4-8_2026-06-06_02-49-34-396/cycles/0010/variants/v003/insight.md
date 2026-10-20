# Variant v003 Insight

## Helped

- hit-rate: Swapping LRU-window victim sampling for random direct-index draws left hit-rate flat-to-marginally-up, confirming the cross-cycle finding that eviction is accuracy-saturated — victim quality is set by the sketch estimate, not by where the candidates are sampled from, so the sample *source* is irrelevant to admission quality.
- metadata-size: Parking `rng_state` on `TinyLfuCache` (struct grew 128→136) instead of on `Node` kept the scored node layout at 24 bytes exactly as the hypothesis intended, so the reported metadata dimension did not move.

## Hurt

- per-op-time: The predicted put speedup inverted — both get and put p50/p99 rose. Random index draws into `nodes` hit the *same* unpredictable physical slots the link chase did, so the random-`Vec` cache misses the hypothesis blamed on pointer-chasing were never the chain's fault; they are inherent to indexing a recency-ordered list by anything other than sequence. The rewrite kept that miss pattern and stacked per-sample PRNG arithmetic (`next_rng` xorshift + Lemire multiply-shift) on top, and the extra `u64` field nudged the get path too.

## Takeaway

The walk's cost was misattributed: cache misses come from touching `nodes` at order-uncorrelated positions, a property shared by both the link chase and random index sampling, not from the link traversal itself. Removing the chain dependency therefore bought nothing on locality while adding PRNG overhead, so this change is a pure per-op-time regression — and proves any future put-path speedup must change *how* victim slots are laid out, not just how they are selected.
