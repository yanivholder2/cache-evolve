# Variant v001 Insight

## Helped

- metadata-size: Replacing the two-HashMap layout with a single `resident_map` eliminated one HashMap header from the S3Fifo struct itself, shrinking the struct's own memory footprint as a secondary effect of the merge — even though the summary metric (which tracks Entry size) did not move.

## Hurt

(none)

## Takeaway

Merging `data` and `freq_map` into a single `resident_map<K,(V,u8)>` did not reduce get p50 latency despite halving the probe count on every hit. The Zipfian hot set is small enough that both maps' hot buckets likely reside in the same CPU cache level, so the hypothesised cross-region cache-line penalty either does not materialise or is smaller than the benchmark timer's resolution floor. The refactor simplifies the struct layout and reduces its size, but the primary latency hypothesis was not confirmed at this workload scale.
