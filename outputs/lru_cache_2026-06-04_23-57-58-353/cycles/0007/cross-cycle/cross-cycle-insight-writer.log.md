# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-06-04T22:54:42+00:00

## timeline

### 22:54:46 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first find the output directory, then write the file.

Let me look for the output directory structure.

### 22:54:46 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 22:54:46 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 22:54:49 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 22:54:49 tool: Bash

```json
{
  "command": "ls -la /home/yaniv/projects/cache-evolve/"
}
```

### 22:54:51 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/task.md"
}
```

### 22:54:53 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/cross-cycle"
}
```

### 22:55:18 thinking

The output directory is `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/cross-cycle`. I need to write `cross-cycle-insights.md` there.

Let me now synthesize the cross-cycle insights from all 7 cycles of the ledger.

**Recurring wins:**
1. Two-segment SLRU consistently outperforms plain LRU from cycle 1 onward — the protected/probationary partition is the most durable structural gain.
2. Applying the eviction signal at both eviction sites (protected demotion AND probationary eviction) rather than just one produces meaningful gains (cycle 4 showed this clearly).


3. Finer-grained signals consistently win out: IRR beat frequency counts in cycle 7, frequency counts outpaced plain LRU recency in the early cycles, and full-scan probationary eviction beat quarter-scan approaches.
4. Preserving counters and signals across segment transitions proves critical — whether it's counter halving on demotion or IRR preservation, maintaining continuity between segments is fundamental to performance.

**Recurring failures:**
1. Ghost admission never delivered gains despite testing across cycles 2-6 at various window sizes, from probationary capacity through full-capacity windows.
2. Frequency-aware demotion with reset counters and bounded scan windows consistently underperformed, suggesting the approach itself has structural limitations.
3. Fixed-rate decay and logical clocks proved too workload-sensitive and actually eroded the distinctions they were meant to create.
4. Mechanisms that restrict node position updates — like the second-chance bit requiring position freezes — fail because continuous move-to-tail updates are essential.
5. Expanding scan fractions beyond the necessary range shows minimal benefit, as confirmed when testing protected segments at 1/3 versus 1/4.

**Open questions:**
1. Whether IRR-based eviction generalizes beyond LIRS workloads remains untested; I need to evaluate this across diverse workload patterns.
2. The 80/20 protected-to-probationary split may not be optimal under IRR, and adaptive segment-ratio balancing hasn't been explored yet.
3. Current scan windows (1/4 protected, 1/2 probationary) might need tighter calibration if IRR provides a sharper signal.
4. A second-chance approach could work if the referenced bit allows move-to-tail instead of freezing position.
5. Frequency-aware demotion might interact neutrally with IRR timestamps rather than regress.

**Per-policy notes:**
- **Plain LRU (baseline)**: Scan pollution is a real vulnerability, and every variant improves on it, so baseline comparisons show consistent 0% lift.
- **CLOCK**: Removes linked-list pointer overhead, though throughput gains haven't been measurable.
- **Frequency-window scan**: Limited by eviction-time-only signals; improving quality requires placement decisions at insertion, not just eviction.
- **SLRU (80/20)**: The 80/20 split has held since cycle 1 after testing showed 70/30 hurt performance with the larger working set, and adaptive balancing remains untested.
- **SLRU + ghost admission**: Ghost sets contributed negligibly in isolation testing, and multiple window sizes yielded no improvement.
- **SLRU + frequency-aware demotion + counter halving**: The frequency-scanning approach across both eviction sites with MAX_FREQ capping represents the ceiling of this design direction.
- **SLRU + IRR timestamps**: The biggest single-cycle improvement came from replacing counter halving with IRR timestamp preservation across demotion, eliminating the ghost set without performance loss and establishing the new baseline.

### 22:55:18 chat

Now I have the output directory. Let me write the file:

### 22:55:51 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Two-segment SLRU is the load-bearing structural gain.** Every winning baseline since cycle 1 has been built on protected/probationary segmentation; no cohort has found a way to match it without it.\n- **Applying the eviction signal at both segment boundaries pays off.** Cycle 4 confirmed that frequency-aware demotion in protected while using blind LRU in probationary left a gap; closing the asymmetry produced a clear gain. The pattern generalised to IRR in cycle 7.\n- **Signal preservation across demotion is load-bearing.** Counter halving on demotion (cycles 3–6) and IRR-timestamp preservation across demotion (cycle 7) both showed that continuity of the warmth signal across the segment boundary is required for the signal to remain useful in probationary.\n- **Sharper signals consistently dominate coarser ones.** IRR (last-two-accesses recency gap) beat accumulated frequency counts in cycle 7 by a wider margin than frequency counts beat plain LRU recency in cycle 1. The LIRS-style trace rewards temporal clustering, and policy quality tracks how precisely the signal captures that.\n- **Neutral wins are real convergence signals.** Two or three consecutive cohorts where the best variant advances on neutrality (cycles 5–6) reliably indicated the policy had reached a local optimum before IRR broke the plateau.\n\n## Recurring failures\n\n- **Ghost admission across every window size tested.** Probationary-sized window (cycles 2–3): negligible lift, confirmed negligible in cycle 3 isolation. Half-total-capacity (cycle 6): regressed. Full-capacity (cycle 4): regressed harder. Reuse events at distances beyond probationary capacity on this workload are cold traffic, not deferred-warm traffic; window resizing cannot rescue the mechanism on this trace.\n- **Eviction-site-only frequency mechanisms.** The cycle-1 frequency-window scan was bounded by its placement: acting only at eviction time, after warm entries were already candidates, is structurally weaker than acting at insertion (SLRU). The initial implementation also failed from a small fixed window and counter reset; both flaws are now understood.\n- **Fixed-rate frequency decay.** Logical-clock halving at a fixed half-life (cycle 6) collapses the frequency distinction between demoted-protected entries and cold newcomers whenever the half-life is shorter than the workload's median inter-access gap. Any fixed constant is likely wrong without workload-specific tuning.\n- **Mechanisms that suppress or freeze node position updates.** The cycle-3 second-chance bit that froze protected-segment node positions broke LRU ordering and hurt performance. Continuous move-to-tail on every access is load-bearing; any reprieve mechanism must preserve it.\n- **Scan-fraction widening beyond the established thresholds.** Widening the protected scan from 1/4 to 1/3 (cycle 5) and various wider probationary scans produced negligible or no returns, indicating frequency variation near the LRU end is already captured by the narrower window on this trace.\n\n## Open questions\n\n- **Does IRR-based eviction transfer beyond the LIRS trace?** All gains were measured on a single workload. Multi-workload evaluation is overdue and is the clearest way to distinguish broadly strong policy from workload-specific alignment.\n- **Is the 80/20 protected/probationary split still optimal under IRR?** The split has been fixed since cycle 1 and stress-tested only for frequency-based policies. Adaptive segment-ratio balancing (H01) has never been evaluated and is now more interesting on the IRR baseline.\n- **Are the current scan-window fractions well-calibrated for IRR?** IRR is a sharper discriminator than frequency counts; narrower scan fractions might yield equivalent eviction quality with less overhead. This is unexplored on the new baseline.\n- **Can a second-chance reprieve be salvaged without position-freeze?** The cycle-3 failure was caused by suppressing move-to-tail, not by the reprieve concept itself. Setting a referenced bit while still performing the position update was not tested.\n- **Is there a fixed decay constant that preserves IRR distinctions on workload-a?** It is unclear whether the decay-collapsing problem is fundamental on this trace or specific to the cycle-6 half-life choice; it remains untested on the IRR baseline.\n\n## Per-policy notes\n\n- **Plain LRU**: Scan-pollution vulnerability confirmed real and consistently exploitable. Serves only as the initial reference baseline.\n- **CLOCK (cycle 1, v002)**: Eliminates linked-list pointer maintenance; performed between LRU and SLRU. Whether its structural simplification yields throughput advantages under high concurrency has never been measured.\n- **SLRU 80/20 (cycles 1–7 baseline)**: Durable foundation. Widening probationary to 30% hurt in cycle 2 — the working set is large enough that the smaller protected segment causes more churn than the extra scan-filtering headroom saves. Ratio has not been revisited since.\n- **SLRU + ghost admission**: Marginal contribution confirmed negligible in cycle-3 isolation test. Dropped from the baseline after cycle 7 with no regression.\n- **SLRU + frequency-aware demotion + counter halving (cycles 3–6)**: Full design closed in cycle 4 (symmetric eviction signal at both sites) and capped in cycle 5 (MAX_FREQ=8 guardrail, neutral). Three consecutive neutral-winner cohorts confirmed workload-a convergence. This policy line is exhausted on this trace.\n- **SLRU + IRR timestamps (cycle 7, current baseline)**: Largest single-cycle hit-rate gain in the run. IRR preserved across demotion replaces counter halving naturally; cold newcomers arrive with IRR = MAX and lose the eviction contest without manual counter adjustment. Ghost set dropped. Scan fractions, segment ratio, and multi-workload behaviour all remain untested on this baseline.\n"
}
```

### 22:55:59 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what was synthesised across the seven cycles:

**Recurring wins** — SLRU segmentation is the durable structural foundation; signal preservation across the segment boundary (counter halving, then IRR timestamps) has been load-bearing in every strong iteration; applying the eviction signal at both sites rather than just one closed a consistent gap; sharper signals beat coarser ones each time they've been tested.

**Recurring failures** — Ghost admission is exhaustively dead across every window size (probationary, half-total, full capacity); eviction-site-only frequency mechanisms are structurally weaker than insertion-time filtering; fixed-rate frequency decay collapses the very distinctions it was meant to sharpen; anything that freezes node positions breaks LRU ordering.

**Open questions** — IRR transferability to non-LIRS workloads is the most urgent unknown; adaptive segment-ratio balancing (H01) has never been evaluated despite being identified in cycle 7; scan-fraction calibration under IRR is untested; a salvaged second-chance reprieve that preserves move-to-tail was never tried.

**Per-policy notes** — traces the trajectory from plain LRU → CLOCK → SLRU → ghost variants → frequency-aware demotion (exhausted at cycle 6) → IRR timestamps (current baseline, many parameters still untested).


---

- finished: 2026-06-04T22:55:59+00:00
- duration: 77.67s
- status: ok
- input_tokens: 7
- output_tokens: 3363
- cache_read_tokens: 106415
- cache_write_tokens: 10856
- cost_usd: 0.128593
