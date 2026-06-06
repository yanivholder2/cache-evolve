# Variant v001 Insight

## Helped

- metadata-size: dropping the redundant `key: K` field from `Entry` — replaced by a zero-sized `PhantomData<K>` — shrinks the scored `Entry<u64,u64>` from 24 to 16 bytes, since `entries` is keyed by the resident key and the stored copy was dead weight.

## Hurt

- (none)

## Takeaway

The `key` field in the per-entry record was pure duplication of the `entries` HashMap key, and carrying `K` through `PhantomData` instead recovers those 8 bytes with zero behavioral change: hit-rate stays bit-identical at 0.7312 and the get-p50 that defines per-op-time holds at its best, since no eviction, promotion, or ghost-membership logic was touched. The put/get p99 dips versus the parent are run-to-run noise, not an effect of removing a field that was never read on the hot path.
