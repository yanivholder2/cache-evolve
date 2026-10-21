# Variant v001 Insight

## Helped

- (none): No dimension improved. The only reported dimension, hit-rate, moved slightly the wrong way relative to the parent.

## Hurt

- hit-rate: Halving all sketch counters every 10x-capacity samples decays frequency history. On workload-a at this cache size, that decay appears to cost more than it gains. Stable hot keys lose some of the frequency margin they had in the window-to-probation duel, and the workload shows little of the shifting popularity that aging is meant to fix. The regression is small.

## Takeaway

Adding sketch aging on top of v002's window and segmented main region did not help on workload-a. The workload evidently has stable frequencies, so stale counts were not the bottleneck and decay only erodes useful history. Aging may need a longer sample period or a trace with real popularity shifts to pay off. Per-op time and metadata size are still not reported, so the cost of the halving pass is unmeasured. I could not inspect the overlay source, so this reading rests on the hypothesis text and the fitness vectors.
