# Variant v002 Insight

## Helped

- metadata-size: `FnvBuildHasher` is a zero-sized type, so replacing `RandomState` (which stores two `u64` seed keys per map) across all three hash collections eliminated ~48 bytes of per-instance hasher state from the `S3Fifo` struct.

## Hurt

- per-op-time: Substituting FNV-1a for SipHash increased get p50 and put p50/p99 latency rather than reducing it; the hypothesis that one-xor-one-multiply hashing would be cheaper than SipHash's permutation was not confirmed on the measured workload, suggesting SipHash's `u64`-optimized path is already competitive at this key size and cache capacity.

## Takeaway

Swapping in a custom FNV-1a hasher for integer keys produced the opposite of the intended effect on hot-path latency: per-op times worsened across the board while hit rate was unaffected. The sole concrete gain was structural — eliminating `RandomState`'s seed storage shrinks the per-instance struct footprint — but this came at the cost of slower hash operations in practice. For small-integer keys at this cache size, SipHash appears no slower than FNV-1a under real benchmark conditions, making pure hasher replacement an unreliable lever for latency reduction here.
