# Cycle 0009 Insight

## Cohort outcome

The cycle tried a keyed-lookup refactor that replaced a linear queue scan with one map probe (v001), ARC-style adaptive small-queue sizing (v002), and a lower main frequency cap (v003). Only v001 has a mechanism-backed gain, a tail-latency win with hit rate unchanged. v002 and v003 were selected as Winners without a real gain.

## Patterns

- v001 won by removing an O(n) scan, so the gain shows in the `get`/`put` tails and not in the medians. The earlier cycles' map-cost findings are consistent with this.
- v002's feedback loop did nothing on this trace. Hit rate matched the fixed parent, and the added counters, window check and drain loop only added state and latency. Adaptive sizing is not worth its cost without evidence that the ghost-hit ratio leaves its dead band.
- v003 traded a slightly lower hit rate for a marginally shorter eviction tail. The frequency cap of 3 protects reused main entries, so lowering it is not free.
- Two of three Winners are likely noise-assisted. This continues the Cycle 0008 trend: selection without a mechanism-backed gain is unreliable.
- Policy-parameter tweaks (small ratio, frequency cap, adaptive sizing) are now saturated or harmful on this trace.
- Evidence is still one trace at one cache size. v001's analysis was done without reading the sources, so it is weaker than usual.

## Open questions

- Does v001's single-lookup structure conflict with, or duplicate, the tagged index and split ghost map from Cycles 0006–0007? The two lines of work need to be reconciled.
- Does the adaptive-sizing signal ever leave its dead band on other traces or sizes?
- Do the latency gains survive repeated runs and other traces, and what is the real heap footprint?

## Continuation

Keep v001's keyed single-lookup as the base, after confirming it is compatible with the tagged index and Fx hasher. Drop adaptive small sizing and frequency-cap sweeps, and stop parameter tuning generally. Next cycle should try promotion or ghost-policy changes that do not loosen main admission, and validate on multiple traces with repeated runs and an explicit heap measurement.
