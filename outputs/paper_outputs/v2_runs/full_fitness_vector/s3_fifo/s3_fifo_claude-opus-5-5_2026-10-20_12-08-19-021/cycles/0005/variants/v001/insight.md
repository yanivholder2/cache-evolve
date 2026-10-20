# Variant v001 Insight

## Helped

- hit-rate: Lowering the ghost-hit shrink guard from `capacity/16` (128 at cap 2048) to `capacity/64` (32) via `small_floor()` / `SMALL_FLOOR_DIV` was the only change that affects the policy. The non-test eviction, graduation, upper clamp and ghost-sizing paths are byte-identical to the parent. So any change in hit-rate means the parent's split did reach the old floor on workload-a, and ghost hits that had been blocked there now move slots into main. The gain is real but small. Main gets some extra room past the old floor, which helps a little, and each slot it gains is a slot small loses for screening new arrivals.

## Hurt

- per-op-time (`get`, the summary): `get` is byte-identical to the parent's. The new `floor_div` / `stats` fields are `#[cfg(test)]` only, and the `S3Fifo` size row confirms that the release layout did not change. The `put`-side change is one constant in a comparison. The code therefore cannot explain the `get` regression, especially the jump at p99. It is most likely measurement noise or a code-layout side effect. A second possibility is that a larger main means more resident main entries with nonzero `freq`, but that would show up in `put`'s eviction loop, not in `get`. Re-measure this before blaming the clamp for it.

(metadata-size did not move. No release-build field was added, and `Entry` did not change. `put` latency was essentially flat. That fits a change that only alters how often a ghost hit's comparison passes.)

## Takeaway

The `capacity/16` floor was a binding constraint on workload-a, which answers the question cycle 0004 left open. If the floor were inert, the hit-rate would have been identical. But the small gain from moving the floor 4× lower suggests the split only sits at the floor part of the time, or that the payoff from more main levels off quickly. Further cuts to the floor are unlikely to pay much. The settled `small_size` / `floor_blocks` figures from `h01_split_report` belong in the variant notes; I could not see them from here. They should decide whether the next lever is the floor again or the grow-small trigger that pulls the split back up.
