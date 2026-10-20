# Variant v002 Insight

## Helped

(No dimension improved. metadata-size did not move. `Entry` keeps its existing `freq: u8`, and no struct fields were added. `GHOST_READMIT_FREQ` is a compile-time constant, so both size rows are the same.)

## Hurt

- hit-rate: Starting ghost-readmitted keys at `freq = 1` counts the same reuse twice. The ghost hit already rewards the key by placing it directly in main, skipping small. The extra credit then forces `evict_main` to rotate the key past the tail once before it can be evicted. A readmitted key that is not reused again therefore keeps its main slot for a full extra CLOCK lap. During that lap, `evict_main` evicts some other entry whose `freq` has reached 0. Often that is an older readmit or a graduate whose `get`-earned credit has run out, and on this trace those turned out to be the more valuable keys to keep.
- per-op-time (`put`): Ghost readmits now enter main with nonzero frequency, so each `evict_main` call pops and re-pushes more entries before it finds a victim. The ghost-hit path and the graduate-into-full-main path in `evict_small_inner` both pay for this extra queue churn. The added `slot_mut(i).freq` store and the `put_with_readmit_freq` wrapper with `#[inline(always)]` are minor costs next to it.
- per-op-time (`get`, the summary): The `get` body is byte-identical to the parent's. This move cannot come from the policy change on the hit path. The likely cause is noise, or a code-layout side effect of the restructured, force-inlined `put`. As in the parent, a change to `put` or the policy cannot improve the headline `get` number, though it can still disturb it.

(The hypothesis asked for `small_size` settling to be reported via the in-file `report_small_size_settling_parent_vs_variant` test. That test's output was not available to this writer. So it is still unknown whether the hit-rate loss came through a shift in the adaptive split or only through main-queue residency.)

## Takeaway

Treating a ghost hit as stronger evidence than a `get`, and giving the key an unearned second-chance lap, lowered hit-rate on workload-a. Admitting the key straight to main is already enough reward. Extra frequency credit keeps one-time-reuse keys around longer than keys whose reuse was observed with real `get`s, and it adds rotation work to every main eviction. If ghost-readmit retention is tried again, it should be conditional: for example, grant credit only on a second ghost hit, or only while small is near its floor. A blanket starting `freq` is not the way to do it, and any retry should report the `small_size` trajectory so the split effect can be separated from the residency effect.
