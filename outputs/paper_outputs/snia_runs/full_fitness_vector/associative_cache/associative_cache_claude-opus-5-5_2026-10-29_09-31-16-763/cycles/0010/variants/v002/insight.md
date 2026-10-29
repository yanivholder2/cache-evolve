# Variant v002 Insight

## Helped

- per-op-time (put): Put got much faster at both p50 and p99. On the ClassicLRU path, a put that updates an existing key used to go `put_lip` → `Bucket::get_from_position` → boxed `ClassicLRU::get_from_position` (a vtable call), and it fixed up `tags` separately afterwards. It now calls the inline `lip_promote`. That function moves `items` and `tags` in one visible `remove`/`push` sequence and returns immediately when the hit is already on the back slot, so the compiler can specialise the whole update for `Item<u64,u64>`. The `evict_item` change (`idx = 0` when `victim_is_front`) probably adds little to this. On a full ClassicLRU bucket, `put_lip` already takes the fused evict+insert path before it reaches `evict_item`.
- per-op-time (get): Get p50 and p99 both moved down slightly, in the direction the hypothesis predicted. The hit path now uses `lip_promote` instead of the dynamic call. The gain is much smaller than for put, so on get hits the virtual call was only a small part of the cost.

## Hurt

- (none) hit-rate is the same in every trace/size cell, as intended. `lip_promote` produces exactly the order ClassicLRU produced, and the new `inline_hit_matches_reference_last_slot_heavy` test confirms this against `RefBucket` at sizes 1/2/17/300/1024. metadata-size is unchanged: no field was added and the 80-byte stride test still passes.

## Takeaway

Removing the `Box<dyn EvictionPolicy>` call from the ClassicLRU promote path cost nothing in hit rate. It sped up put far more than get. The put-update path had more hidden work: a trait hop plus a policy move, followed by a separate `tags` fix-up. With the vtable call gone, the next get profile should look at the full-key compare in the tag scan or at bucket indexing, as H02's rationale suggests. For put, the remaining cost is mainly the miss/insert memmoves.
