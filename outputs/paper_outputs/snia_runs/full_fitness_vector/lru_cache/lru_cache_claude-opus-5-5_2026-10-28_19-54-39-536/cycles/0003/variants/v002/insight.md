# Variant v002 Insight

## Helped

- hit-rate: The hypothesis predicted no change, and every trace and size matches the parent exactly. So the specialised `move_to_prot_mru` / `move_to_prob_mru` / `promote` / `demote_prot_head` routines and the MRU early-exit keep v003's eviction order exactly. The new `specialised_hit_path_matches_generic_reference` test checks the same thing directly against the retained `on_hit_generic`.
- metadata-size: Node layout and flag encoding were left alone as instructed. Promotion writes `PROT_BIT | tail` and demotion writes a plain `tail`, which sets or clears the segment flag in the same store as the prev index. So the 24-byte node and the cache header are unchanged.
- per-op-time (get p99, put p50): Both moved only slightly in the right direction. That fits the change: removing the `if protected` head/tail selection, the length updates on same-segment moves and the dead `set_prev(NIL)`/`set_next(NIL)` stores trims a few ALU ops and stores from the relink. It does not change the cost profile of either operation.

## Hurt

- per-op-time (get p50, the summary metric): The typical get got slower, which contradicts the central claim of the hypothesis. A get is dominated by the `HashMap` lookup and the dependent arena loads. The removed work was mostly predictable branches and stores to cache lines the relink already touches. Meanwhile every hit now pays an extra `idx != prot_tail` compare. Its outcome depends on the trace, so it is hard to predict, and it saves work only when the hit is already MRU, which is rare at 16K–128K capacities.
- per-op-time (put p99): The put tail got worse. The likely cause is code layout. The fresh-key path moved into a separate `#[inline] insert_new` and still uses the generic `unlink` + `push_back`, while `on_hit` grew four `#[inline(always)]` routines that are inlined into both `get` and `put`. The larger hot functions put more pressure on the i-cache and branch predictor on miss-heavy puts, and the specialisation saves nothing on those puts.

## Takeaway

Making the SLRU hit-path relinks segment-specific is safe, since policy and layout are exactly preserved, but it does not reduce per-op-time. The relink was never the bottleneck behind v003's put/get cost, so removing branches and stores from it is lost in measurement noise, and the MRU early-exit adds a branch on every hit that rarely pays off. Further per-op-time work should target the hash lookup and memory access pattern (e.g. map/arena locality or the key clone on insert), not the linked-list relinks.
