# Cycle 0004 Insight

## Cohort outcome

All three variants came from cycle 3's v001 (ghost at half of capacity) and each tried a different way to filter the ghost-hit path into `main`:
- **v001** shrank the ghost again, to a quarter of capacity. It won with a modest gain.
- **v003** kept the ghost size and added a count-min frequency gate on ghost re-admission. Its gain was only marginal.
- **v002** routed ghost hits into `small` and required two hits for cold keys. Its hit rate regressed.

## Patterns

- **Shrinking the ghost is still the best lever, but the gains are getting smaller.** Two halvings in a row have helped, and this one helped less than the last. The optimum is probably close to a quarter of capacity or a little below it. It has only been measured at one cache size.
- **A ghost hit is a useful reuse signal once the window is short.** Cycle 2 called ghost hits misleading on long loops. With a short window, though, they mostly catch real reuse. v002 made ghost-hit keys prove reuse again inside the tiny probation queue and lost hit rate. v003's frequency gate found little left to filter.
- **Requiring reuse while resident has hit a limit.** In earlier cycles, stricter admission kept paying off. Here it stopped: with a probation queue of about 10%, a two-hit bar or a forced pass through `small` evicts keys whose reuse gap is only slightly longer than that queue. Admission can be too strict as well as too loose.
- **Gates on the ghost path leak.** Both v002 and v003 left keys a second route into `main` through `small`, the same leak cycle 3 found with the promotion bar. v003 also compared against `main.front()`, which is often not the entry that actually gets evicted. Any admission gate should compare against the real eviction victim.
- **Measurement is still missing, for the fourth cycle running.** v001's smaller ghost probably saves memory. v003's sketch probably costs both memory and time, since it adds a table and extra hashing on every operation. Neither effect has been verified.

## Open questions

- Where does hit rate turn over: an eighth of capacity, or lower? Does the best ghost size hold at other cache sizes?
- Was v002's regression caused by the routing change or by the two-hit bar? The two were never tested separately. Its ghost bookkeeping fix, which stops stale entries erasing newer ones, is correct and should be kept.
- Would TinyLFU-style admission against the real eviction victim, applied to all insertions and not only ghost hits, pay back the sketch's unmeasured cost?

## Continuation

Use this cycle's v001 as the parent. Run one more ghost-size probe at an eighth of capacity, at more than one cache size if possible, then freeze the ghost size. Port v002's sequence-number ghost fix by itself as a correctness change.

Drop forced routing of ghost hits through `small`, and drop two-hit promotion bars for now. Drop ghost-path-only frequency gates. Bring the sketch back only as victim-based TinyLFU admission, and only once `per-op-time` and `metadata-size` can be measured. With the ghost size settling, run the long-postponed representation-only experiments next: the fingerprint ring and the hasher plus narrow-index graft. Getting the harness to measure those two dimensions is still the top priority.
