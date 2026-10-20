# Variant v003 Insight

## Helped

## Hurt

## Takeaway

H03 adds a 64-bucket counting filter in front of the ghost linear scan in `put`, and the code preserves the parent's eviction sequence, so hit-rate is unchanged and the eval cannot show whether skipped scans pay off. The filter's cost is also understated: the counters are `u32` rather than the `u8` the hypothesis specified (about 256 bytes of metadata, not 64), and every ghost push, pop and re-admission now hashes the key, so the eviction path gets slower too. Whether the saved misses outweigh that is still unmeasured, since per-op-time and metadata-size are not in the fitness vector.
