# Variant v001 Insight

## Helped

- hit-rate: Replacing the fixed `prob_head` victim with `sample_prob_victim()` — a bounded head→tail scan of up to `SAMPLE=5` probationary nodes that keeps the one with the lowest `sketch.estimate` — lets eviction drop the genuinely coldest probationary key instead of merely the most-LRU one, so a few low-frequency keys that recency would have spared are now correctly displaced, lifting hit-rate by a hair.

## Hurt

- (none observed): hit-rate is the only dimension in the vector and it did not regress; the extra ≤5-node scan and per-candidate `estimate()` calls in the full-cache branch carry no measured cost here.

## Takeaway

On workload-a at size 2048, combining SLRU recency-protection with frequency-ranked victim sampling *inside* the probationary tier nudges hit-rate just past the parent, but the gain is marginal: the `incoming_freq > victim_freq` admission gate was already filtering most cold arrivals, so sharpening which probationary key leaves only matters for the small set of cases where the LRU head was not also the coldest of its first five neighbors. The two levers compound, but barely clear the noise floor each hovered at alone.
