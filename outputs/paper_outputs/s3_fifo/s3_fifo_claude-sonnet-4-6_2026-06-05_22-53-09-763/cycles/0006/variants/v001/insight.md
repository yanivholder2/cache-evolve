# Variant v001 Insight

## Helped

- hit-rate: Skipping the push_front reprieve for freq=1 items freed main-queue head positions slightly faster, producing a marginal improvement that confirms freq=1 items in this workload do not carry enough forward access probability to justify the extra traversal.

## Hurt

## Takeaway

Tightening the second-chance gate from `freq > 0` to `freq > 1` moved hit-rate in the predicted direction but by an almost negligible amount, revealing that the freq=1 population on this workload is either too small or too cold for its early eviction to meaningfully reshape the resident set. The hypothesis is directionally correct — those items are not worth reprieving — but the gain is so thin that freq=1 linger time is not a high-leverage knob for this access pattern.
