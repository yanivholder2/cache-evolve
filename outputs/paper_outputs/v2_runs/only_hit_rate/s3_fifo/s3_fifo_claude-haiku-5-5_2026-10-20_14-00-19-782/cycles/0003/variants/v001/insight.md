# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The hypothesis adds a resident-key `HashSet` so that misses in `get()` and `put()` skip the queue scans, which should lower per-op-time while leaving eviction order unchanged. This variant's fitness vector reports only hit-rate, and hit-rate held flat against the parent, which is consistent with an unchanged eviction order. It gives no evidence on per-op-time or metadata-size, so the cost claim is untested. Those two dimensions need to be measured in a rerun before the resident-set change can be judged.
