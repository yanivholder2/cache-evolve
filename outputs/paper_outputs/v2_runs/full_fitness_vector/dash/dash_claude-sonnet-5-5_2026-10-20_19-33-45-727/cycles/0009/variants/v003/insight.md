# Variant v003 Insight

(Note: only the hypothesis and fitness vectors were readable; overlay source could not be listed, so the code-level reasoning below is inferred from the hypothesis.)

## Helped

- hit-rate: Unchanged versus the parent, as the hypothesis intended; admission semantics for non-resident keys and eviction/promotion were left alone, so the update-path early return did not alter which keys get cached.
- metadata-size: Unchanged; the change only reorders control flow in put and adds no per-item state.

## Hurt

- per-op-time: Both get and put latency are clearly worse than the parent, despite the change targeting only update-puts. Skipping the second-hit bookkeeping on resident keys cannot plausibly slow the get path, so the regression likely comes from something other than the intended logic, such as changed code layout or inlining, a lost fast path, or measurement noise or environment variation between cycles.

## Takeaway

Trimming second-hit filter work on update puts did not buy latency. Whether it was the early return or an incidental side effect, this variant is strictly worse than its parent on per-op-time at equal hit-rate and metadata-size. The filter bookkeeping on resident keys was not a meaningful cost, and the parent's per-op-time lead is fragile. Any future latency change should be re-measured against the parent in the same run to separate real effects from drift.
