# Variant v002 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 3 to 7 lets a key in `main` build up to 7 credits. Each `main` pass in `evict_one` spends one credit, so a hot key can now survive up to 7 lazy reinsertions instead of 3. On workload-a, this protected the hot set a little better against loop and scan pressure, and it beat the parent by a small margin. The gain was small, though, about the size of the cycle-5 ghost-halving gain, which also counted as marginal.

## Hurt

(No measured dimension regressed. The fitness vector reports only `hit-rate`. Three costs exist in principle but weren't measured, so they don't show up here as `per-op-time`:
- a worst-case eviction sweep over `main` can now take up to 7 passes instead of 3
- keys that were hot but have gone stale take longer to drain out of `main`
- `Node.freq` was already a `u8`, so memory use didn't change

The new `freq_saturates_at_max_freq` test only checks the cap value. It has no bearing on fitness.)

## Takeaway

On this trace, a wider frequency counter in `main` helps only slightly. If the cap were too sticky, with stale hot keys clogging `main`, hit-rate would have dropped. It didn't, so that failure mode isn't dominant at size 2048. But more than doubling the counter range bought very little. That suggests that telling 3 hits apart from 10 hits matters little here, and that `main` retention is nearly as saturated a lever as admission was. Before going further on retention (a larger cap or decay-style aging), check this result at a second cache size. A gain this small, from one trace at one size, could just be noise from that trace.
