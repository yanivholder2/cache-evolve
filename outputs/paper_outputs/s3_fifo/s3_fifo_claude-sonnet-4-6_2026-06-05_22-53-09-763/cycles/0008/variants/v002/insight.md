# Variant v002 Insight

## Helped

- hit-rate: Shrinking the small queue from ~10% (`capacity / 10`, 204 slots at 2048) to ~7% (`capacity / 14`, 146 slots) transferred roughly 58 additional resident slots to main, allowing the Zipfian hot set to displace one-time-access items more quickly and improving the overall resident quality.

## Hurt

## Takeaway

The gain from reducing small-queue size is monotone past the 10% point: at capacity 2048 on a Zipfian workload, a ~7% small queue still filters one-time-access traffic effectively while the enlarged main queue retains more of the hot set. This result directly challenges the prior characterization of 10% as a "near-optimal floor" — the benefit continues, at least to ~7%, even with the eager-promotion logic already in place from H01.
