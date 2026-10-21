# Cycle 0008 Insight

## Cohort outcome

All three variants came from cycle 7's v002 (`small` at a fortieth, ghost at three-twentieths):
- **v003** halved `small` again to an eightieth and kept the combined window fixed. It won, by a sliver.
- **v001** carried extra `small` hits into `main` as credits, together with `MAX_FREQ = 7`. It won, by a tiny amount.
- **v002** replaced `main`'s FIFO-with-reinsertion with a SIEVE hand. Hit rate went down slightly.

## Patterns

- **The `small`/ghost split curve has flattened.** The eightieth step gained less than the already marginal fortieth step. It is the same shrinking-gains curve that the ghost-only halvings followed. Treat the split as settled somewhere between a fortieth and an eightieth, at least on this trace and size.
- **The two winners work against each other.** Carryover only pays for keys hit more than once while in `small`. v003 shrinks `small`, which leaves even fewer such keys. v001 also changed the cap and carryover together, so its own result cannot say which of the two caused the gain. Compared with cycle 7's cap-only result, credit work on the `small` path looks close to used up.
- **`main`'s full-rotation grace period is useful, not waste.** SIEVE lost because zero-credit arrivals on workload-a are often reused later, just not right away. This agrees with cycle 7: `main`'s CLOCK loop already gives weak-evidence keys enough protection. Both experiments that changed how zero-credit arrivals are treated, a free credit and faster demotion, made things worse.
- **The variant insights keep proposing a fix that has already failed.** Both v002 and v003 suggest giving ghost arrivals a starting credit. Cycle 7's v003 tried exactly that and lost. The "zero-credit ghost arrivals" bottleneck is a story, not a measured limit.
- **The evidence gaps are still open, now for the eighth cycle.** Every result is still on one trace at one cache size, with gains too small to trust from that alone. The fingerprint ring has not been ported, and `metadata-size` and `per-op-time` are still unmeasured.

## Open questions

- Do the split ratio and the carryover gain survive at a second cache size, or are they tuned to workload-a at 2048?
- How much of v001's gain came from carryover and how much from the higher cap? Does carryover add anything on top of v003's smaller `small`?
- If `main`'s scan order, credit cap and entry credit are all exhausted, where is the remaining hit-rate headroom? Should the ghost also record `main` evictions?

## Continuation

Use v003 as the parent and port v001's carryover with `MAX_FREQ = 7` onto it, as one cheap combined check. Expect little from it. The main job of the next cycle is to measure that combination at a second cache size, and, if possible, on a second trace.

Drop these lines of work:
- shrinking `small` further
- changes to `main`'s scan order (SIEVE or similar)
- starting credits for ghost arrivals, which have now been rejected twice
- raising the credit cap

If you want a new hit-rate lever, look beyond `small` admission and `main` retention, for example at what feeds the ghost. Otherwise, spend the cycle on the long-pending fingerprint-ring port and on measuring `metadata-size` and `per-op-time`.
