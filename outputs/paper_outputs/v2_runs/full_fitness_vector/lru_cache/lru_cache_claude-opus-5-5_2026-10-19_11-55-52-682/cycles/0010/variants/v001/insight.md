# Variant v001 Insight

## Helped

- per-op-time (`put` tail only): `put` p99 dropped slightly. This is consistent with a few fewer long probe chains on the miss/insert path once the rotated hash feeds hashbrown's bucket index. But the change is no bigger than the parent's own run-to-run wobble, and `put` p50 did not move, so it is not evidence of a real collision reduction.

## Hurt

- per-op-time (`get`): `get` p50 and p99 each got slightly worse rather than better. `finish` now does an extra `rotate_left(26)` on every lookup, and `get` is already at the map-probe floor. Even a single-cycle extra op shows up there when there were few collisions for the better mixing to remove. The regression is noise-sized but goes against the hypothesis's predicted direction.

(`hit-rate` and `metadata-size` stayed flat, as intended. `ghost_hash` exactly undoes the rotation, so ghost slots, fingerprints and eviction order match the parent bit for bit. `Node` and the struct layout were not touched.)

## Takeaway

Swapping plain Fx for a rustc-hash-2.x rotate finalizer did not shorten probes on workload-a. This triggers the hypothesis's own exit condition: the trace's `u64` keys already spread well across hashbrown buckets and tags under the raw Fx product. What is left of the `get` cost is memory latency on the map's control/bucket lines, not collisions. Hash quality is ruled out as a probe lever. The rotate is a free, behaviour-neutral change, but it is not worth keeping for speed. Further `get`/`put` gains need fewer or cheaper memory touches per lookup (for example, a smaller or denser index), not better key mixing.
