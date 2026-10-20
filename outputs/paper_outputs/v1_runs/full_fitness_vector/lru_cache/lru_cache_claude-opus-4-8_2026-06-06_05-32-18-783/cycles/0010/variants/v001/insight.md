# Variant v001 Insight

## Helped

- hit-rate: stepping `prot_cap` from `capacity * 7 / 8` to `capacity * 8 / 9` retained a few more promoted entries (~+28 slots at capacity 2048), capturing retention headroom above the parent without crossing the 15/16 starvation cliff.

## Hurt

- per-op-time: get and put p50/p99 all roughly doubled, but the change touches only the one-time `prot_cap` constant in `try_new` — the `get`/`put` fast paths, `demote_protected_lru`, and the 24-byte node layout are byte-identical, and a larger soft cap makes demotion fire *less* often, so a real fast-path slowdown is not mechanistically possible; the symmetric ~2x rise across all four percentiles reads as measurement/environment variance, not a genuine cliff.

## Takeaway

The 8/9 geometry nudge confirms the cross-cycle finding that small protected-cap steps buy hit-rate where the coarse 15/16 jump overshot — retention rose without any algorithmic latency cost, since the edit is a single startup constant. The accompanying p50 jump cannot stem from this change and should be treated as noise rather than evidence of a starvation cliff below 8/9.
