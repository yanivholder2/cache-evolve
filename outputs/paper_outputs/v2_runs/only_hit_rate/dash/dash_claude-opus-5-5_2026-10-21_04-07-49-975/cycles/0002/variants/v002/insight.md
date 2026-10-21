# Variant v002 Insight

## Helped

- hit-rate: Raising `PROMOTE_THRESHOLD` from 2 to 3 means a fresh key must get three stash hits before it enters its target bucket. Keys with medium reuse (two references) now get served in place from the larger stash (24 buckets vs 8) and no longer push out an LRU resident of a full target bucket. The `PROMOTE_THRESHOLD - 1` pre-credit on both demotion paths now works out to 2, so ex-residents still return after one hit. The stricter filter therefore lands only on new keys and does not slow down keys that already proved themselves.

## Hurt

## Takeaway

On this LIRS trace the stash→target admission edge had not reached its optimum at two hits. Holding back two-hit keys as well still improves hit-rate, though the gain is smaller than the one from moving 1→2. That points to diminishing returns, so a threshold of 4 is unlikely to pay off much. The gain depends on the pre-credit being written relative to the threshold, so returning residents keep a one-hit bar while new keys face a higher one. Like v003, only hit-rate was measured. The hypothesis's claim of fewer remove/insert cascades on gets (per-op-time) is still untested.
