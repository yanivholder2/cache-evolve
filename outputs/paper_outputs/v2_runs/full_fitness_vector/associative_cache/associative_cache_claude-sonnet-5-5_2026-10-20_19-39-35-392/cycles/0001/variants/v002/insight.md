# Variant v002 Insight

## Helped

- per-op-time (put): `rotate_left(1)` on a sub-slice replaces the baseline's `remove(position)` + `push`. This shifts fewer elements when the hit lands in the back half of the bucket. Put latency improved, probably because hit-time bucket reordering is cheaper and leaves the bucket in a friendlier layout.
- metadata-size: unchanged. Partial promotion reorders the existing items and adds no per-item or per-cache fields.

## Hurt

- hit-rate: Moving a hit item only halfway to the MRU end makes the bucket order a weaker recency signal. A recently hit item can stay near the eviction end (index 0) and be evicted before colder items. On workload-a, any scan resistance gained did not outweigh this loss of recency precision.
- per-op-time (get): Get got slightly slower. The extra index arithmetic and the sub-slice rotation do not beat the baseline's simple remove-and-push path for a bucket this small.

## Takeaway

Partial promotion in a small, low-associativity bucket loses more recency information than it gains in one-hit-wonder resistance on workload-a. With only a few slots per bucket, a hit item that is not fully promoted becomes the next eviction victim. Any scan-resistance idea probably needs a signal separate from position, such as a frequency or reference bit, rather than a weaker move.
