# Variant v002 Insight

Note: I could not read or diff the overlay source in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- per-op-time: Only the tail moved slightly (get p99 and put p50 each a nanosecond lower), which is within noise. The saturation early-exit in the sketch `increment` skips counter writes only when counters are at cap, and that is rare at this sketch size and workload.

## Hurt

- Nothing moved for the worse. Hit-rate and metadata-size are unchanged, as intended. The early-exit leaves counter values, admission and eviction untouched, and it adds no fields.

## Takeaway

Skipping sketch writes at saturation is hit-rate-neutral, but it does not cut median per-op latency. The p50 get time did not move, so the sketch increment was not a meaningful share of per-op cost on this lineage. Further latency work should target something other than the saturation path, and the hash derivation is the obvious thing to test next.
