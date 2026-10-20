# Variant v002 Insight

> Source note: I could not open the overlay source files (no directory listing was available and the usual paths did not resolve), and I could not read the `h02_split_report` output. These conclusions come from the H02 change spec, the two fitness vectors, and the parent insight. Mechanism claims below are inferences, not checked against the diff.

## Helped

- per-op-time (`put` p50): The typical `put` did not get slower even though `ghost_map` is now sized for `capacity + 1` entries instead of `3·capacity/4 + 1`. Most `put` misses do one ghost probe whose cost does not depend on table size. The slight improvement at p50 is best read as noise, not a gain.

## Hurt

- hit-rate: Growing the ghost to the full `capacity` did not let useful medium-reuse keys come back. The hit rate went slightly down. The likely cause is that the extra quarter-capacity of ghost history mostly readmits keys whose reuse distance is too long to survive in main. Each such readmission goes straight into main and evicts a main resident that was more valuable. If, as the spec says, ghost hits also drive the small/main trigger rules, more ghost hits may push small toward its `capacity/64` floor and hurt one-hit-wonder filtering.
- per-op-time (`put` p99): The `put` tail grew noticeably. This fits a larger `ghost_ring` and `ghost_map` working set: ghost probes and the `ghost_push` overwrite/remove pair miss in cache more often. `get` is unchanged, as the hypothesis predicted, because it never touches the ghost.

## Takeaway

On this LIRS workload-a trace at size 2048, the ghost was not limiting hit-rate at `3·capacity/4`. Adding a full `capacity` of ghost history admits more stale keys into main than reuse-distance keys it saves, and it costs `put` tail latency. The next ghost-capacity probe should go the other way, or make ghost hits admit into main less eagerly (for example, require a ghost hit *and* a frequency check). It should not grow the ring further; the planned `2·cap` row in `h02_split_report` is unlikely to change this conclusion. Check that report's `floor_blocks` and ghost-hit counts to confirm whether small got pinned at the floor.
