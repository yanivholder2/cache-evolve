# Variant v003 Insight

## Helped

- hit-rate: Inserting most new keys at the LRU tail, and only every 32nd insert at the MRU head, keeps one-touch keys from displacing re-referenced entries on workload-a, so the victim is more often a key that was never reused.

## Hurt

## Takeaway

Placing new nodes at the LRU end by default is a cheap scan-resistance mechanism: it adds no second list or ghost set, and it only pays off because the hit path still promotes re-referenced nodes to MRU. The gain is measured on hit-rate alone; metadata-size and per-op-time were not recorded for this variant, and the source diff was not available to confirm the code matches the hypothesis.
