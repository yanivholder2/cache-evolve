# Variant v001 Insight

## Helped

- hit-rate: Splitting the cache into a 20% probation list and an 80% protected list meant one-time reads only cycled through probation. Re-referenced blocks in the MSR web_0 and hm_0 traces stayed in the protected list, so hit-rate went up at every trace and size measured.
- hit-rate (scaling): The gain was clearly larger at the 131072 size than at 16384, most of all on web_0. With more room, the protected list can hold a bigger share of the reused working set. At the small size, the 20% probation list is short, so entries seen twice but far apart are often evicted before their second access can promote them.

## Hurt

(none observed. v001's fitness vector has only `hit-rate`. `metadata-size` and `per-op-time` were not measured, so this variant gives no evidence either way on them.)

## Takeaway

On MSR block traces, an SLRU split resists scan pollution the way the hypothesis predicted. The benefit grows with capacity, because a short probation list cuts off promotion for blocks whose reuse distance is longer than it. Two things remain untested and should be checked next cycle: the `u32` index change from `Option<usize>` and the extra segment-move work in `touch` (a promotion can also trigger a demotion). Code inspection suggests `Node<u64,u64>` became smaller than the baseline's 48 bytes, and `get` may be slightly slower.
