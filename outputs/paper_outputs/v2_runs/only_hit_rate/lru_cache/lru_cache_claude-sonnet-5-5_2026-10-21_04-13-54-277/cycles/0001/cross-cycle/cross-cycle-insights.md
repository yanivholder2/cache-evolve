## Recurring wins

- Only changes that move a dimension the evaluator actually records can win; so far that has been hit-rate via a departure from strict LRU (segmented LRU), accepted by the invariant gate.
- Single-axis hypotheses, each aimed at a different dimension, are easy to attribute, but they do not compete with each other.

## Recurring failures

- Strict-LRU-preserving micro-optimizations (compact layout, hot-path rewrite) are neutral on hit-rate, so they cannot win while hit-rate is the only measured axis. Their gains are predictions, not results.
- Missing metadata-size and per-op-time measurements leave selection on hit-rate alone. This hides the cost of added bookkeeping (extra flags, a second list, demotion work).
- Hit-rate evidence so far comes from one trace and one cache size, which is thin.

## Open questions

- Is the missing metadata-size and per-op-time data a harness gap or a configuration issue? Resolve this first.
- Does the segmented-LRU gain generalize across traces (not just scan-heavy ones) and cache sizes?
- What metadata and latency does segmentation cost?
- Can compact links and cheap promotion combine with segmentation without cancelling each other?

## Per-policy notes

- Strict LRU: hit-rate matches the parent in every variant, so there is no headroom on the measured axis. Do not propose more micro-optimizations until size and time are measured.
- Segmented LRU: the current best on hit-rate. The gain is unverified beyond one trace, and its costs are unmeasured. Next step is to combine it with compact layout and fast-path ideas.
