# Variant v001 Insight

## Helped

- hit-rate: `get` now passes `InsertPosition::Back`, so stash hits land at MRU of the target bucket and a single further hit makes them the promotion candidate for probing. Demoted and cascaded items also stop landing at the LRU end, so they are not the first to be evicted. The recency-heavy LIRS trace rewards this. The gain is modest.
- per-op-time: Both get and put latency dropped, even though the change only flips an enum argument and adds no work. The likely cause is cheaper bucket operations when inserting at the back, and fewer repeated promote/demote/cascade chains. Part of the drop may be measurement noise, because the code path is otherwise identical.

## Hurt

- Nothing measurable. metadata-size is unchanged because no struct layout was touched.

## Takeaway

Insert position is a cheap, zero-footprint lever in this segment design. Treating hits as MRU rather than cold-landing improved hit-rate on a recency-skewed workload without adding any state. The gain is small, so insert position alone is unlikely to be the main driver. A follow-up could test whether the Front/Back choice should differ between stash→target promotion and demotion cascades, since this variant changes both together. The latency gain should be re-measured to separate real effect from noise.
