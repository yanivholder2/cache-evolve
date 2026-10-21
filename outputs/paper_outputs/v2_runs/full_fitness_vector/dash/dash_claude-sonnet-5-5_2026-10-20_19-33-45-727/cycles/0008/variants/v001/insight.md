# Variant v001 Insight

Note: only Read and Write were usable in this session, so I could not list or diff the overlay source. This insight rests on the hypothesis and the two fitness vectors, not on a line-level diff.

## Helped

- per-op-time: Put latency fell sharply at both p50 and p99. The hypothesis cut the doorkeeper to one 64-byte aligned line with both probes in that line, so each put touches one line and avoids a cache miss. Resets became a counter check plus clearing 8 words, with no popcount pass under the lock.
- per-op-time: Get latency improved slightly. The get path was untouched, so this is probably reduced cache pressure from the smaller filter, or noise.
- hit-rate: Hit-rate rose slightly. The smaller filter, aged at 50% fill, filtered about as well as the parent's large array. Because the aging is more frequent, the filter stays sparse and false positives stay low. The gain is small and may be within noise.

## Hurt

- No dimension regressed. metadata-size stayed the same because the filter lives outside the per-item struct, as the hypothesis required.

## Takeaway

The parent's put-latency cost came from the doorkeeper's memory footprint and its reset method, not from the admission filtering it did. A one-cache-line filter with both probes derived from one hash and a set-bit counter kept the hit-rate and removed most of that cost. When a small admission filter is aged often enough to stay sparse, it can match a much larger one.
