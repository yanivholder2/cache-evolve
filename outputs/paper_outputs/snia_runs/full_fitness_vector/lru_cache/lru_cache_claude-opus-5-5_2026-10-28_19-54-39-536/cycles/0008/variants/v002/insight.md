# Variant v002 Insight

## Helped

- per-op-time (get p50): Get p50 dropped clearly even though `get` and `on_hit` were not changed. Going from `Box<[u16]>` to `Box<[u8]>` halves the ghost table (16 KB at 16K, 64 KB at 131K), which leaves more cache lines for the node arena and map buckets that `get` uses. This is the first direct evidence for the cycle 3–7 suspicion that the ghost's footprint, and not its code path, was what slowed get.
- per-op-time (put p50): Put p50 eased slightly. This fits put's `Vacant` arm doing the same two ghost accesses (lookup plus victim-tag store) on a table that is half the size, so those accesses are more likely to hit in cache. The move is small enough that it may be noise.
- hit-rate: All four trace/size points match the parent exactly. The new tag is `(h >> 24) as u8 | 1`, which is the upper byte of the old `(h >> 16) as u16 | 1` tag. So every reprieve the parent granted is still granted, and the only possible change is extra spurious reprieves. Those cost nothing measurable here, which supports the claim that one extra probation pass is cheap.

## Hurt

- per-op-time (get p99): Get p99 rose a lot while get p50 fell. The diff touches nothing on the get path and leaves `Node` and the header layout unchanged, so no code change explains a worse tail. Treat it as tail noise or a layout/alignment shift for now, and re-measure before reading it as a real regression. Parent-cycle p99 values have already swung without a matching code cause.

## Takeaway

Shrinking the ghost tag from 16 to 8 bits gave a clear get-p50 gain with no hit-rate cost. That isolates the ghost table's cache footprint as a real source of get latency, even though get never reads the table. Shrinking side structures in bytes, rather than changing the policy, is therefore a productive direction, and the next suspect can be tested the same way. One caveat: the hit-rate is bit-identical, even though 8-bit tags should collide about 256× more often. Confirm that the hit-rate harness actually runs this ghost path, for example by counting spurious reprieves, before relying on "free" hit-rate results from tag-width changes.
