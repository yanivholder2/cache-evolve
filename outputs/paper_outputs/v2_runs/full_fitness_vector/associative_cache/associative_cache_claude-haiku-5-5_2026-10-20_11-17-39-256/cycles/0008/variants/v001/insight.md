# Variant v001 Insight

## Helped

- hit-rate: The doorkeeper on the put-miss path keeps a key out of the victim scan until its second sighting, which is the mechanism H01 targeted; the gain on workload-a is very small and comes from a single run on one trace, so it may not clear run-to-run noise.

## Hurt

- per-op-time: The doorkeeper adds a bit lookup and a branch before the victim scan on every put miss, and put tail latency widened the most, which fits that extra work sitting on the miss path.
- per-op-time: Get tail latency also widened even though the change leaves the get path as it was, so this regression is not explained by the admission rule; it is more likely noise or cache effects from the separately allocated bitset and should be re-measured before being attributed to this change.

## Takeaway

Second-sighting admission barely moves hit-rate on workload-a at this size, so the admission rule alone does not protect the working set much. Its cost lands on the put-miss path, so a follow-up needs a clearly larger hit-rate gain, or a cheaper check, before the doorkeeper is worth keeping.
