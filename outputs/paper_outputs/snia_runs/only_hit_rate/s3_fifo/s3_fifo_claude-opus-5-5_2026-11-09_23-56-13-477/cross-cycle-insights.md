# Cross-Cycle Insights (through cycle 0010)

## Recurring wins

- **Strict admission into a full main.** The two-reuse graduation rule plus a freq reset has held from cycle 0001 to 0003. No change to the threshold, split, occupancy gate or credit has beaten it on every trace. Main fills once the cache warms and stays full, so the policy is decided by which keys get into main.
- **Per-key evidence beats global rate signals.** Every lasting ghost-path gain used state the key carries with it: its small-queue freq when it went to ghost (0004), its ghost age (0005), and its hit pattern right after insertion (0009). Global signals never separated the traces: main occupancy, windowed ghost-return rate and unweighted ARC ghost hits.
- **Recency gate on freq-0 ghost returnees, about one small-queue turnover wide (0005-v002).** This is the most robust policy step so far. It gained on both traces at the small size with no regressions. Probes at 2× and ½× both lost, so 1× is a local optimum.
- **Deferring main admission rather than rejecting it (0009-v003 CRP cap).** Keys whose hits all fall in their insertion burst keep freq 1 into the ghost. They reach main later only if they come back after a real gap. This gained on hm_0 at both sizes, and it is the current base.
- **Windows that scale with `small_size`** are the only mechanisms that have reached the large cache size. Fixed or tiny windows only ever moved the small size.
- **Cost-only refactors keep behaviour identical.** The O(1) index/slab, fast hasher and 8-byte `Loc` all kept hit/miss bit-identical, and an oracle test enforces that. They can be ported onto any policy parent.

## Recurring failures

- **A single constant change cannot beat both traces.** For five straight cycles (0006–0010), almost every lever helped one of hm_0 and web_0 and hurt the other. The only exceptions were near no-ops. hm_0 rewards more probation, memory or horizon at a tight capacity. web_0 penalises any privilege for long-gap returns or exactly-twice keys.
- **Static and adaptive small/main split tuning** (0001–0002, 0008) gave noise-level effects whose sign flipped across traces and sizes. It is weak even for web_0 at the large size.
- **Rewarding ghost returnees in bulk:** flat credit (0003), extra laps in main (0007), age ceilings on freq ≥ 1 returnees (0006) and tiering ghost privileges by age (0009). A ghost return is weak evidence on web_0, and on hm_0 the direct-to-main path for freq ≥ 1 returnees is valuable.
- **Stacking relaps on top of the ghost gate** (0005–0006). Freq-resetting relaps erase the signal the ghost gate reads. Relaps that keep that signal just duplicate the ghost path and cost small-queue capacity.
- **Resizing ghost budgets:** uniform doubling (0008), and growing `main_ghost` toward `main_size` (0009). Both are exhausted axes. Anything that feeds the shared ghost FIFO or stamp clock also silently narrows the freq-0 window. Fixing the stamp clock did not recover the losses it seemed to explain.
- **Tuning the CRP window by insertion distance** (0010). hm_0's burst repeats and web_0's real quick reuses fall in the same distance band.
- **Per-op-time and metadata-size have been unmeasured for ten cycles.** Every hypothesis that assumed they were reported was wrong. Several cost variants are still waiting to be measured, and ghost-heavy variants have unknown metadata cost.

## Open questions

- **Recency-scoped main-victim readmission.** 0010-v003 was a gated main ghost that readmits returnees with freq 1. It gained on three of four cells and hurt only web_0 at the large size, which suggests the `main_hit` flag goes stale across second-chance passes. Does clearing `main_hit` on each decrement, or readmitting with freq 0 or via small, fix that cell and keep the other gains? This is the most promising thread.
- Is there a signal available at first admission that separates hm_0's burst-only keys from web_0's real quick reusers? Candidates are the hit count inside the window and whether main is full at insertion. Or is the CRP cap's web_0 small-size cost inherent?
- If the readmission fix also fails, is web_0 at the large size at its ceiling for this policy family?
- How much do the pending cost-only changes actually save in per-op-time and metadata-size once those dimensions are measured? On 8-byte keys, alignment may cancel the ghost-deque saving.

## Per-policy notes

- **S3-FIFO lineage (only policy explored):** baseline → two-reuse rule plus freq reset with an occupancy gate (0002) → O(1) index (0003) → freq-gated ghost path (0004) → age-gated freq-0 returnees (0005-v002) → CRP cap (0009-v003, current base). In 0006–0008 the nominal winners were roughly neutral, and 0005-v002 remained an equally valid base. 0010's winner was a near no-op, so 0009-v003 is still the best policy.
- **Gains by cell:** gains have come mostly at the small cache size. At the large size, web_0 has resisted ghost routing, ghost age, ghost budgets, the split, main-victim memory and burst filtering. It moved only in response to how main victims are readmitted, and then in the wrong direction (0010-v003).
- **hm_0 vs web_0:** hm_0 has real reuse that gets squeezed out of a small queue, and also burst-only keys that pollute main. web_0's pollution comes from late one-off rereferences and exactly-twice keys, not from bursts, and its quick reuses are real.
- **Ghost structure:** the ghost is about 3× the small queue, which inflates any ratio-based signal. Main victims and small victims need different horizons and should be kept in separate budgets and clocks.
