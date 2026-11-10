# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-11-10T10:33:31+00:00

## timeline

### 10:33:58 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0010)\n\n## Recurring wins\n\n- **Strict admission into a full main.** The two-reuse graduation rule plus a freq reset has held from cycle 0001 to 0003. No change to the threshold, split, occupancy gate or credit has beaten it on every trace. Main fills once the cache warms and stays full, so the policy is decided by which keys get into main.\n- **Per-key evidence beats global rate signals.** Every lasting ghost-path gain used state the key carries with it: its small-queue freq when it went to ghost (0004), its ghost age (0005), and its hit pattern right after insertion (0009). Global signals never separated the traces: main occupancy, windowed ghost-return rate and unweighted ARC ghost hits.\n- **Recency gate on freq-0 ghost returnees, about one small-queue turnover wide (0005-v002).** This is the most robust policy step so far. It gained on both traces at the small size with no regressions. Probes at 2× and ½× both lost, so 1× is a local optimum.\n- **Deferring main admission rather than rejecting it (0009-v003 CRP cap).** Keys whose hits all fall in their insertion burst keep freq 1 into the ghost. They reach main later only if they come back after a real gap. This gained on hm_0 at both sizes, and it is the current base.\n- **Windows that scale with `small_size`** are the only mechanisms that have reached the large cache size. Fixed or tiny windows only ever moved the small size.\n- **Cost-only refactors keep behaviour identical.** The O(1) index/slab, fast hasher and 8-byte `Loc` all kept hit/miss bit-identical, and an oracle test enforces that. They can be ported onto any policy parent.\n\n## Recurring failures\n\n- **A single constant change cannot beat both traces.** For five straight cycles (0006–0010), almost every lever helped one of hm_0 and web_0 and hurt the other. The only exceptions were near no-ops. hm_0 rewards more probation, memory or horizon at a tight capacity. web_0 penalises any privilege for long-gap returns or exactly-twice keys.\n- **Static and adaptive small/main split tuning** (0001–0002, 0008) gave noise-level effects whose sign flipped across traces and sizes. It is weak even for web_0 at the large size.\n- **Rewarding ghost returnees in bulk:** flat credit (0003), extra laps in main (0007), age ceilings on freq ≥ 1 returnees (0006) and tiering ghost privileges by age (0009). A ghost return is weak evidence on web_0, and on hm_0 the direct-to-main path for freq ≥ 1 returnees is valuable.\n- **Stacking relaps on top of the ghost gate** (0005–0006). Freq-resetting relaps erase the signal the ghost gate reads. Relaps that keep that signal just duplicate the ghost path and cost small-queue capacity.\n- **Resizing ghost budgets:** uniform doubling (0008), and growing `main_ghost` toward `main_size` (0009). Both are exhausted axes. Anything that feeds the shared ghost FIFO or stamp clock also silently narrows the freq-0 window. Fixing the stamp clock did not recover the losses it seemed to explain.\n- **Tuning the CRP window by insertion distance** (0010). hm_0's burst repeats and web_0's real quick reuses fall in the same distance band.\n- **Per-op-time and metadata-size have been unmeasured for ten cycles.** Every hypothesis that assumed they were reported was wrong. Several cost variants are still waiting to be measured, and ghost-heavy variants have unknown metadata cost.\n\n## Open questions\n\n- **Recency-scoped main-victim readmission.** 0010-v003 was a gated main ghost that readmits returnees with freq 1. It gained on three of four cells and hurt only web_0 at the large size, which suggests the `main_hit` flag goes stale across second-chance passes. Does clearing `main_hit` on each decrement, or readmitting with freq 0 or via small, fix that cell and keep the other gains? This is the most promising thread.\n- Is there a signal available at first admission that separates hm_0's burst-only keys from web_0's real quick reusers? Candidates are the hit count inside the window and whether main is full at insertion. Or is the CRP cap's web_0 small-size cost inherent?\n- If the readmission fix also fails, is web_0 at the large size at its ceiling for this policy family?\n- How much do the pending cost-only changes actually save in per-op-time and metadata-size once those dimensions are measured? On 8-byte keys, alignment may cancel the ghost-deque saving.\n\n## Per-policy notes\n\n- **S3-FIFO lineage (only policy explored):** baseline → two-reuse rule plus freq reset with an occupancy gate (0002) → O(1) index (0003) → freq-gated ghost path (0004) → age-gated freq-0 returnees (0005-v002) → CRP cap (0009-v003, current base). In 0006–0008 the nominal winners were roughly neutral, and 0005-v002 remained an equally valid base. 0010's winner was a near no-op, so 0009-v003 is still the best policy.\n- **Gains by cell:** gains have come mostly at the small cache size. At the large size, web_0 has resisted ghost routing, ghost age, ghost budgets, the split, main-victim memory and burst filtering. It moved only in response to how main victims are readmitted, and then in the wrong direction (0010-v003).\n- **hm_0 vs web_0:** hm_0 has real reuse that gets squeezed out of a small queue, and also burst-only keys that pollute main. web_0's pollution comes from late one-off rereferences and exactly-twice keys, not from bursts, and its quick reuses are real.\n- **Ghost structure:** the ghost is about 3× the small queue, which inflates any ratio-based signal. Main victims and small victims need different horizons and should be kept in separate budgets and clocks.\n"
}
```

### 10:34:06 thinking



### 10:34:06 chat

I've written `cross-cycle-insights.md` from the ledger covering cycles 0001 to 0010, with the four sections in the required order.

- **Recurring wins:**
  - Strict two-reuse admission into a full main.
  - Signals carried by each key (its freq when it went to ghost, its ghost age, its hits right after insertion) rather than global rate signals.
  - The ghost-age window for keys that went to ghost unused (freq 0), sized to one small-queue turnover.
  - The correlated-reference cap from 0009-v003, which is still the best policy and the base.
- **Recurring failures:**
  - The hm_0 / web_0 trade-off: in every cycle from 0006 to 0010, each lever helped one trace and hurt the other, apart from near no-ops.
  - Tuning the small/main split, rewarding ghost returnees as a group, and stacking relaps.
  - Resizing ghost budgets, and tuning the correlated-reference window by distance from insertion.
  - Per-op-time and metadata-size have not been measured in any of the ten cycles.
- **Open questions:**
  - Whether making the `main_hit` flag reset with recency, or readmitting with freq 0, fixes 0010-v003's loss on web_0 at the large size. This is the most promising thread.
  - Whether any signal at first admission can separate hm_0's burst-only keys from web_0's real quick reuses.
  - Whether web_0 at the large size has hit its ceiling for this policy family.
  - How much the waiting cost-only changes actually save.
- **Per-policy notes:** the S3-FIFO lineage, which cells have moved, how the two traces differ, and how the ghost is structured.


---

- finished: 2026-11-10T10:34:06+00:00
- duration: 35.20s
- status: ok
- input_tokens: 4
- output_tokens: 2762
- cache_read_tokens: 43795
- cache_write_tokens: 16025
- cost_usd: 0.19331
