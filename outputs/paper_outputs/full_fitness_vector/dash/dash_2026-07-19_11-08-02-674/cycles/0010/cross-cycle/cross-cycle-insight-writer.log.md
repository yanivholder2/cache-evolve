# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\cross-cycle
- allow_edits: True
- started: 2026-07-21T07:10:47+00:00

## timeline

### 07:10:53 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with exactly four sections.

Let me first find where the output directory is for this project.

### 07:10:53 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 07:10:53 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 07:10:56 tool: Bash

```json
{
  "command": "ls /",
  "description": "List root directory"
}
```

### 07:10:56 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 07:10:58 tool: Read

```json
{
  "file_path": "/task.md"
}
```

### 07:11:01 thinking

The output directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0010\cross-cycle`. I should write the file there.

### 07:11:52 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0010\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **MRU-end insertion over LRU-end insertion** delivered a simultaneous hit-rate and per-op latency improvement in Cycle 1 and has served as a stable anchor in every subsequent variant; no tested combination has reversed this gain.\n- **Per-item struct compaction** (timestamp/`Instant` field removal) reduced memory footprint, eliminated clock syscalls, and improved cache-line utilization during bucket traversal — a pure multiple-dividend win that compounds other improvements by making bucket scans cheaper.\n- **Vacancy-gated promotion** (fire only when the target tier has an open slot) eliminated cascade evictions by construction in Cycle 3 and has been a regression-free contributor in every stack it appeared in; structural prevention outperformed threshold-rate suppression on every head-to-head comparison.\n- **Orthogonal mechanisms stack additively.** Across Cycles 1–3 and again in Cycle 6, pairs of independently validated wins combined without interaction regressions, provided they do not share a mutable counter field.\n- **Two-chance clock eviction** (Cycle 6) solved the O(n) LFU-minimum scan latency problem by providing amortized O(1) cold-victim selection on Zipfian traces where cold-tail items accumulate near the front of the rotation; it improved all three measured dimensions simultaneously from the baseline.\n\n## Recurring failures\n\n- **O(n) per-eviction scans produce consistent tail-latency regressions.** LFU-minimum scanning of the full bucket was validated in Cycles 5–6 as a hit-rate lever but paid a put-path p99 cost every time it was applied unconditionally; the clock was introduced specifically to avoid this, and the stash cold-eviction scan (Cycle 8) avoids it only because stash eviction fires infrequently.\n- **Counter decay on a stationary workload is antagonistic.** Post-eviction counter halving (Cycle 6 v002) undermined the frequency-gated promotion that accumulated those counts, adding an extra scan and doubling median latency with no benefit; decay solves stale-popularity drift that does not exist on a stationary Zipfian distribution.\n- **The clock and `lfu_counter`-based promotion gates share a field they cannot simultaneously own.** The clock requires counter-zero cold items as eviction targets; the promotion gate requires non-zero counters on warm items. Every attempt to combine them through a shared `lfu_counter` field has degraded either hit-rate (clock victims have earned promotion), latency (clock rotates O(k·n) on a counter-rich base), or both (Cycles 7–10). Simple integer thresholds (freq≥1, ≥2, ≥3) have each been falsified: ≥1 saturates the warm set; ≥2 reproduces saturation on Zipfian distributions where most items hit exactly twice; ≥3 spares hot-bucket rotation depth but leaves the working set undertrained.\n- **Stash-tier interventions (bypass, cold eviction, warm bypass) have not moved hit-rate on this workload.** The stash holds too small a fraction of total capacity, and stash-resident items rarely carry high frequency counts, so gate conditions fire too infrequently to shift the aggregate miss curve regardless of how the gate is structured.\n- **Probing re-entry at counter ≥1 is too permissive.** Items that have seen only one probing hit bounce before earning their target slot, increasing churn and degrading both hit-rate and get-path tail latency (Cycle 10 v003). The counter gate at ≥2 contributes independent signal beyond the vacancy condition.\n- **Stash-bypass direction-flips on the MRU-end base.** A hypothesis that was a net win on the LRU-end base (Cycle 2) became a regression on the MRU-end base (Cycle 3) because back-insertion makes target saturation the common case, amplifying bypass-fire rate. Multi-file inheritance gaps (Cycle 4) further corrupted stash-bypass measurements by silently reverting parent struct compaction.\n\n## Open questions\n\n- **Can the clock and frequency-promotion counters be decoupled structurally?** A separate boolean \"chance bit\" — set only when an eviction rescue occurs, not on every qualifying access — would allow the clock's warmth signal to remain sparse while the promotion counter accumulates unimpeded. This is the most direct path to composing both mechanisms without shared-field conflict.\n- **Can a consecutive-rotation cap bound hot-bucket tail latency?** Aborting `clock_evict_prepare` after k cleared bits and falling back to LFU-minimum would prevent worst-case O(k·n) rotation depth in hot buckets while preserving the freq≥3 threshold's discriminating power in cold buckets — isolating the latency regression to a bounded worst case rather than an average-case cost.\n- **Is the probing clock pre-pass viable with occupancy gating?** v002 (Cycle 10) demonstrated genuine put-path latency savings from front-loading cold-victim selection in probing, but the mechanism stickifies warm probing residents and blocks target→probing→target circulation. Gating the pre-pass on bucket fill fraction above a threshold could preserve the latency saving while suppressing the admission-blocking effect.\n- **Are there workload or cache-size conditions where dormant mechanisms activate?** Probing re-entry (vacancy gate) has been persistently thin at high cache occupancy. Stash bypass and cold-stash eviction have been inert. Both families are structurally sound but precondition-rare; a smaller cache-to-working-set ratio or a non-stationary trace may change their firing rates enough to yield measurable signal.\n- **Do alternative warm-set signals exist that avoid integer clustering?** Zipfian access distributions concentrate heavily at exactly one or two accesses per item, making every clean integer threshold for CLOCK_BIT either too inclusive (≥1, ≥2) or too exclusive (≥3). A decayed counter, a per-item \"rescued from eviction\" bit, or a density-adaptive gate might provide sparsity without depending on absolute counts that cluster problematically on this trace.\n\n## Per-policy notes\n\n- **LRU (baseline):** Positional-LRU victim selection consistently leaves low-frequency items protected by recency alone; MRU-end insertion (placing new items at the eviction end) exploits this to accelerate warm-up but does not change eviction-selection quality. The LRU default is the origin of the struct-overhead, cascade-eviction, and promotion-ordering bugs that most winning hypotheses have addressed.\n- **LFU-minimum (scan-based):** Confirmed as a hit-rate lever in Cycles 5–6, improving victim quality by selecting minimum-frequency rather than LRU-position. Consistently produces put-path p99 regressions from the O(n) full-bucket scan; the magnitude scales with how often eviction fires, making it sustainable only for tiers that evict rarely (e.g., stash in Cycle 8). Not a viable drop-in for target-tier eviction without a structural replacement.\n- **Two-chance clock:** Introduced in Cycle 6 as the latency-safe alternative to LFU-minimum. Provides frequency-aware eviction via a rotation that marks warm items with CLOCK_BIT and terminates at the first cold victim. Works best on Zipfian workloads where cold-tail items are abundant and the clock terminates early in the average case. Its fundamental limitation is that CLOCK_BIT and `lfu_counter` must remain decoupled to preserve discriminating power: any mechanism that sets CLOCK_BIT on ordinary accesses (not just eviction rescues) saturates the warm set and degrades the clock to O(n) rotation. All integer thresholds on the raw counter have been tried and falsified across Cycles 8–10; decoupling through an independent bit is the outstanding structural fix.\n- **Frequency-gated promotion (vacancy-guarded ≥2):** The most durable multi-cycle winner. Established in Cycle 3 and confirmed optimal-for-workload-a at ≥2 in Cycle 4; ≥3 filtering was shown to exclude genuinely reusable items. Coexists safely with MRU-end insertion and struct compaction. Conflicts with clock eviction through shared `lfu_counter`.\n- **Cold-preferring stash eviction (Cycle 8 v003):** Regression-free latency improvement; O(n) scan amortized safely because stash eviction fires infrequently. Does not move hit-rate on this workload because stash residents are rarely frequency-rich. A clean latency win but not a hit-rate mechanism under current workload conditions.\n"
}
```

### 07:12:01 chat

`cross-cycle-insights.md` written to `cycles/0010/cross-cycle/`. Here's what the four sections synthesize across all ten cycles:

**Recurring wins** — five durable anchors: MRU-end insertion, per-item struct compaction (timestamp removal), vacancy-gated promotion, additive stacking of orthogonal mechanisms, and the two-chance clock eviction (which solved the O(n) scan latency problem).

**Recurring failures** — six patterns that keep not paying off: O(n) per-eviction scans (consistent p99 regressions), counter decay on stationary workloads, the clock/promotion counter shared-field conflict (all integer thresholds ≥1/≥2/≥3 now falsified), stash-tier hit-rate interventions (consistently inert), probing re-entry at ≥1 (too permissive, confirmed Cycle 10), and stash-bypass direction-reversal on the MRU base.

**Open questions** — five threads worth probing: decoupling clock and promotion signals via an independent chance bit, a consecutive-rotation cap to bound hot-bucket tail latency, occupancy-gated probing clock pre-pass, workload/size conditions where dormant mechanisms activate, and alternative warm-set signals that avoid Zipfian integer clustering.

**Per-policy notes** — coverage of LRU baseline, LFU-minimum scan, two-chance clock, vacancy-gated frequency promotion, and cold-preferring stash eviction, each with their confirmed strengths, failure modes, and outstanding constraints.


---

- finished: 2026-07-21T07:12:01+00:00
- duration: 74.44s
- status: ok
- input_tokens: 6
- output_tokens: 2952
- cache_read_tokens: 107052
- cache_write_tokens: 12968
- cost_usd: 0.133492
