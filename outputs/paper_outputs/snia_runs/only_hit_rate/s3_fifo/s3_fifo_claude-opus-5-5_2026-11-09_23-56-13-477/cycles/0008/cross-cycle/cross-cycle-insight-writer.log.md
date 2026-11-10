# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-11-10T10:19:35+00:00

## timeline

### 10:19:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0008)\n\n## Recurring wins\n\n- **Per-key evidence beats global signals.** Every real policy gain (0004 freq-gated ghost path, 0005 age-gated freq-0 returnees) came from evidence the key itself carries: its small-queue freq when it was ghosted, and its ghost age. Global rate or occupancy signals never separated the two traces.\n- **Strict admission into a full main.** The two-reuse graduation rule plus the freq reset has held up since 0002. Loosening admission once the cache is warm keeps costing hit-rate.\n- **About one small-queue turnover is the right freq-0 recency window.** It is bracketed by losses at 2× (0006) and ½× (0007), so it is a confirmed local optimum.\n- **Cost-only refactors are safe to port.** The O(1) index/slab, fast hasher and compact `Loc` all kept hit/miss behaviour bit-identical, and an oracle test enforces it, so they can be layered onto any policy parent.\n- **Windows that scale with `small_size` reach the large cache size**, whereas fixed-scope tweaks only ever moved the small size.\n- **A ghost record's useful content is how long ago the key was seen and where it was evicted from (small or main).** Treating every ghost hit the same loses this information.\n\n## Recurring failures\n\n- **Single-constant tweaks to the threshold or the small/main split** (0001, 0002, 0008). They always trade hm_0 against web_0, and their effect flips sign across cache sizes.\n- **Adaptive gates driven by global signals.** Main occupancy saturates (0002). A windowed ghost-return rate fired on the wrong traces and was inflated by the ghost/small size ratio (0003). An unweighted ARC-style split drifts toward a larger small queue.\n- **Flat rewards on the ghost path.** Flat second-chance credit for returnees (0003) and age ceilings on freq ≥ 1 returnees (0006) both lost hit-rate.\n- **Small-queue relaps.** Relaps that reset freq erased the ghost signal (0005). Relaps that kept freq-1 just duplicated the ghost path and cost small capacity (0006).\n- **Anything that moves other entries ahead of quick freq-0 returnees in main** (0007 main-admission freq boost). These returnees are a valuable population.\n- **New ghost producers on the shared ghost FIFO.** They shrink the effective freq-0 window. Restoring the stamp clock did not fix the related small-size loss (0008), so don't retry clock fixes.\n- **Uniform ghost-budget scaling** (0008 doubled ghost). The gain on hm_0 at the small size was offset by web_0 losses on long-gap returnees, and it doubles metadata.\n- **Per-op-time and metadata-size have been unmeasured for eight cycles running.** Several cost-only variants are still waiting on measurement, and continuation requests to measure them were repeatedly not acted on.\n\n## Open questions\n\n- Does a decoupled main-victim ghost with a budget near `main_size` keep web_0's large-size gain while protecting the freq-0 window?\n- Does a tiered extended ghost zone help hm_0 at the small size without the web_0 losses? Old records would give freq-0 returnees the freq-1 small path but would not grant direct entry to main.\n- If both of those work, do they compose?\n- What limits web_0 at the large size? Ghost routing and the small/main split have been ruled out. First-admission handling of never-seen keys is still unprobed, and main-victim memory is the only lever that has moved this cell.\n- How much do the cost-only changes actually save in per-op-time and metadata-size, especially as ghost structures grow?\n\n## Per-policy notes\n\n- **S3-FIFO lineage (the only policy studied).** The current policy base is the age-gated ghost path from 0005 v002 (window at 1×), with 0008 v003 as a nominally better but roughly neutral alternative. 0007 v001's main-victim ghost is worth keeping only in its decoupled form.\n- **The two traces pull in opposite directions.** hm_0 rewards more probation, longer memory and bigger ghost horizons at tight capacity. web_0 punishes privileges for keys that return after a long gap and for keys accessed exactly twice.\n- **Gains are mostly at the small cache size.** At the large size the ghost already covers a long reuse window, so routing changes barely change which key gets evicted. web_0 at the large size is the persistent plateau.\n- **Small-queue displacement is costly.** Every re-admission into small pushes out another resident, and at the small cache size this keeps costing more than the main pollution it avoids.\n- **Main stays full in steady state.** Any extra protection given to one population evicts another, so admission decisions matter more than what happens to an entry once it is in main.\n"
}
```

### 10:20:04 thinking



### 10:20:04 chat

I've written `cross-cycle-insights.md` to the cycle 0008 cross-cycle directory. It uses the four required sections in order and draws only on the ledger for cycles 0001–0008.

- **Recurring wins:**
  - Decisions based on evidence each key carries (its small-queue freq and its ghost age) beat global rate or occupancy signals.
  - Strict two-reuse admission into a full main has held up since cycle 0002.
  - A freq-0 recency window of one small-queue turnover is a confirmed local optimum: 2× lost in 0006 and ½× lost in 0007.
  - The cost-only refactors are safe to port because they leave hit/miss behaviour bit-identical.
  - Windows that scale with `small_size` are the only changes that reached the large cache size.
- **Recurring failures:**
  - Single-constant tweaks to the threshold or the small/main split.
  - Gates driven by global signals.
  - Flat rewards for keys returning from the ghost.
  - Small-queue relaps.
  - Changes in main that put other entries ahead of quick freq-0 returnees.
  - Uncoupled ghost producers and uniform ghost-budget scaling.
  - Per-op-time and metadata-size have now gone unmeasured for eight cycles running.
- **Open questions:**
  - A separate main-victim ghost with a budget near `main_size`.
  - A tiered extended ghost zone where old records don't grant direct entry to main.
  - Whether those two compose if each works.
  - What limits web_0 at the large size; first admission of never-seen keys is still unprobed.
  - How much the cost changes actually save.
- **Per-policy notes:** S3-FIFO is the only policy studied. hm_0 and web_0 pull in opposite directions, gains have mostly been at the small cache size, and web_0 at the large size remains the one cell that won't move.


---

- finished: 2026-11-10T10:20:04+00:00
- duration: 28.82s
- status: ok
- input_tokens: 4
- output_tokens: 2413
- cache_read_tokens: 41846
- cache_write_tokens: 13660
- cost_usd: 0.166826
