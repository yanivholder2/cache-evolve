# Variant v002 Insight

## Helped

- hit-rate: Halving `PROBE_WINDOW` did not cost any hit-rate. Three of the four trace/size cells rose by a tiny, equal amount, and web_0 at the small size did not move, so the summary stays well inside the hypothesis's ±0.0005 band. A narrower window means fewer hottest-position hits qualify for promotion, so fewer probing-LRU residents are evicted outright. The small gain suggests some of those residents had reuse that the wider window was throwing away. Either way, the arm's promotions are clearly not carrying real hits, so the hit-rate refutation condition is not met.
- per-op-time (get p50, get p99): Both get figures fell relative to the parent, which is the direction the hypothesis predicted for a narrower window on the eviction arm. But put p50 and put p99 fell by a larger proportion, even though the change does not touch the put path. That near-uniform drop, including on a path the change did not touch, looks like the parent's environmentally inflated timings (flagged in the v001 insight) going away. It does not look like an effect of the window. The hypothesis's own test is against the same-session H01 incumbent, which I cannot see. Against the parent alone, the get-latency gain cannot be credited to the change.

## Hurt

(none — metadata-size is unchanged. Keeping the window bound as a `const` or a construction-time value held `Dash<u64,u64,DashSegment>` and `Item` at their previous sizes, as step 2 of the hypothesis required.)

## Takeaway

Narrowing `PROBE_WINDOW` on the eviction-arm lineage did not cost hit-rate and may help slightly. This mirrors cycle 0008's finding that widening it gained nothing: the window is not a hit-rate lever in either direction, so it can be tuned purely for latency. Whether the narrower window actually lowers get latency still needs the same-session comparison with H01. The cross-session drop against v001 is mostly environmental, as the matching drop on the untouched put path shows. Caveat: I could not list or open the overlay source in this session. The code-level reasoning assumes H02 was implemented as written, with only the window halved and the bound kept as a `const`.
