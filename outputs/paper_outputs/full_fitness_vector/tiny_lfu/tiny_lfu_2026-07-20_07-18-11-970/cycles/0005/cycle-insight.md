# Cycle 0005 Insight

## Cohort outcome

This cycle pushed three independent threads from Cycle 0004: tightening the best-of-2 eviction probe rate, replacing the second SipHash with a cheap bijective remix, and stacking a stricter admission gate atop the remix base. Only the bijective-remix variant won, by eliminating the put-path serial hash dependency while leaving hit rate essentially unchanged.

## Patterns

- **The probe-amortisation axis is now bracketed and closed.** 1-in-8 is the upper bound on probe frequency that preserves latency neutrality; 1-in-4 broke p50 and p99 visibly. No further hit-rate gain is available along this axis without reusing already-computed estimates rather than issuing fresh sketch probes.
- **The count-min sketch needs less hash independence than theory suggests.** A three-instruction multiply-xorshift remix produces row mixing that is sufficient for accurate admission decisions on this workload, making the second full SipHash invocation wasteful. The bijective remix is a strict improvement: it removes a serial data dependency on the put path with negligible accuracy cost.
- **Admission gate latency is a residency problem, not an execution-overhead problem.** The stricter gate doubled p50 for both get and put by altering which nodes occupy cache lines — an effect the 1-in-8 eviction amortisation is structurally unable to cancel. The two mechanisms target independent failure modes, so v002's amortisation cannot neutralise v003's residency disruption.
- **Additive hit-rate signals do not imply additive latency budgets.** v003 confirmed that combining two independently beneficial mechanisms can compound their latency costs even when their hit-rate gains compound cleanly.

## Open questions

- Is there a softer admission threshold — applied only on the amortised probe path rather than every eviction — that reduces the number of nodes whose residency changes while still capturing some of the hit-rate benefit?
- With the sketch, hash-work, and eviction-amortisation axes all largely exhausted, are there meaningful gains remaining on the window/protected-segment sizing or probationary-to-protected promotion threshold?

## Continuation

v002's bijective-remix baseline is the new starting point. The amortisation-rate axis (1-in-8 vs. 1-in-4) is definitively closed — pursue it further only if a way to reuse pre-computed frequency estimates appears. The admission gate is still a live thread but requires a softer mechanism: applying the stricter comparison only on the amortised-probe path, or using a fractional threshold, to limit residency churn. If both of those threads quickly exhaust, the next fruitful direction is structural policy parameters (window ratio, protected/probationary balance) that have not been touched in any prior cycle.
