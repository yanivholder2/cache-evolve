# Variant v003 Insight

## Helped

- per-op-time (get p99): Get p99 fell back to well below the parent's spike. This is the result the hypothesis predicted. `get` and `on_hit` are unchanged, so the only plausible cause is the new eviction order: `map.remove(&old_key)` now runs before `map.insert(key, victim)`. That changes how hashbrown's control bytes and `growth_left` evolve under churn, which supports the parent insight's suspicion that the parent's get-p99 rise came from the insert-before-remove order and was not noise.

## Hurt

- per-op-time (get p50, the summary): Get p50 rose sharply even though no instruction on the get path changed. The bucket count is unchanged too, because `with_capacity(capacity)` and `with_capacity(capacity + 1)` round to the same power of two at the traced sizes. So the cause is either run-to-run noise or a side effect of the new table state on the common hit probe. Either way, the change traded the summary metric for the tail.
- per-op-time (put p50 and p99): Both rose, as the hypothesis expected. Splitting `v.insert` into `v.into_key()` plus a separate `map.remove` and `map.insert` adds a third SipHash and probe to every evicting put. That gives back the single-lookup saving that v002 had won.

## Takeaway

Remove-before-insert on the stock hashbrown map does cure the tail. However, it costs a full extra hash on every evicting put, and get p50 moved the wrong way while get p99 improved. A tail-only fix is therefore a net loss under a p50 summary. The reuse mechanism is also weaker than the hypothesis assumed. The victim's freed bucket lies in the victim's probe group, not the new key's group, so hashbrown rarely reuses it directly. The tail gain more likely comes from fewer DELETED markers and `growth_left` exhaustion (in-place rehash), not from slot reuse. To keep the tail benefit without the extra put hash, a follow-up should remove the victim via a hash-preserving path, such as stored hashes or `raw_entry`/`HashTable`. It should also re-measure get p50 to rule out noise.
