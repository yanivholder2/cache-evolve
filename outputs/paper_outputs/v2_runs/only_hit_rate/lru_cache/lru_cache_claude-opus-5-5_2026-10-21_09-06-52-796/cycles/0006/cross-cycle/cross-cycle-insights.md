## Recurring wins

- **Admission filtering beats recency tracking.** Every winner since cycle 1 has changed what gets into `main`, not how recency is tracked. S3-FIFO (probation queue plus ghost) has been the lineage parent ever since.
- **Tightening the admission window paid off for several cycles in a row.** Shrinking the ghost won in cycles 3, 4 and 5. Shrinking `small` won in cycle 6. Each gain was smaller than the one before, and the curve is now close to flat.
- **Ghost and `small` work as one combined admission window.** A ghost smaller than `small` adds almost nothing. A ghost larger than `small` acts as a cheap second stage of probation: it costs a ghost key instead of a resident slot, and the freed slots go to `main`.
- **Once the ghost window is short, a ghost hit is a trustworthy reuse signal.** Sending ghost hits straight into `main` beats making those keys prove reuse again.
- **Representation-only changes leave hit rate exactly as it was.** The fingerprint ring and CLOCK's coarse recency both did. They are safe to stack on any policy, but their memory and speed benefits are still unproven.

## Recurring failures

- **`metadata-size` and `per-op-time` have never been measured, in six cycles.** Every claim about memory or speed, including the representation track, the cost of the sketches and LIRS overhead, is unverified. This is the main blocker.
- **Loosening admission costs hit rate.** That includes longer ghost windows, ARC-style adaptive resizing of `small`, and fast paths into `main` that skip proof of reuse.
- **Admission can also be too strict.** Two-hit promotion bars and forcing ghost hits back through a tiny `small` evicted keys whose reuse gap was only slightly longer than the probation queue.
- **Gates applied only to the ghost path leak.** Keys still reach `main` through `small`. Gates that compared against an entry other than the real eviction victim were also ineffective.
- **Frequency admission is near its ceiling.** Count-min gates and full TinyLFU admission against the true victim (cycles 4 and 5) gained only marginally on top of the S3-FIFO filter, while adding per-access cost that has never been measured.
- **No evidence beyond a single setup.** Every result comes from one trace (workload-a) at one cache size. A second size was requested in cycles 4, 5 and 6 and has never been delivered.
- **Planned follow-ups keep getting skipped.** The hasher plus narrow-index graft has never been attempted. The fingerprint ring was tested once in cycle 5 but has not been ported onto the current parent.

## Open questions

- How should the combined window be split between resident `small` and the ghost? Should `small + ghost` stay roughly constant as slots move from `small` to the ghost?
- How small can `small` get before keys reaching `main` with no frequency credit start to hurt hit rate?
- Does widening `main`'s frequency cap (retention) stack with the `small`/ghost split, or do the two overlap?
- Do the small, tuned gains hold at a second cache size or on another trace, or do they just fit workload-a's reuse distances?
- Do the fingerprint ring and the hasher plus narrow-index graft actually cut memory and per-op time once they can be measured? Can a single hash serve both the fingerprint and the lookup?
- Are there hit-rate levers outside admission (retention, eviction order inside `main`) that are worth more than the leftover admission tuning?

## Per-policy notes

- **S3-FIFO:** The lineage winner. Current parent: `small` at about a twentieth of capacity and the ghost at about an eighth. The fixed small split is better than adaptive sizing. Ghost bookkeeping has a correct sequence-number fix for stale erases, though the case it guards against almost never happens. Widening the frequency cap gave a small retention gain that is orthogonal to the window split.
- **CLOCK (plain):** Matches LRU's hit rate, so exact LRU ordering is not needed. Its only appeal would be lower memory or latency, which is unmeasured. A possible `put` sweep latency spike when most entries are marked referenced is unexamined. Deprioritized.
- **LIRS:** Beats LRU but trails S3-FIFO. Enlarging the HIR queue hurt. Its likely heavier nodes and a linear walk to enforce the ghost bound are unmeasured. Set aside until cost can be measured and the bound walk is O(1).
- **TinyLFU / count-min admission:** Fixing the leak through `small` and comparing against the real victim was not enough. Gains stayed marginal on top of S3-FIFO. Drop unless measurement shows it is nearly free.
- **ARC-style adaptivity:** Drifted away from the right split and lost hit rate. Dropped.
- **Strict LRU plus representation changes (Fx hashing, `u32` links, MRU fast path):** Policy-neutral and never measured. They remain a pending graft onto the S3-FIFO parent.
