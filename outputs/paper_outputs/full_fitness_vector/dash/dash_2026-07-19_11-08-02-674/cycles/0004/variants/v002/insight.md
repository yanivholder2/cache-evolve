# Variant v002 Insight

## Helped

*(no dimensions moved positively)*

## Hurt

- **metadata-size**: The bypass logic itself adds no new fields to `Item`, yet per-item footprint grew substantially compared to the parent; the v002 overlay covers only `dash_segment.rs` and does not re-apply v003's trimmed `Item` definition, so the build falls back to the heavier base struct that v003 had specifically slimmed down.
- **per-op-time**: Put p99 doubled relative to the parent; routing stash hits into probing via the bypass increases probing turnover — evicted probing residents cascade through their true target and into stash, making stash inserts (the put path) more contested at the tail. The larger per-item footprint from the missing Item trim compounds this by increasing cache-line pressure on every operation.

## Takeaway

The vacancy-gated stash bypass produced no measurable hit-rate benefit on this trace, suggesting that the joint condition — target full *and* probing simultaneously vacant at stash-hit time — fires rarely enough that the rerouted items represent a negligible share of total accesses. Beyond the structural ineffectiveness, the variant exposed a coverage gap in the overlay: by touching only `dash_segment.rs`, it silently shed v003's Item-struct cleanup, reverting to a heavier per-item layout that inflated both memory footprint and put-tail latency. Future stash-bypass attempts must carry the full Item definition or validate that the overlay inherits the parent's struct changes.
