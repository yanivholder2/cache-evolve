# Variant v001 Insight

## Helped

- per-op-time (`put`, tail): `evict_main` now halves the counter, so a main entry survives at most 3 laps. That is the same ceiling the parent had with `MAX_FREQ = 3`. A key at freq 3 now drains in 2 laps (3→1→0) instead of 3 (3→2→1→0), so a CLOCK scan pushes back fewer entries before it finds a victim. This shows up in the `put` tail, where the long drains used to land. The `put` median is flat, which fits: most `put`s do not hit a long scan.
- per-op-time (`get`, the summary): `get` is the same code as in the parent, except it compares against `MAX_FREQ = 7` and shifts by 29 instead of 30, so it does no extra work per hit. The lower `get` numbers, the p99 in particular, cannot come from a cheaper hit path. They may be an indirect effect of fewer main-queue rotations on the same benchmark, or run-to-run drift. Re-measure side by side with the parent before crediting the change for them. The hypothesis's latency goal was met either way: the 3-bit cap did not bring back 0008 v003's drain cost.

## Hurt

- hit-rate: Halving takes away exactly the retention depth that 0008 v003's gain came from. Keys at freq 4–7 all get the same 3 laps as the parent's saturated keys. Keys at freq 3 actually lose a lap compared with the parent. So the wider counter tells hot keys apart when they are counted, but `evict_main` throws that ranking away after one halving. Only keys that are hit again between laps keep any of the extra resolution. The net effect is slightly shallower main-queue retention than the parent, and hit-rate ended up a little below the parent rather than at or above it.
- metadata-size: No change. The counter still lives in the spare high bits of the `index` value, and the slot id narrows from 30 to 29 bits. That still fits easily at the evaluated capacity.

## Takeaway

On this trace, how high the counter can go matters less than how many laps a hot key survives in main. 0008 v003's hit-rate gain depended on decrement's long linear retention, and replacing it with log2 halving gives up that gain along with the drain cost. The hypothesis set its own test: a hit-rate back at the parent's level would mean v003's gain needed the long linear retention. The result is at or slightly below that level, so the test came out against halving. A future attempt to get v003's hit-rate without its latency should cap the drain work some other way, such as a bounded scan budget per eviction or a decrement on a smaller sample. It should not make the retention schedule shorter.
