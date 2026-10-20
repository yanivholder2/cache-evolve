# Variant v003 Insight

## Helped

- hit-rate: Splitting the single LRU list into an SLRU (protected_cap = floor(0.8·capacity), eviction restricted to the probationary head) shields promoted-on-hit keys from the admission churn, so a few proven-hot residents that v001 would have let drift to the LRU victim end now survive — nudging hit-rate up by a hair.

## Hurt

- (none observed): hit-rate is the only dimension in the vector and it did not regress; the added `protected` flag, second link-pair, and promote/demote bookkeeping carry no measured cost here.

## Takeaway

On workload-a at size 2048, recency-protection via SLRU is directionally right but nearly inert: the same frequency sketch and `incoming_freq > victim_freq` gate already keep the hot working set resident, so reserving 80% of the main space as a protected tier only rescues the handful of keys that sat near the LRU head between hits. The protection axis is real but, against this admission gate, almost fully redundant — most of what SLRU would save was already being saved.
