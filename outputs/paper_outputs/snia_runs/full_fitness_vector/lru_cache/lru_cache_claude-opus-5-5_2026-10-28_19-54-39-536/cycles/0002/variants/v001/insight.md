# Variant v001 Insight

## Helped

- metadata-size (`LruCache<u64,u64>`): `FxBuildHasher` is a zero-sized type. It replaces `RandomState`, which carries two `u64` SipHash keys, so the cache struct shrank. The headline `Node` size did not change, since the node layout was deliberately left alone.
- per-op-time (put p99): The eviction path does a `get`, a `remove` and an `insert` on the map, and now none of them runs SipHash setup or finalisation. That may have trimmed the put tail a little, but the CLOCK sweep still sets that tail, so treat this as weak evidence.
- hit-rate: Unchanged, as the hypothesis predicted. Swapping the hasher only changes where keys sit in the map, not which slot the hand evicts, so the CLOCK victim order is the same as v002's.

## Hurt

- per-op-time (get p50/p99): The single-multiply hash did not make the lookup faster. Get got slower at both median and tail, which contradicts the claim that SipHash is most of a hit's cost. At the large trace sizes the probe is likely limited by cache misses into the table, not by hashing. A one-round FxHash with `rotate_left(26)` also mixes structured MSR block-offset keys less evenly than SipHash. The likely result is more 7-bit tag collisions and longer probe groups, which more than cancels the cheaper hash. This is a guess: the eval doesn't show probe lengths.
- per-op-time (put p50): A typical put does two or three map operations, so it took the same lookup regression as get. The expected drop in put latency reversed into a rise.

## Takeaway

Replacing SipHash with a hand-rolled FxHash did not speed up `HashMap<u64, usize>` lookups. The idea that "hashing is the remaining cost of a hit" is now unsupported for this workload. The probe's memory access, and the distribution quality of a weak hash on structured keys, matter more than how many cycles the hash takes. If get latency is still the target, the next attempt should make the table smaller or more cache-friendly, for example a `u32` value or a denser index. Another option is a hasher that mixes better, such as one with an xor-shift finaliser. It should not be another cheaper hash function.
