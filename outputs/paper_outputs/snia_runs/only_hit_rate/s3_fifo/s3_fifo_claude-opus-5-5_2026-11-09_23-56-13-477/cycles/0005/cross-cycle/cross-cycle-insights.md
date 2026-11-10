# Cross-Cycle Insights

## Recurring wins

- Per-key evidence works better than global signals. Two cycles in a row (0004 and 0005) improved by gating the ghost→main path on what each key carried with it: first its small-queue freq at ghost time, then its ghost age. Both gains were small but real, and the age gate is the first to gain on both traces with no losing cell.
- What reliably moves hit-rate is which keys get into main, and how strictly. Every cycle has kept the strict two-reuse rule plus the freq reset as the floor. No looser admission rule has beaten it on all traces.
- Filtering in small or ghost costs less than filtering at main admission. Main stays full, so any key admitted or protected there pushes out an entry that would have been hit.
- Windows that scale with `small_size` can reach the large cache size. Fixed-size mechanisms only moved results at the small size.
- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) have kept hit/miss behaviour bit-identical every time. An oracle test enforces this, so they can safely be ported onto whichever policy parent is current.

## Recurring failures

- Changing a single constant for the threshold or the small/main split. hm_0 and web_0 react in opposite directions, and the effect flips sign across cache sizes, so a static tweak doesn't win on both (cycles 0001–0002).
- Global rate or occupancy signals don't work as adaptive gates:
  - Main occupancy fills and stays full, so the gate falls back to the baseline rule.
  - The windowed ghost-return rate is inflated by the ghost being much larger than the small queue, and it turned lenient on exactly the wrong traces.
  - An unweighted ARC-style split tends to drift toward a larger small queue.
- Giving returnees flat credit, or extra protection in main. Ghost returns are mostly weak single-reuse keys, so the credit keeps dead keys in main for an extra lap.
- Mechanisms that reset freq and erase the per-key signal the ghost gate reads, such as cycle 0005's relap: a failed relap loses its direct return to main.
- Process failure: per-op-time and metadata-size have been missing from the fitness vector for five cycles running, even though hypotheses kept assuming they were reported. Several cost-only variants are still waiting on measurement.

## Open questions

- How much do the cost changes actually save in per-op-time, and what do they do to metadata-size? On 8-byte keys, alignment may cancel some layout savings. None of this can be answered until the cost dimensions are in the evaluation.
- Sweeping the age window (`GHOST_RECENT_AGE_DIVISOR`): can it move web_0 at the large size, the one cell no policy variant has moved?
- Does a relap stack with the age gate if failed relaps keep their freq-1 ghost signal or a relapped mark?
- Should a key's second reuse in small count only if it is recent? Reuses spread over a long window looked like weak evidence.
- Could a direct comparison of the misses avoided by promoting against the main entries displaced by promoting beat any proxy?
- Adaptive variants never logged where their controlled values settled (`small_size`, threshold), so it's still unclear whether those controllers actually moved.

## Per-policy notes

- **S3-FIFO (sole policy so far)**: the current base is the O(1) index, the strict two-reuse graduation rule with freq reset, and a ghost→main path gated on each key's ghost-time freq and ghost age (cycle 0005 winner). The cost-only changes still need porting onto it.
- **Trace behaviour**:
  - hm_0 holds real reuses that get squeezed out of the small queue, so a quick ghost return is a meaningful signal there.
  - web_0 is dominated by keys accessed exactly twice and by late one-off rereferences, so it punishes lenient admission.
- **Cache size**: policy gains concentrate at the small size. At the large size the ghost already covers long reuse windows, so most filtering changes do little there unless their window scales with the queue.
- **Gain size**: improvements in this lineage are small. Treat near-noise changes in the small/main split as no change.
