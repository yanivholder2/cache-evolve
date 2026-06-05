# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-04T23:12:19+00:00

## timeline

### 23:12:23 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first figure out where the output directory is, then write the file.

Looking at the system context, the project is at `/home/yaniv/projects/cache-evolve`. I need to find where the output directory would be for this framework.

Let me look at the project structure to find the output directory.

### 23:12:24 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 23:12:24 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 23:12:26 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/"
}
```

### 23:12:28 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve"
}
```

### 23:12:30 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/cross-cycle"
}
```

### 23:12:59 thinking

The output directory is `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/cross-cycle`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize all 8 cycles into the four required sections.

**Recurring wins:**
- Two-segment SLRU (protected + probationary) consistently outperforms single-segment LRU — established in cycle 1, reinforced ever since
- Frequency-aware eviction (then IRR-aware) applied at BOTH eviction sites (not just one) — cycle 4 closed the asymmetry, cycle 8 confirmed it holds under IRR too


- Counter/signal preservation across segment transitions is critical — halving frequency on demotion and preserving timestamps for IRR both prevent performance degradation
- Probationary half-scan consistently finds entries worth keeping since they're distributed throughout, not clustered at the LRU head — validated under both frequency and IRR policies
- Protected quarter-scan proves sufficient to capture the relevant signal under both policy regimes
- The 80/20 protected-to-probationary ratio remains stable across all tested policies; expanding probationary beyond 20% consistently hurts performance

**Recurring failures:**
- Ghost admission: tested extensively across multiple configurations (probationary capacity, isolation, full capacity, half-capacity, and with IRR) but never improved performance — no longer a viable direction
- Widening probationary: regressed under both frequency and IRR baselines; the working set density requires 80% protected as a minimum threshold
- Fixed-rate logical-clock decay: destroyed the frequency distinction needed for effective scanning; any constant decay rate is too workload-dependent
- Second-chance reprieve bit: breaking recency ordering by freezing node position eliminated its utility, and removing the freeze didn't restore benefit since the trace doesn't generate the conditions needed

Narrowing scan-fraction in probationary failed under both frequency and IRR baselines, while widening it in protected showed minimal gains. I'm now considering whether a hybrid approach combining IRR with a coarse visit count could preserve long-term warmth signals without reintroducing staleness, and whether the 80/20 segment split remains optimal when policy quality improves. The core uncertainty is whether IRR's performance gains are specific to this trace or generalizable across different workloads, and whether adaptive segment-ratio balancing could have helped in earlier cycles.

I'm also questioning whether scan fractions tied to live hit-rate feedback could break through the plateau we're seeing, or if the inter-access distance distribution makes any fixed decay constant fundamentally mismatched to this workload. Looking at the policy comparisons, SLRU's 80/20 split proved robust as the structural foundation, while CLOCK's eviction-time-only signal underperformed compared to insertion-time routing, and frequency-aware approaches with halving-on-demotion and persistent counts showed mixed results depending on scan configuration.

The frequency-aware variants recovered in cycles 3–6 after fixing the scan fraction and counter reset logic, then closed the eviction asymmetry in cycle 4 before hitting a local optimum on workload-a. IRR-based SLRU in cycle 7 delivered the largest single-cycle improvement by treating recency-of-recency as the eviction signal instead of total count, eliminating the need for halving-on-demotion or ghost sets entirely. Ghost admission across various window ranges and underlying policies never produced a clean isolated win and appears fully exhausted as an avenue.

### 23:13:25 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Two-segment SLRU (80/20 protected/probationary) is the load-bearing foundation.** Every winning variant since cycle 1 rests on this structure; single-segment LRU is uniformly worse across all tested policy axes.\n- **Symmetric signal application at both eviction sites.** Applying the eviction signal (frequency or IRR) only in the protected segment while leaving probationary as blind LRU always left performance on the table; closing the asymmetry (cycle 4) was a consistent gain that held when the signal changed to IRR (cycle 8).\n- **Signal preservation across segment transitions.** Counter halving on demotion (frequency era) and timestamp preservation across demotion (IRR era) both proved load-bearing; resetting or discarding the signal at the segment boundary degraded quality in every test.\n- **Probationary half-scan is consistently necessary.** Eviction-worthy entries are distributed through probationary, not compressed near the LRU head — confirmed under both the frequency baseline (cycle 5) and the IRR baseline (cycle 8). The equivalent quarter-scan suffices for protected under both policy regimes.\n- **IRR timestamps produced the largest single-cycle gain in the series (cycle 7).** Switching from cumulative access counts to inter-reference recency broke out of a three-cycle neutral-winner plateau, confirming that recency-of-recency is a sharper eviction signal than total frequency on this workload.\n\n## Recurring failures\n\n- **Ghost admission is exhausted.** Tested at probationary capacity (cycle 2 — gain was SLRU, not ghost), isolated on the frequency baseline (cycle 3 — negligible), at full capacity (cycle 4 — regressed), at half-total capacity (cycle 6 — regressed), and with IRR (cycle 8 — regressed). The reuse distances on this workload do not produce a reliable ghost-intercept signal at any window size tested.\n- **Widening probationary beyond 20% always regresses.** Confirmed independently on the frequency baseline (cycle 2) and the IRR baseline (cycle 8). The active working set is dense enough that an 80% protected segment is a hard floor; any larger probationary triggers excess demotion churn.\n- **Fixed-rate counter decay collapses the eviction signal.** A fixed logical-clock half-life (cycle 6) erased the distinction between demoted-warm and cold-new entries, the exact distinction the scan depends on. The right decay constant is workload-specific; no fixed value is likely to generalize.\n- **Scan-fraction narrowing in probationary always fails.** Reducing from half to quarter in probationary degraded quality under both frequency (cycle 5) and IRR (cycle 8) baselines — IRR variation is not compressed near the LRU head.\n- **Second-chance reprieve bits are inert or harmful.** Freezing node position on protected hits destroyed recency ordering (cycle 3). When the position-freeze was removed (cycle 6), the referenced-bit tie-breaker became harmless but also inert — equal-frequency collisions targeted by the mechanism do not occur at the current scan window width on this trace.\n- **Scan-fraction widening in protected yields diminishing returns.** Quarter-to-third expansion in protected (cycle 4) produced marginal movement; going further is not a productive direction on this workload.\n\n## Open questions\n\n- **Hybrid IRR-plus-visit-count signal.** Pure IRR discards long-term warmth history that frequency-halving preserved across demotion cycles; a coarse visit counter layered onto IRR might recover that without reintroducing staleness from unbounded accumulation.\n- **Multi-workload evaluation is overdue.** All wins and failures are anchored to a single LIRS-generated trace. It is unknown whether the IRR gain is broadly transferable or specifically aligned to this trace's reuse-distance distribution. This is the highest-value open question.\n- **Adaptive segment-ratio balancing (H01).** Never evaluated despite being nominated in cycles 7 and 8. With a sharper eviction signal in place, the optimal protected/probationary ratio might differ from the inherited 80/20; live hit-rate feedback could drive the split dynamically.\n- **Adaptive scan fractions.** Fixed fractions (quarter, half) have been validated empirically but not tuned to workload shape. Fractions proportional to observed reuse-distance distributions might yield equivalent eviction quality at lower scan overhead, especially on the IRR baseline where the signal is sharper.\n\n## Per-policy notes\n\n- **Baseline LRU:** scan-pollution vulnerability confirmed cycle 1 and never recovered; all variants across all cycles improved over it. Serves only as a reference floor.\n- **CLOCK:** outperformed by SLRU in cycle 1; avoids linked-list pointer maintenance but acts at eviction time, a weaker intervention point than SLRU's insertion-time routing. Not revisited.\n- **SLRU + frequency-aware eviction (cycles 3–6):** rescued from an early design failure (too-small scan window, counter reset on demotion) in cycle 3; peak form is two-segment 80/20, halving-on-demotion, quarter-scan in protected, half-scan in probationary, MAX_FREQ cap. Hit a local optimum by cycle 5; three consecutive neutral-winner cycles confirmed it.\n- **SLRU + IRR timestamps (cycles 7–8):** current frontier baseline; replaces halving-on-demotion with timestamp preservation naturally, no ghost set. Scan fractions and segment ratio confirmed at the same values as the frequency baseline. Ghost admission incompatible. Plateau not yet confirmed — H01 and multi-workload evaluation remain.\n- **Ghost admission (all cycles):** never produced an isolated win; effect is always negative or negligible when separated from the structural SLRU gain. Fully mapped across the meaningful window range on two underlying policies.\n"
}
```

### 23:13:31 chat

Written to `/home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0008/cross-cycle/cross-cycle-insights.md`.

The file synthesizes all eight cycles into the four required sections:

- **Recurring wins** — five durable patterns: SLRU 80/20 as the structural foundation, symmetric signal application at both eviction sites, signal preservation across segment transitions, the necessity of the probationary half-scan, and IRR as the sharpest eviction signal yet recorded.
- **Recurring failures** — six exhausted directions: ghost admission (fully mapped across all window sizes and both policies), probationary widening (regressed on both frequency and IRR baselines), fixed-rate decay, probationary scan narrowing, second-chance reprieve bits, and protected scan widening.
- **Open questions** — four threads worth pursuing: a hybrid IRR-plus-visit-count signal, multi-workload evaluation (the highest-value open question), adaptive segment-ratio balancing (H01, still untested), and adaptive scan fractions.
- **Per-policy notes** — observations grouped by eviction policy across the full series, from baseline LRU through CLOCK, SLRU+frequency, SLRU+IRR, and ghost admission.


---

- finished: 2026-06-04T23:13:32+00:00
- duration: 72.82s
- status: ok
- input_tokens: 7
- output_tokens: 3431
- cache_read_tokens: 110163
- cache_write_tokens: 11646
- cost_usd: 0.134462
