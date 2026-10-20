# Variant v003 Insight

## Helped

- per-op-time: The hypothesis computes one cheap multiplicative hash per access and derives both sketch row indices from it, replacing the two SipHash calls in `record()` and the per-candidate `estimate()` hashes in `put()`. Both `get` and `put` latencies dropped against the parent, which is the cost the parent insight flagged as the price of the admission gate.

## Hurt

- None measured. Hit-rate is effectively unchanged from the parent, and metadata-size is identical, so neither dimension regressed.

## Takeaway

Replacing SipHash with one shared mix for both sketch rows and the victim-estimate change keeps the admission gate's behaviour and removes most of its per-access cost. The hit-rate is flat, which fits a change that should not alter the admission decision, but the single trace at one capacity cannot show the cheaper hash is as accurate in general. The next check is whether the per-op gain and the hit-rate hold on other workloads and capacities.
