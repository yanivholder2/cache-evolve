# Variant v002 Insight

## Helped

- (none): The aging pass did not improve any measured dimension; the hypothesis that stale high-frequency entries were pinning capacity is not supported on this workload.

## Hurt

- hit-rate: The new `evict_item` halves every survivor's `lfu_counter` (`>> 1`) on each eviction, eroding exactly the accumulated frequency signal that gave the parent its edge — proven-hot keys lose their accumulated counts under eviction pressure and become victims sooner, flattening victim selection back toward near-recency and discarding reusable working-set members.

## Takeaway

On this workload the dominant pressure is capacity collision, not staleness: the parent's pure-LFU victim rule wins precisely because durable high counts protect the reusable working set, and aging those counts removes that protection rather than freeing capacity for newly hot keys. Decaying survivor counters is a net loss here — the once-hot-key immortality the hypothesis targeted was a feature, not a failure mode, for this trace.
