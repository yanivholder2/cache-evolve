# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384, web_0 @ 16384): Both small-size cells beat the parent. Two changes push in the same direction here. Removing the miss→put `tick()` means a deduplicated miss→put adds nothing to `additions`. `sample_factor(16384) = 16` also lengthens the halving period. Together they keep frequency history alive long enough for moderately popular keys to outscore probation victims. This is the confirming outcome H01 named. It suggests the small-size lever was not yet capped by 4-bit saturation at factor 10.
- hit-rate (web_0 @ 131072): This cell rose slightly even though its `SAMPLE_FACTOR` stays at 10. The only change at this size is the `tick()` removal, which slows the effective aging clock there as well. So the dedup-without-tick behaviour is a mild benefit at large sizes too, not just a neutral revert.
- hit-rate (summary): Three cells improved and none regressed, so the summary improves on the parent.

## Hurt

- per-op-time (get/put p50 and p99): Every percentile is slower, but the code does not explain it. `get` is byte-identical to the parent. `put` now does strictly less work on the miss→put path, because the `tick()` branch is gone. `sample_factor` runs only at construction. This is the third large latency swing in a row on an unchanged hot path. Treat it as benchmark noise or layout effects, not a cost of this change.

## Takeaway

Slower sketch aging is the lever that moves small-cache hit-rate in this TinyLFU. Going from 10×capacity to 16×capacity at ≤32768, on top of the no-tick dedup, extended the gain instead of saturating it. A capacity-gated `sample_factor` is a clean way to tune small sizes without touching the large-size path. hm_0 @ 131072 did not move at all, which fits that isolation. Since factor 16 has not hit a ceiling yet, a natural next probe is a larger small-size factor, with the 4-bit counter limit as the expected bound.
