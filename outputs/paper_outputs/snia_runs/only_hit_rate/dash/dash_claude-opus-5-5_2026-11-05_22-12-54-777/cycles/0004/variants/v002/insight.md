# Variant v002 Insight

## Helped

- hit-rate: The per-segment 64-slot ghost ring credits keys that come back soon after being dropped from the stash. `DashSegment::put` admits a remembered key with `lfu_counter = 1`, so it beats one-touch arrivals in the 8-slot stash tail scan. hit-rate went up on hm_0 at both sizes, which matches the hypothesis's long-gap-reuse target. The gain is small and about the same at both sizes. That fits the mechanism: the ghost only remembers about 64 drops per segment, and the reward is one counter step, not a placement change. So it can only save keys whose reuse distance is just past the stash's residency. Reuse gaps longer than that are still lost.

## Hurt

- hit-rate (web_0): No real regression. web_0 is unchanged at size 16384 and moved by a noise-level amount at size 131072. This supports the rationale that few keys in this scan-heavy trace return within the ghost's short memory, so the ghost rarely fires there and costs nothing.
- (Not measured: this fitness vector reports only hit-rate. The hypothesis asked for metadata-size per entry against 40, since the ring adds about 130 bytes per segment and `Item` is unchanged. It also asked for per-op-time, since every `put` now does a linear scan over 64 entries, and `put_into_stash` records a fingerprint on every stash drop. Neither was re-measured, so the overhead on the admission path is still unknown.)

## Takeaway

Admission-side ghost credit does reach hm_0's long-gap reuse, which earlier recency-position changes could not. But at 64 fingerprints per segment, with a one-step counter bonus that halve-on-cap ageing can wipe out, the effect is marginal. The mechanism points the right way, but its reach is too short. The obvious next knobs are a larger `GHOST_SLOTS`, so the ghost covers reuse distances well beyond stash residency, or a bigger admission bonus on a ghost hit. Either one should be tested together with a per-op-time measurement, because a bigger ring makes the scan on every `put` cost more.
