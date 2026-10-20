# Cycle 0007 Insight

## Cohort outcome

The cohort tried a deeper sketch (one more hash row), a larger protected share within the main region, and a promotion rule that gates probation-to-protected moves on the sketch estimate. Only one gain was reported, a slight hit-rate rise from the added sketch row (v002); the protected-share increase and the gated promotion both lost ground against the parent. The Winners list names v001, but v001's record reports a slight hit-rate loss, so the Winner designation needs checking before anything is built on it.

## Patterns

- Adding estimator depth helped where removing it hurt in Cycle 0005. Extra rows give the min-across-rows estimate more independent samples, which again points at collision error as the lever in the admission comparison.
- The protected share has a peak rather than a monotone trend. Cycle 0006 credited a larger protected share with a gain; pushing it further here reversed that, which suggests the useful range is close to the current setting and that probation room matters once the share gets large.
- Gating promotion on the sketch estimate hurt. Frequency counts lag behind recency, so the gate kept some entries in probation that were about to earn a second hit. Promotion on every hit appears to be the better rule on this trace.
- Depth's memory cost is unmeasured. The added row grows sketch counter storage in proportion, and no variant reports that footprint or any timing, so the gain cannot yet be weighed against its cost.
- Every result comes from one trace at one cache size, so all differences here are provisional.

## Open questions

- Is the Winners designation for v001 correct? Its record shows a hit-rate loss, while the only reported gain is v002's depth change.
- Does the depth gain hold on other workloads and sizes, and where does adding rows stop paying for the extra counter memory?
- Where is the protected-share optimum on other traces, and does it sit near the current ratio or somewhere between it and the reversed setting?

## Continuation

Next cycle, resolve the Winner attribution first: confirm whether v001 or v002 is the real advance before building on either. If the depth gain is kept, test it on a second workload or size and measure its counter memory before stacking further sketch changes. Hold the protected share at or near the incumbent ratio rather than pushing it higher, and drop the sketch-gated promotion rule.
