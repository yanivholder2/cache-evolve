# Variant v002 Insight

## Helped

## Hurt

## Takeaway

On workload-a at capacity 2048, the sketch can drop from 32-bit counters at 16× width to 16-bit conservative-update counters at 8× width without changing hit-rate. The result is identical to v001, not just within noise. The extra collisions at half width plus CU's tighter estimates did not flip any strict-`>` admission contest or `select_victim` choice. u16 saturation never applied, because without aging the lifetime counts stay well below 65535. This supports the c4 reading that CU and extra width are substitutes for fixing overcount. It also shows that the parent's 512 KiB sketch was over-provisioned for this trace. The 4× metadata cut is real by construction (`heap_bytes` now uses `size_of::<u16>()`, and the inline test pins it at 4 × 16384 × 2), but the fitness vector still reports only hit-rate, so the metadata-size gain remains unscored. Before going narrower (4× width, or 8-bit/4-bit counters, which would need aging to avoid saturation), the metadata-size bin has to be wired into evaluation so the trade can actually be measured.
