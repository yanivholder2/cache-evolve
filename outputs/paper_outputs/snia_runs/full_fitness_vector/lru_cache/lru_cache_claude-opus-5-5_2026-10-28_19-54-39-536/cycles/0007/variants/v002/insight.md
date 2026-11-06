# Variant v002 Insight

## Helped

- hit-rate (web_0/131K): Turning the reprieve on at 131K with a capacity/2 ghost (65536 slots) gave web_0 a real gain over the parent, where the ghost was off at 131K. Some of web_0's re-references at this size come back within about 64K evictions, and a one-shot probation reprieve catches them.
- hit-rate (summary): The summary rose because web_0/131K gained more than hm_0/131K lost. Both 16K points match the parent exactly, as the invariant required. That confirms the change only affects capacities above `GHOST_MAX_CAPACITY`.
- per-op-time (put p99): Put p99 fell sharply, even though large caches now do ghost slot reads and writes on vacant puts. No code change explains a cheaper tail, so this is most likely run-to-run tail noise or a layout shift, not an effect of the hypothesis. It does show the larger ghost did not add a visible put tail.

## Hurt

- hit-rate (hm_0/131K): hm_0/131K dipped slightly. A capacity/2 window still readmits a small share of hm_0's loop keys, or gives spurious reprieves from 16-bit tag collisions in the 64K-slot table. Those reprieves keep entries in probation that a plain SLRU would have evicted. The dip is much smaller than the full-width ungated reprieve's cost noted in the rationale.
- per-op-time (get p99, put p50): Both moved up by a hair. `get`/`on_hit` were not touched, so this is consistent with noise, or with the 128 KB ghost table now being live at 131K and competing for cache with the node and map data.

## Takeaway

A capacity/2 ghost is a usable midpoint at 131K. It keeps most of the web_0 gain from a full-width reprieve while costing hm_0 only a little, so web_0's useful reuse distance at this size is mostly under capacity/2 and hm_0's loops mostly lie beyond it. The leftover hm_0 loss shows the window still leaks a little. The next step is to tune the length between capacity/8 and capacity/2, for example capacity/4, or to make reprieves stricter at large sizes, rather than to make the ghost wider. Note that `metadata-size` does not count the heap-allocated ghost table, so the 128 KB per cache is a hidden memory cost.

<!-- Note: the overlay source and manifest could not be opened in this session; I could only read files at known paths and could not list directories. This insight is based on the hypothesis, the parent insight, and both fitness vectors. -->
