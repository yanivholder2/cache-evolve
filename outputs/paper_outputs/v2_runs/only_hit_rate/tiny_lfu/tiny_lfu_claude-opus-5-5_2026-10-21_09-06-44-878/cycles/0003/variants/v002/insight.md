# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Widening `VICTIM_SAMPLE` from 8 to 64 let `select_victim` reach deeper into the LRU list, into recently admitted entries. Because the sketch never ages, those newer keys have small lifetime counts while stale once-hot keys near the head keep inflated estimates. So the wider window tends to pick a young key that is still being reused and keep the stale one. The lower `victim_freq` also loosens the `incoming_freq > victim_freq` contest, so more marginal newcomers get admitted, which adds churn. The net effect is a small loss against v001.

## Takeaway

A wider victim sample does not find "genuinely cold" victims on workload-a. With a never-decaying count-min sketch, the lowest estimate in the window marks the youngest resident, not the coldest one. So searching deeper makes eviction lean toward recently admitted keys. The `best_freq <= 1` early exit does not change which victim is picked: residents always have an estimate of at least 1, and strict `<` already keeps the first minimum. It only cuts scan cost, so the regression comes from the width itself. Together with v001's marginal gain, this answers the hypothesis's open question: sample width is not the limiting factor, and widening it backfires. Sketch aging (periodic halving or reset), which would make estimates reflect recent use, should come before any further work on victim sampling.
