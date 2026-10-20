# Cycle 0006 Insight

## Cohort outcome

Variants tried a lower insertion point (len/5, v001), plain 64-byte bucket alignment on top of swap insertion (v002), and a rebuilt stateless mid-bucket insertion with transpose promotion directly on baseline (v003). All three were Winners. Only v003 shows a plausible mechanism-backed gain, a hit-rate improvement with a put-tail cost. v001 and v002 have insights that report a loss.

## Patterns

- Winner status is again weakly tied to the variants' own evidence. v001 reports lower hit rate and slower get and put. v002 reports slower get and put with a hit-rate gain that has no plausible cause. This is the fourth straight cycle where Winners include variants whose own insights are negative.
- Insertion position looks saturated on both sides. Moving from len/4 toward len/3 (cycle 0005) and toward len/5 (this cycle) both failed to help, so len/4 is a local optimum for position-only tuning.
- Cache-line alignment, which cycle 0005 suggested might help put-tail latency, did not repeat. It made latency worse and its hit-rate change was unexplained. Allocation-only layout tweaks are not a reliable win.
- v003 independently reproduces the mid-insertion plus transpose result from cycle 0003. Stateless scan resistance works, and its cost is data-dependent shuffling in the put tail.
- Get latency again did not improve. This is now six cycles with no get gain.
- No variant changed metadata size.

## Open questions

- Are any of these Winner statuses real, or do they come from noisy latency and a flat get?
- Does v003's put-tail penalty vanish when it is combined with swap-based insertion at len/4?
- Where does get time go, given that position, layout, hashing and reordering are all ruled out? It may be harness or dispatch overhead.
- Does anything hold on a second workload?

## Continuation

Drop further tuning of the insertion constant and drop alignment or other layout work; both are exhausted. Treat swap-based len/4 insertion with transpose promotion as the settled baseline. Aim the next hypotheses at a different mechanism, such as the hit-promotion path or per-slot signals under the 40-byte metadata budget. Also add a second workload and a noise check, and discount Winners whose own insight shows a loss.
