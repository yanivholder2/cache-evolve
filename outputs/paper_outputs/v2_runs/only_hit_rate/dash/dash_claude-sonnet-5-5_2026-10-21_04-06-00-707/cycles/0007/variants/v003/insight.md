# Variant v003 Insight

## Helped

- None measured. hit-rate is the only reported dimension, and it did not improve over the parent.

## Hurt

- hit-rate: Per the hypothesis, keys now skip the second-sighting filter whenever the bucket or stash has an empty slot. That admits one-hit-wonders into free space, where they can later displace useful entries. The result is a small drop on workload-a, close to the noise level for a single trace. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Letting free space bypass the admission filter did not pay off, so the filter is not redundant with reuse-bit eviction on this workload. It also screens out cold keys before they occupy empty slots, not only before they evict a victim. The difference is tiny and comes from one trace, so the evidence is weak. It is enough to say the idea gives no gain, and not enough to call the filter strictly necessary.
