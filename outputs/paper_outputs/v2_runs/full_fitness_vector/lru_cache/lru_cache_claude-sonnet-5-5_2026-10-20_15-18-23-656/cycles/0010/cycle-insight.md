# Cycle 0010 Insight

## Cohort outcome

Three structural or admission-style probes were tried: fusing the map remove and insert in the evicting `put` (v001), a ghost-counter admission gate that protects probation victims (v002), and second-chance eviction with an adaptive walk cap (v003). v001 and v002 were Winners, but neither showed a verified gain. v001 left hit-rate and layout unchanged and v002 left hit-rate unchanged. Winner status again looks like noise tolerance. Only v003 improved hit-rate, slightly, and it was not a Winner because it cost latency and metadata.

## Patterns

- Every change that added ghost, counter or reference-bit bookkeeping to the put path made put slower and grew state. The admission and adaptive-cap variants (v002, v003) both did this, and neither bought a clear gain.
- The parent's cold-key insertion rule seems to already capture most of the useful admission signal. A second admission layer (v002) changed no eviction decisions on this trace.
- Fusing map calls (v001) did not speed up put. Together with last cycle's node-slot reuse, this suggests map-call count and allocation are not the bottleneck, or that a single-probe version costs about as much as remove plus insert.
- `get` timing moved in variants that did not meaningfully touch `get` (a large tail drop in v001). This is the same noise seen in every cycle. The noise floor is still unmeasured.
- Metadata growth leaked past the stated layout targets in two variants (struct growth in v002, node and struct growth in v003), despite the hypotheses planning for none.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- What is the per-op-time noise floor? Without it, no latency claim can be credited or blamed. This has been pending for many cycles.
- Does the ghost-gated design hold on non-scan workloads? The check is still pending.
- If map calls and allocation are not the put bottleneck, what is? Hashing, cache misses on list pointers and the eviction walk are untested.
- Is any further hit-rate left on this trace, or is the baseline near its ceiling?

## Continuation

Keep the cycle-0008 ghost-gated baseline as the parent. Drop ghost-counter admission gates, adaptive walk caps and second-chance bits, since they add put cost and state without a verified payoff. Also drop further map-call fusion and allocation tweaks. Next cycle, build the noise-floor (repeat-run) and heap-inclusive memory measures and evaluate on non-scan workloads. Profile where put time actually goes before choosing another lever.
