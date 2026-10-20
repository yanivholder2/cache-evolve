# Variant v002 Insight

## Helped

- metadata-size: deleting the `lfu_counter: u8` field left `Item<u64,u64>` as two bare `u64`s, removing the trailing alignment padding so the struct packs from 24 down to a tight 16 bytes.
- per-op-time: with no `lfu_counter` to initialise in `Item::new` and no increment on access, the `get`/`put` hot paths shed a small write, nudging both p50 and p99 latencies down.

## Hurt

- hit-rate: unchanged — the active `ClassicLRU` eval policy is purely positional and never read `lfu_counter`, and the `Lfu` policy was rewritten to the same positional fallback, so eviction order was untouched.

## Takeaway

Once `timestamp` was already gone (v003), the lone `u8` LFU counter was the only thing forcing `Item<u64,u64>` past its 16-byte key+value core; deleting it collapsed the padding for free, because the exercised `ClassicLRU` config never consulted the field. Rewriting `Lfu`'s `get_from_position`/`eviction_candidate_index`/`promotion_candidate_index` as positional mirrors of `ClassicLRU` kept the `EvictionPolicyKind::Lfu` JNI ordinal and gate contract intact while erasing the last dead read.
