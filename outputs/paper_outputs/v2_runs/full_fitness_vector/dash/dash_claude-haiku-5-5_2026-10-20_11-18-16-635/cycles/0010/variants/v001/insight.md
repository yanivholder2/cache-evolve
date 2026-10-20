# Variant v001 Insight

## Helped

- metadata-size: Unchanged from the parent, as the hypothesis required; the put-path increment reuses the existing counter field and does not widen the packed record.

## Hurt

- per-op-time: Put latency rose clearly relative to the parent, which is the expected cost of the added saturating increment on every overwrite of a resident key, since that write now also touches the counter.
- per-op-time: Get latency also moved up even though the hypothesis left the get path and promotion logic untouched, so this is not attributable to the change and is more likely run-to-run noise or a code-placement effect.

## Takeaway

Counting overwrites of resident keys as reuse did not move hit-rate on workload-a: the Zipfian hot set appears to already accumulate its counts through get hits, so extra increments on the put path add cost without changing which victims are chosen. The hypothesis's premise that overwrite traffic is a meaningful source of retention signal is not supported by this single trace. The overlay source could not be inspected in this session, so the exact put-path probe and whether the increment fires on the intended slot remain unverified.
