# Variant v002 Insight

## Helped

- hit-rate: Requiring the candidate's sketch estimate to beat the victim's by more than 1 (with a lower bar of `cand >= 1` when the victim estimate is 0) turns away more one-hit scan keys. Fewer of them displace resident entries, and the improvement over the parent is small.

## Hurt

- hit-rate: No dimension clearly regressed. The stricter gate can also reject genuine second-touch keys until retries have built up their sketch count. Any such cost seems to be outweighed by the scan-pollution savings on this trace.

## Takeaway

Tightening the TinyLFU-style admission margin to `cand > victim + 1` gave only a marginal gain on workload-a. That suggests the parent's admission gate was already filtering most scan pollution, so threshold tuning has little left to give. Counting rejected misses in the sketch keeps the stricter rule from locking out recurring keys. Bigger gains would likely have to come from somewhere other than the admit margin. This was evaluated on a single trace and cache size, so the gain may be within noise.
