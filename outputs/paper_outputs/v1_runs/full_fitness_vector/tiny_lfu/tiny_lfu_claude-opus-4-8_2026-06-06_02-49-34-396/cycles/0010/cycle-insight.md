# Cycle 0010 Insight

## Cohort outcome

Acting on Cycle 0009's "probe the one untouched lever — the eviction walk
/ victim sampling — for per-op savings" mandate, all three variants
branched off cycle-0005/v001 and attacked that surface three ways: a
floor-frequency early exit (v001), deleting the per-get LRU reorder that
feeds the sample window (v002), and swapping the pointer-chasing walk for
random direct-index sampling (v003). Every variant missed its per-op-time
claim, and v002 additionally lost hit-rate. They are Winners only by
default; the cohort's real product is a clean diagnosis of *why* the walk
won't speed up.

## Patterns

- The eviction walk is confirmed *not* the dominant put-path cost. The
  cheapest possible trim (v001's provably hit-rate-neutral floor exit)
  removed real work yet moved nothing above benchmark noise, and the
  untouched get path drifted by the same margin — so the per-op summary
  is tracking run-to-run variance, not walk length. Cutting estimate
  calls is not the lever.
- The walk's cost was *misattributed* to the link chase. v003 showed the
  cache misses come from touching `nodes` at order-uncorrelated physical
  positions — a property shared by both the link traversal and random
  index draws — not from pointer-chasing itself. Removing the chain bought
  zero locality and stacked PRNG overhead on top. Any future put-path
  speedup must change how victim slots are *laid out*, not how they are
  selected.
- The per-get LRU reorder is load-bearing and nearly free. v002 disproved
  the standing "hit-rate is admission-bound, not recency-bound" premise:
  the move-to-tail is what keeps re-accessed hot keys out of the
  oldest-inserted sample window, and the frequency gate alone does not
  protect them. Deleting it cost hit-rate sharply while buying no
  measurable get-p50 — it stays.

## Open questions

- Is node *layout* (clustering or compacting `nodes` so victim candidates
  are physically contiguous) a real per-op lever, or is the order-correlated
  miss pattern intrinsic to any recency-ordered structure?
- With the walk now ruled a non-surface for speed and hit-rate accepted as
  saturated, is there *any* untaxed cost-axis win left to bank?

## Continuation

Close the eviction-walk thread: it is a confirmed non-surface for per-op
speed (neither floor early-exit, reorder removal, nor random sampling
helped), and victim quality remains sketch-estimate-bound, not
sample-source-bound. The per-get LRU reorder is load-bearing for hit-rate
— do not remove it. The misattribution finding reframes the only remaining
per-op lever as *node layout / physical locality*, not selection logic; if
next cycle pursues per-op-time at all, it should test laying victim
candidates out contiguously. Otherwise treat both hit-rate and the
eviction walk as saturated and stop probing selection-side edits.
