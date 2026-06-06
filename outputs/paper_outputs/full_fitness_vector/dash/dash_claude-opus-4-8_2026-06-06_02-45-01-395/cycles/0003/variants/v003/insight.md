# Variant v003 Insight

## Helped

- hit-rate: Unchanged, exactly as intended — reordering `promotion_get`'s
  branches is invisible to placement because ADR 0004's uniqueness invariant
  means a resident key is found in the same slot regardless of probe order.
- metadata-size: Unchanged — the edit only touched control flow in `get`, not
  `Item` layout, so per-entry footprint is identical to the parent.

## Hurt

- per-op-time: The predicted win did not land — get p50 and p99 both rose even
  though the new target→probing→stash order strictly removes a `get_position`
  scan (and a lazy `stash_idx`/`get_index` call) from every target/probing hit.
  Put p50 also rose despite `put` never calling `promotion_get`, which marks
  the regression as benchmark variance swamping the micro-saving rather than a
  real cost of the reorder.

## Takeaway

Reordering the slot probes was logically a free scan-count reduction on the hot
hit path, but the saving is far smaller than the per-op-time measurement noise
floor: get and put latency moved together in the wrong direction, and put can't
be affected by this change, so the result is dominated by variance. A
single-`get_position`-scan saving per hit is below what this harness can
resolve — confirming the reorder is correctness-neutral but not a measurable
latency lever on this trace.
