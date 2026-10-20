# Variant v001 Insight

## Helped

- hit-rate: Going back from halving to a linear `*e -= FREQ_ONE` restores the full 7-lap retention schedule of the 3-bit counter. Keys at freq 4–7 again survive in proportion to how often they were hit, where halving had collapsed them all to 3 laps. Hit-rate rises above the parent as a result, and above the ≈0.730 level the hypothesis named as refutation. It still falls short of the hypothesis's ≥0.733 target. Most of the laps came back, but not quite all of the 0008 v003 gain. Possible causes are the budget's forced releases of freq>0 entries, or differences between this variant and 0008 v003 that are not visible here.
- per-op-time (`put`): Linear decrement makes each saturated key take more laps than halving did. Even so, the `put` median and tail did not get worse and come in slightly under the parent. So `MAIN_ROTATE_BUDGET = 32` does keep the long-drain tail out of `put`: each `evict_main` stops after at most 33 pops, however many saturated keys pile up at the back of main. The hypothesis set "`put` p99 stays at v003's level" as its refutation condition, and that did not happen.

## Hurt

- per-op-time (`get`, the summary): The `get` median and p99 both rose noticeably against the parent. Yet `get` is byte-for-byte the same code: one probe, a compare against `MAX_FREQ = 7`, and an add. The change only touched `evict_main`. The regression therefore can't come from a costlier hit path. Two indirect explanations are possible. Longer linear retention may change which keys are resident, and so the hashbrown probe or cache-line behaviour on the hit path. Or it is run-to-run noise. The cycle 0009 insight flagged the same caution when `get` moved the other way. Because this is the summary dimension, re-measure side by side with the parent before treating the budget as a latency loss.
- metadata-size: No change. `MAIN_ROTATE_BUDGET` is a constant, and `rotations` is a stack local. `forced_evictions` exists only under `#[cfg(test)]`, so neither `Entry` nor `S3Fifo` gains any bytes.

## Takeaway

Cycle 0009 split retention depth from drain cost, and this variant confirms the split. Linear decrement gives back hit-rate, and a fixed per-call rotation budget caps the `put` cost that linear laps used to bring with them. The budget gave up a little of the hoped-for hit-rate, which still fell short of the target. The next step is to read `forced_evictions` from `h01_split_report` and check whether the budget binds often. If it does, a larger budget (e.g. 64) should be tried. Before that, the `get` regression needs re-measuring, because no line on the hit path changed.
