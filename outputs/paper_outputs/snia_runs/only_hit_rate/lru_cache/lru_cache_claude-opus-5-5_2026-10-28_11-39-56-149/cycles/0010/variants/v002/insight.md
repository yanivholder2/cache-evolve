# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 16384, hm_0 @ 16384): Not a gain but a passed control. `reset_mult()` still returns 32 for 64 ≤ capacity ≤ 32_768, so both small cells match the parent exactly. The invariant held, and every movement below is caused only by the large-size aging period.

## Hurt

- hit-rate (web_0 @ 131072): This is the cell the hypothesis targeted, and it went down instead of up. With a 32 × capacity period, halvings at 131072 are millions of accesses apart. Old frequency stays in the 4-bit counters, which saturate at 15, and that history makes the main-area victims look strong. So keys returning after a long gap lose the duel to entrenched, now-stale residents instead of being protected. Cycle 0009 showed that web_0's long-reuse keys rely on protected residency. Preserving their *history* did not substitute for that residency.
- hit-rate (hm_0 @ 131072): This cell dropped more than web_0 @ 131072, which contradicts the rationale's claim that hm_0's one-shot filtering would get stricter. One likely mechanism: the doorkeeper (4·capacity bits) is cleared only at reset. With the period doubled it stays saturated for longer, so its false positives give one-shot keys an extra +1 and weaken filtering. Meanwhile hm_0's shifting hot set waits twice as long before old counts decay enough for new hot keys to win admission.
- hit-rate (summary): Both large cells moved down and both small cells held, so the summary declined. This lever made no trade between traces: it just cost hit-rate at large size.

## Takeaway

Longer sketch memory does not scale with capacity. 32 × capacity helped at 16384 (c9), but the same multiplier hurt both traces at 131072, where the absolute period is 8× longer. This suggests the useful aging period is bounded in access count rather than proportional to capacity. The large-size gate should therefore keep 16 × (or try shorter), and the doorkeeper's fill over long periods is a suspect worth isolating. The hypothesis expected a lever that helps both large cells together. This change instead hurt both together, so longer memory is ruled out as that lever at 131072.
