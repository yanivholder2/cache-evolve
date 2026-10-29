# Variant v001 Insight

## Helped

- hit-rate: All four cells match the parent exactly. This confirms that the fused `items[0..=q].rotate_left(1)` + `mem::replace` gives the same bucket layout as the parent's `remove(0)` + `insert((len-1)/4)` (and `remove(0)` + `push` on BIP ticks), and that `bip_tick` advances at the same point.
- metadata-size: Unchanged, as expected. No fields were added and `Item` layout was not touched.

## Hurt

- per-op-time: Put p50 and p99 got clearly worse when they were meant to drop sharply. Get p50 and p99 also got worse by a similar relative amount, even though the get path (`get_position` / `get_from_position`) is byte-identical to the parent. Because get regressed too, most of the swing is likely a shift in the benchmark environment, not something this change caused. The measurement therefore cannot show the expected memmove saving. One real cost the change does add sits on every full-bucket miss: a dynamic `eviction_candidate_index` call through `Box<dyn EvictionPolicy>`, made before the fast path is taken. A generic slice `rotate_left` may also cost more than a plain `memmove` for one-element shifts. Either could cancel part of the gain from moving ~0.25·W items instead of ~1.75·W.

## Takeaway

Merging evict and insert into one partial rotation is a correct, layout-preserving rewrite: hit rate held exactly. But the predicted put speedup did not show up. Get also slowed although its code did not change, so this run cannot judge per-op-time. The rotation should be re-benchmarked against the parent in the same session before it is kept or dropped. A cleaner version would skip the per-miss dynamic `eviction_candidate_index` check. It could cache "victim is always index 0" in the existing `lip` flag at construction. It could also use `ptr::copy` for the one-slot shift instead of a generic `rotate_left`. Then the fused path costs only the shortened memmove.
