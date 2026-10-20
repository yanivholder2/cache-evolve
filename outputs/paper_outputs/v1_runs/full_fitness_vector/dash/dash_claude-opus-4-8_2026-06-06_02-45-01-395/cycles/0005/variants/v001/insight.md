# Variant v001 Insight

## Helped

- hit-rate: The devirtualized `lru_get_from_position`/`lru_eviction_index`/`lru_promotion_index` free functions reproduce `ClassicLRU` bit-for-bit, so the parent's hit-rate carried over unchanged — the latency refactor cost nothing in quality.
- metadata-size: Bypassing the vtable touched only call dispatch, not layout, so the parent's 16-byte `Item` floor held intact.
- per-op-time: Removing the `Box<dyn EvictionPolicy>` indirect call on the get path shaved the put p99 tail down, the one place the vtable elimination showed through.

## Hurt

- per-op-time: get-p50 did not move below the parent's 86ns and the get p99 tail ticked up, because the new per-op `is_classic_lru` branch was added on top of — not in place of — the `target_heat` HashMap probe that actually gates this variant's hot path, so put-p50 also edged up slightly.

## Takeaway

Porting the devirtualized ClassicLRU hot path onto the side-table parent did not stack: get-p50 stayed pinned at the parent's 86ns, proving the `target_heat` HashMap probe — not the vtable dispatch — is what bounds this variant's latency. The two cycle-0004 Pareto wins are substitutive here, not additive; you cannot collect the devirtualization speedup while the side-table hash sits on the same hot path.
