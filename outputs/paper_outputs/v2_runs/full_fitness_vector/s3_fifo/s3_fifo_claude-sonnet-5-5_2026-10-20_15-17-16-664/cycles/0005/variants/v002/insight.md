# Variant v002 Insight

## Helped

- per-op-time: get p50 and p99 dropped slightly, but this change only touches main-queue eviction, which get never runs. That makes the drop most likely run-to-run noise, not an effect of the edit.

## Hurt

- per-op-time: put p50 and p99 got slightly worse. This is the path the hypothesis targeted (pop_front/push_back CLOCK-style eviction). The movement is within noise, so the change did not deliver the intended put-tail reduction.

## Takeaway

Rewriting main-queue eviction as a pop-head, decrement, push-tail loop reproduced the parent's eviction decisions exactly. Hit rate and metadata-size did not move, so the equivalence held and no state was added. The parent's eviction loop was evidently already doing the same amortized work, because put latency did not improve. The put cost is probably dominated by something else, such as ghost-queue maintenance or hash-map operations, so further work on the eviction loop is unlikely to pay off.
