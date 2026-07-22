# Variant v001 Insight

## Helped

_(none)_

## Hurt

- **hit-rate**: The first-touch teleport to the warm-cold boundary displaces items already sitting near `boundary_pos`, pushing them deeper into the cold window and exposing them to earlier eviction; the items rescued from the cold zone tail do not compensate for the items newly endangered at the boundary, leaving hit rate marginally lower than the uniform 3-swap parent.
- **per-op-time**: The variable-length swap loop used for first-touch teleportation performs up to `scan_depth` (8) swaps instead of the fixed 3 swaps of the parent, doubling put tail latency (p99) because newly inserted items deep in the cold zone now require the full sweep to reach the boundary on their first access.

## Takeaway

Teleporting first-touch cold-zone items all the way to the warm-cold boundary does not improve hit rate on this workload — the positional gain for deeply cold items is offset by boundary-adjacent items being pushed further toward eviction, producing a net-neutral to slightly negative hit-rate outcome. The more decisive effect is on tail latency: replacing a constant-cost 3-swap with a depth-dependent swap loop up to 8 iterations long doubles the put p99, adding measurable cost without a compensating throughput benefit. Future promotion experiments should keep promotion work O(1) or bounded tightly to avoid this latency regression.
