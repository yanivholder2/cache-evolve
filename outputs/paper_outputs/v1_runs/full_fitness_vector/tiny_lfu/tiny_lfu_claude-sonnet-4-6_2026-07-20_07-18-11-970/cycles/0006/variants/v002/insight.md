# Variant v002 Insight

## Helped

- hit-rate: Applying the stricter `incoming_freq > victim_freq + 1` gate on 1-in-8 evictions blocked enough marginal admissions to produce a small but real hit-rate improvement over the plain `> victim_freq` parent.
- per-op-time: Limiting the strict gate to 1-in-8 evictions reduced residency churn to one-eighth of the rate seen in the prior all-evictions variant, collapsing get p99 and put p99 back from 300 ns to 100–200 ns — confirming that churn frequency, not the threshold value itself, was the latency driver.

## Hurt

- metadata-size: Adding the `evict_counter: u32` field grew the `TinyLfuCache` struct from 152 to 160 bytes; the 8-byte increase is an alignment-padded cost for the new counter field.

## Takeaway

Amortizing the strict admission gate to fire only on every 8th eviction threads the needle between the two prior failure modes: the standard threshold (parent) leaves marginal admissions unchecked, while the strict threshold applied every eviction (cycle 0005 v003) doubled p99 latency through residency churn. At 1-in-8 frequency the gate still improves hit rate while keeping p99 at the same level as a plain comparison, at a cost of just 8 bytes of struct metadata.
