## Recurring wins

- **Admission filtering is the lever that works.** S3-FIFO's gain over LRU (cycle 1) came from controlling which keys enter `main`, not from how recency is tracked. Every later winner refined admission further.
- **Shrinking the ghost window keeps paying, with smaller gains each time.** Each of four halvings in a row (cycles 3–5, from near capacity down to an eighth) improved hit rate, and each gained less than the one before. The ghost is now about the same size as `small`.
- **A short ghost makes ghost hits a trustworthy reuse signal.** With a long window, ghost hits misled admission on long loops. Once the window was short, the direct ghost-hit path into `main` became reliable, and adding filters to it gained little or lost hit rate.
- **The fixed ~10% probation split holds up.** Moving away from it, whether adaptively (cycle 2) or by enlarging LIRS's probation buffer (cycle 3), cost hit rate.
- **Representation-only changes leave hit rate alone, as expected.** The CLOCK-style links (cycle 1) and the fingerprint ring at the same window size (cycle 5) both kept hit rate level. They are safe bases for later policy work, though their cost benefits are still unverified.

## Recurring failures

- **Measurement of `metadata-size` and `per-op-time` has been missing for five cycles running.** Every memory or speed claim is still untested: the narrow links, the hasher, the fingerprint ring, the smaller ghosts, and the costs of the sketch and LIRS. Fixing this now blocks judging most hypotheses.
- **Loosening admission costs hit rate.** Longer ghost windows, ARC-style adaptive `small` sizing, and other fast paths that let keys into `main` without proven reuse all regressed (cycle 2).
- **Over-strict admission also fails.** Two-hit promotion bars and forcing ghost hits back through `small` were flat or regressed (cycles 3–4). Keys whose reuse gap is a little longer than the probation queue get evicted.
- **Gates on only one admission route leak.** A stricter `small` bar is bypassed by the ghost path, and ghost-only gates are bypassed via `small`. Gates that compared against `main.front()` instead of the real eviction victim were also flawed.
- **Frequency sketches gain almost nothing on this design.** A count-min gate on the ghost path, and then full TinyLFU against the real victim with the leak fixed (cycles 4–5), each gained only marginally. The short ghost plus the small-queue filter already rejects most cold keys, and the sketch's cost is unmeasured.
- **Changing recency tracking alone doesn't raise hit rate.** CLOCK roughly matched LRU (cycle 1). Exact LRU order buys nothing here.

## Open questions

- Does the ghost-size curve turn over below `small`'s size (for example a sixteenth of capacity)? Or is the ghost-into-`main` fast path dispensable altogether?
- Do the ghost-size gains hold at another cache size or on another trace? So far all evidence comes from workload-a at a single size.
- Do the fingerprint ring, Fx hasher, and `u32` indices actually reduce `metadata-size` and `per-op-time`? Can one hash per key serve both the fingerprint and the lookup?
- Was cycle 4 v002's regression caused by the routing change or by the two-hit bar? The two were never tested separately.
- Where can further hit-rate gains come from now that admission filtering looks close to saturated?
- Does CLOCK's victim sweep cause tail-latency spikes on `put`?

## Per-policy notes

- **S3-FIFO:** the line that keeps winning. Current best: fixed ~10% `small`, one-hit promotion, ghost at an eighth of capacity with a direct ghost-hit path into `main`. Keep the sequence-number ghost fix as a correctness change, even though its effect on hit rate is negligible.
- **CLOCK / LRU approximations:** hit rate about the same as LRU. Worth pursuing only if measurement shows a memory or latency win.
- **Strict LRU + representation tweaks (Fx hash, `u32` links, MRU fast path):** hit rate unchanged. Candidate for grafting onto S3-FIFO once cost can be measured. Not yet attempted.
- **LIRS:** beats LRU but trails S3-FIFO, with a heavier `Node` and a linear ghost-bound walk. Enlarging its HIR queue hurt. Set aside until costs can be measured and the walk is made O(1).
- **ARC-style adaptivity:** drifted away from the good split and lost hit rate. Dropped.
- **TinyLFU / count-min admission:** marginal gains at best, with unmeasured cost. Dropped unless measurement shows it is nearly free.
