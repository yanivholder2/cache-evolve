# Cycle 0004 Insight

## Cohort outcome

Three variants targeted per-op-time through mechanics rather than policy: a get-hit lookup/locking cleanup (v001), a put-path free-slot fast path (v002), and a from-baseline re-derivation of the bit-packed second-hit filter with unread bookkeeping stripped (v003). All were Winners, but only v003 genuinely improved. v001 and v002 held hit-rate and metadata flat and made latency worse.

## Patterns

- Hand-optimizing the hot paths of an already-fast incumbent backfired twice. Atomics, early returns, and merged scans or bitmask checks added work or branches that did not pay off. This is the same failure mode cycle 0003 saw with conditional guards.
- Both latency regressions also showed up on paths the change did not touch (put for v001, get for v002). This suggests code-layout effects or measurement noise, so the regressions cannot be firmly attributed.
- v003 re-derived the second-hit filter from baseline and improved both hit-rate and latency at flat metadata. That independently confirms cycle 0001/0002's finding that this filter is the one real win.
- v003's gain mixes the filter policy with removal of baseline per-get statistics and timestamp writes. Those two effects cannot be separated from these results.
- Removing work that the policy never reads helped. Adding mechanisms to speed up existing work did not.
- Insert-path fast paths look weak, probably because buckets are mostly full under this workload.

## Open questions

- Are the v001 and v002 latency regressions real, or run-to-run and layout noise?
- How much of v003's latency gain comes from dropping dead bookkeeping, and how much from skipping the first-hit remove-and-promote?
- Where do get and put time actually go? Nothing has measured this yet.
- Does the cleaned-up baseline route (v003) match or beat the cycle 0002 v001 incumbent?

## Continuation

Treat v003 (minimal bit-packed second-hit filter with dead bookkeeping removed) as the new base, and compare it directly with cycle 0002's v001. Pursue subtractive changes only, such as deleting other unread writes or stats and removing redundant work. Drop further lookup-atomics, early-return and free-slot fast-path variants. Re-measure latency, with repeats, before trusting any result, and consider a different trace for hit-rate questions.
