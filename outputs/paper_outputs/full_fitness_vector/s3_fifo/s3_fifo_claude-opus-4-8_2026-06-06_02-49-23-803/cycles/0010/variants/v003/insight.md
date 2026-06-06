# Variant v003 Insight

## Helped

- hit-rate: raising `MAX_FREQ` from 3 to 7 lets demonstrably hot keys bank more second chances, so they survive additional `evict_main` decrement-and-recirculate sweeps — clearing the long-standing 0.7312 plateau and edging past the 0.7320 the trace touched once before.

## Hurt

- (none) — metadata-size stayed at 8 because the freq counter still lives in the map slot's trailing `u8` (0..=255), so a ceiling of 7 fits without growing `Entry`; per-op-time summary (get p50) was unchanged, with only sub-noise p99 jitter (put 832→882ns) from a few extra rotations per eviction.

## Takeaway

The hit-rate plateau was a retention ceiling, not a structural limit: keeping the strict `freq > 1` admission gate untouched and only loosening the *retention* cap turns proven reuse into more eviction-pass survival, buying a small real hit-rate gain for free on metadata and latency. This confirms the cross-cycle signal that already-hot keys had unexploited headroom under a too-low freq ceiling.
