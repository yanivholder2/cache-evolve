# Variant v003 Insight

## Helped

- metadata-size: Dropping the standalone `flags: u8` and storing PROTECTED and REFERENCED in the top two bits of `prev` removes the padding that the trailing byte forced onto `Node<u64,u64>`, which is the whole size gain the hypothesis targeted; `LruCache` is unchanged because the map and header fields were not touched.

## Hurt

- per-op-time: Every index read and write now goes through a mask to strip the flag bits from `prev`, so the `get` median and tail and the `put` median and tail all rose relative to the parent. The hypothesis expected `get` to stay mask-free, so the cost is likely in the shared index helpers used by `unlink` and `push_back` on the hit and insert paths.

## Takeaway

Packing the flags into `prev` is policy-neutral, as the unchanged hit-rate confirms, and it buys a real 8-byte node reduction. The trade is paid on the per-op path: the size win holds only if the masking cost is small, and here it was not, so the variant is a size-for-latency trade rather than a free improvement. Any follow-up should keep the flag-free `prev` read on the `get` hot path or drop the packing in favor of the parent's byte-sized flags.
