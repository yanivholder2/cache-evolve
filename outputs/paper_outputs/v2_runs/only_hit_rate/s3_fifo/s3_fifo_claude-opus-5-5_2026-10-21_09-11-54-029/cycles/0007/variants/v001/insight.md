# Variant v001 Insight

## Helped

(No measured dimension moved. The only dimension evaluated was hit-rate, and it stayed exactly at v002's value, which is what H01 required. Only the `BuildHasher` behind `ghost_map` changed. `fingerprint()`, the ring, `ghost_head`, the ghost-hit path in `put`, and `evict_main`/`evict_small` are unchanged. A hasher changes which bucket a fingerprint lands in, never whether it is present. So every ghost-hit decision, and every queue decision after it, is the same as in v002. `matches_ring_reference_on_pseudo_random_streams` still asserts that the ring layout, head, and live count match the reference exactly.)

## Hurt

(No measured dimension regressed. Per-op-time, the target H01 actually claimed, is not measured by this eval.)

## Takeaway

The cost saving goes beyond what H01 described. The identity hasher removes the second SipHash on the miss-path `ghost_map.remove(&fp)`. It also removes up to three more SipHash passes inside every `ghost_insert`: the `get(&old)` staleness check, the `remove(&old)`, and the `insert(fp, …)`. A ghost insertion now costs exactly one SipHash, for the key fingerprint, plus plain hashbrown probing. The identical hit-rate shows it is safe to rely on the fingerprint already being well mixed. But the case for v001 over v002 rests entirely on a per-op-time benchmark this eval does not run, so on current evidence the two variants cannot be told apart.
