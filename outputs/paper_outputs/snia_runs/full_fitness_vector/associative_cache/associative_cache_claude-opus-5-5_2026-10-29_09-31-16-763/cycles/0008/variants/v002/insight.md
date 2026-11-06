# Variant v002 Insight

## Helped

- hit-rate: The width-keyed `bip_mask` did what it was designed to do on wide buckets. With `max_size > BIP_WIDE_BUCKET`, admitting to MRU every 64th miss instead of every 32nd lets web_0@131072 hold more of its scan-resistant working set, so that cell rose into the predicted band. Both 16384 cells are bit-identical to the parent. This confirms that the narrow branch (`bip_mask = 31`, same increment-then-test order) is behaviourally the same as v003's `% BIP_MRU_PERIOD`. The summary beats the parent, but it misses the hypothesis's target of 0.541125.
- metadata-size: Held flat as required. `bip_mask` is computed from the already-loaded `max_size` instead of being stored, so `Item` and the bucket stride are unchanged.

## Hurt

- hit-rate: hm_0@131072 gave back part of its recency benefit, because rarer MRU admission also delays promotion for hm_0's freshly inserted, soon-reused keys. The loss landed inside the predicted band. Moving from 1/32 to 1/64 moved the web_0 gain and the hm_0 loss together, in proportion, so 1/64 is a point on a roughly linear tradeoff, not a sweet spot.
- per-op-time: Get and put both slowed at p50 and p99 relative to the parent. The only code change is on the put-miss path: a `max_size` compare/select plus `&` instead of `%`, which should cost no more than the parent's `%`. The get path is untouched, so the get regression cannot be caused by this diff. As the parent insight warned, this looks like run-to-run variance or a layout/inlining shift, not a real cost of width-keyed BIP.

## Takeaway

Halving the wide-bucket BIP MRU-admission rate traded web_0 gain against hm_0 loss in roughly fixed proportion, and the net summary did not clear the hypothesis's target. By the hypothesis's own stop rule, the wide-bucket BIP period tradeoff is effectively flat, so further tuning of this knob is not worth it. The width-keyed mask is a safe, zero-metadata way to split policy by bucket size: the narrow cells stayed exact. The latency movement here should not be attributed to it without same-session repeated runs.
