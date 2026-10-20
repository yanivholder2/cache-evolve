## Recurring wins

- Choosing probation victims by sketch estimate over a small tail window has been the most consistent gain: two candidates (C0008), then three (C0009), then four (C0010). Each step helped less than the last, so returns are tapering.
- Reducing estimator collision error is the main admission lever: sketch width (C0004), an extra hash row (C0007), and conservative update (C0005) each helped. Their gains appear to overlap rather than stack independently.
- Protected-segment sizing near the incumbent ratio is the main-region setting that holds up. A modest share increase helped once (C0006); pushing it further reversed the gain (C0007).

## Recurring failures

- Aging or halving the sketch on a fixed or table-wide trigger keeps losing hit-rate (C0001, C0005, C0009). It erases the frequency history the admission comparison depends on.
- Gating admission, promotion, or sketch updates on a filter keeps losing hit-rate (C0002 window ahead of LRU, C0006 doorkeeper, C0007 sketch-gated promotion). Frequency counts lag recency, and withheld increments are not recovered by estimate-time correction.
- Removing estimator depth or read-side increments hurts, since both reduce the popularity signal the admission rule uses (C0005, C0010).
- Layout-only and storage-narrowing changes hold hit-rate by construction (C0002, C0003, C0009) but their claimed gains (footprint, per-op time) are not measured, so they cannot be credited.
- Tie-breaking changes that admit equal-estimate newcomers against probation victims hurt (C0008). Queue position was protecting older cold entries the sketch cannot separate.
- Ghost-history frequency bonuses barely move hit-rate and carry unmeasured metadata and per-op cost (C0008).

## Open questions

- Do the victim-scan, estimator, and protected-share gains hold on other workloads and cache sizes? Every result so far is one trace at one size.
- Does the victim scan plateau past four candidates, and does its gain depend on the earlier width and conservative-update changes?
- Is the Winners designation for Cycle 0007 correct? v001 reported a hit-rate loss and only v002 showed a gain; this needs resolving before building on either.
- What metadata and timing signals would let layout, depth, and conservative-update costs be weighed against their gains?
- Would per-row or per-key aging with a threshold that actually fires help bound counter growth without losing admission history?

## Per-policy notes

- Admission (frequency sketch): width and depth help, and conservative update adds a small gain on top. Fixed or global halving hurts. Filtering first-touch keys or read-side increments hurts. Counter narrowing is neutral on hit-rate.
- Main region (probation/protected segments): the incumbent protected share is near the best setting; both a halved cap and a much larger share lost. Demotion to the back of probation is the better placement; push-to-front lost.
- Victim selection (probation eviction): comparing several LRU-tail candidates by sketch estimate is the strongest lever, with diminishing returns. Breaking ties in favour of the newcomer hurts.
- Pre-admission structures: a recency window ahead of LRU with admission at exit lost hit-rate; a doorkeeper used as an update filter lost hit-rate. Neither should be retried in its current form.
- Ghost history: the +1 recently-evicted bonus is marginal and not worth keeping without a more selective signal and measured cost.
