# Variant v001 Insight

## Helped

- **hit-rate**: Adding vacancy-gated probing re-entry (move item from probing back to target MRU end when `lfu_counter >= 2` and target has space) produced a small additional gain on top of the parent's LFU-minimum eviction and vacancy-gated target promotion, confirming the mechanisms are orthogonal — the re-entry gate fires on the probing-hit branch, entirely separate from the eviction-scan path.

## Hurt

_(no dimensions regressed)_

## Takeaway

Stacking probing re-entry onto the cycle/0005/v001 base delivered the expected additive hit-rate lift with zero latency penalty. The re-entry logic cycles hot probing items back through target at the MRU end, letting them immediately re-earn their probing slot on the next access; on a Zipfian trace this keeps the small working set of very-hot keys circulating through both tiers rather than stalling in probing indefinitely. The gain is modest because the gate requires both a second probing hit and a vacant target slot — it fires reliably for the hottest items but is naturally silenced when capacity is tight.
