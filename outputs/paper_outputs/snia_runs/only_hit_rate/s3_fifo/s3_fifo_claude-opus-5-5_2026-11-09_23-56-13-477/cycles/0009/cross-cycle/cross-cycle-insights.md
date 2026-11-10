# Cross-Cycle Insights (through cycle 0009)

## Recurring wins

- Per-key evidence beats global signals. Every real policy gain (0004 ghost-time freq gate, 0005 ghost-age floor for freq-0 returnees, 0009 correlated-reference cap) reads something the key itself carries. No global rate or occupancy proxy has ever won.
- Strict admission into a full main holds up. The baseline two-reuse rule plus freq reset (0001–0003) has never been beaten by loosening it. Gains come from filtering or deferring keys before they reach main, not from protecting them once inside.
- Deferring is safer than rejecting. Rerouting weak keys back to small (0004), or holding freq 1 into the ghost until a later spaced return (0009), keeps their chance to come back instead of discarding them.
- Ghost age gated at about one small-queue turnover is a confirmed local optimum: 2× lost in 0006 and ½× lost in 0007.
- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) have stayed bit-identical in hit-rate every time, so they are always safe to port onto the current policy parent.

## Recurring failures

- Single constant tweaks (threshold, static small/main split) trade hm_0 against web_0 and never win on both traces (0001, 0002, 0008).
- Global adaptive signals: main occupancy fills and stays full (0002). Ghost-return-rate switching fired on the wrong traces (0003). An unweighted ARC-style split drifts. None of them follow steady-state value.
- Extra protection or credit in main or small (returnee credit, relaps, a saturated-freq head start, extra small laps) displaces useful residents. Main is always full, and quick freq-0 returnees especially should not be pushed toward main's tail (0003, 0005–0007).
- Mechanisms that erase or dilute the per-key signal break the ghost gate. Freq-resetting relaps (0005), and extra ghost producers that share the FIFO or budget (0007), are examples.
- Scaling the ghost horizon or budget does not help. Uniform doubling was roughly neutral (0008), age tiering removed a privilege hm_0 needs (0009), and a `main_ghost` scaled to `main_size` did nothing (0009).
- web_0 at the large size has resisted every lever: ghost routing, age, budgets, the split, main-victim memory and burst filtering. At most it moved marginally.
- The cost dimensions (per-op-time, metadata-size) have gone unmeasured for nine cycles running, despite repeated requests. Cost hypotheses cannot be judged, and ghost-enlarging variants carry metadata costs nobody has seen.

## Open questions

- Does a narrower correlated-reference window keep 0009-v003's hm_0 gains while sparing web_0's genuine quick reuses at the small size?
- Is web_0 at the large size at its ceiling for this policy family? The last untried probes are the `main_hit` gate on main-victim ghosting and the freq assigned to main-ghost returnees. If both fail, stop targeting this cell.
- What do the cost refactors actually save in per-op-time and metadata-size, and does alignment on 8-byte keys cancel the ghost-layout savings?
- Can a per-key signal ever break the hm_0/web_0 trade-off at both sizes at once, or are the remaining gains limited to the small size?

## Per-policy notes

- S3-FIFO (the only family explored), current line: baseline two-reuse rule plus freq reset → O(1) index → freq-gated ghost path (0004) → age-gated freq-0 returnees at 1× turnover (0005-v002) → correlated-reference cap (0009-v003, the current base).
- Trace behaviour: hm_0 rewards more probation and memory at tight capacity, and has burst-only keys that pollute main. web_0 punishes privileges for keys returning after a long gap, and treats quick reuse after insertion as real evidence.
- Cache size: gains concentrate at the small size. Only windows that scale with `small_size` have reached the large size.
- Exhausted axes (do not retry): static split, threshold constants, ghost-age window tuning, relap stacking, age ceilings and tiers, ghost and `main_ghost` budget scaling, stamp-clock fixes.
