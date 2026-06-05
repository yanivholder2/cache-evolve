# Variant v001 Insight

## Helped

- hit-rate: Shrinking the FIFO admission queue from `kin = capacity / 4` to `kin = capacity / 20` (~25% to ~5%) hands the proven-reuse main list Am ~95% of capacity, so more reused keys stay resident and the workload-a hit-rate rose above the parent.

## Hurt

- (none measured): the only evaluated dimension moved favorably; the smaller A1in did not choke admission enough to stall or drop hit-rate.

## Takeaway

For 2Q on this trace, geometry alone matters: matching the 95/5 SLRU incumbent's probationary split — a thin ~5% admission floor over a fat reuse list — closes most of the gap to the parent's deficit, but the FIFO-with-ghost-promotion structure still falls short of the incumbent's 0.7313, so geometry is necessary but not sufficient.
