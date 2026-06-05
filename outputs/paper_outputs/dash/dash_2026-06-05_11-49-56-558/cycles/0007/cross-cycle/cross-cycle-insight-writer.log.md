# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-06-05T14:31:35+00:00

## timeline

### 14:31:39 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look for the output directory structure.

### 14:31:40 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 14:31:40 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 14:31:42 tool: Bash

```json
{
  "command": "ls",
  "description": "List files in current directory"
}
```

### 14:31:44 tool: Read

```json
{
  "file_path": "/root/task.md"
}
```

### 14:31:48 tool: Bash

```json
{
  "command": "pwd",
  "description": "Show current working directory"
}
```

### 14:32:07 thinking

The output directory is `/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cross-cycle`. I should write `cross-cycle-insights.md` in this directory.

Now let me synthesize insights from all 7 cycles based on the ledger digest:

**Recurring wins:**
1. Hot-tier topology expansion (normal/stash ratio) was the first dominant lever — going from baseline to 50/50 (cycle 1), then to 75/25 (cycle 2), peaked at 24/8 (cycle 3)


2. MRU insertion positioning consistently delivered gains, especially back-inserting after stash hits and strategic MRU placement throughout the pipeline
3. Eviction selectivity proved critical in later cycles, with zero-access frequency guards in cycle 6 showing the biggest improvements
4. Combining orthogonal mechanisms works — cycle 7's v001+v002 demonstrated that targeting different pipeline stages additively compounds gains
5. Structural changes outperform policy tweaks — topology and positioning adjustments beat frequency-based discrimination every time

**Recurring failures:**
1. Frequency-based eviction strategies consistently underperform on workload-a, where Zipfian distributions punish any frequency discrimination approach

**What didn't work:**
Ghost caches and victim re-admission added only noise regardless of implementation approach, while any deviation from MRU status on new or re-admitted items caused regressions. Timestamp-based promotion clashed destructively with frequency guards, bypassing the stash-as-filter degraded hit rates, and raising frequency thresholds beyond zero hurt performance since single-touch items are actually warm on this workload. I'm still investigating what loss sites remain after combining frequency guards with second-chance eviction, whether timestamp promotion could work if only recorded on first hit rather than insertion, and how counter resets on stash demotion interact with the broader system.

**What worked:**
Classic LRU with position-based recency dominates across all tests with MRU universally preferred, frequency-based eviction consistently fails regardless of formulation, and the frequency guard protecting zero-access items proved to be the biggest recent win with the exact right cutoff. Second-chance stash recovery adds real but modest gains on top of that, though timestamp-based approaches replicate some established improvements while still conflicting with frequency guards.

**Open questions:**
Whether ghost sets or probationary zones could shine on different workload traces with heavier scan and cold-item churn, if there's an optimal stash capacity below 8 worth exploring, and whether any capacity-stress regimes would shift these conclusions.

Now I'm writing up the results.

### 14:32:07 chat

Now I have all the context I need. Let me write the `cross-cycle-insights.md` file synthesizing patterns across all 7 cycles.

### 14:32:37 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Hot-tier topology expansion** was the earliest and highest-leverage gain: enlarging the normal-bucket pool at the expense of stash produced clear wins in Cycles 1–2, peaking at a 24-normal / 8-stash split in Cycle 3. Once the working set fits comfortably in the hot tier, capacity is no longer the binding constraint.\n- **MRU insertion position universally wins on workload-a.** Every attempt to withhold MRU status — at stash re-entry, ghost re-admission, demotion cascades, or probing entry — regressed or was noise. Short reuse gaps mean recently-positioned items are reliably re-accessed before eviction.\n- **Eviction selectivity within a tier is a live lever.** Preferring to evict zero-access newcomers over confirmed-reuse residents (Cycle 6 frequency guard) produced the largest single-cycle gain in the late search, confirming the probing boundary was a meaningful loss site even after topology and insertion position were optimised.\n- **Orthogonal mechanisms stack cleanly.** Topology + MRU re-entry (Cycles 1–3), frequency-guard + second-chance recovery (Cycle 7): when two variants target genuinely distinct pipeline stages, combining them closes both loss sites without interference and produces additive gains.\n- **Stash serves as a load-bearing admission filter.** Respecting stash traversal — never bypassing or weakening it for cold insertions — has been a consistent constraint; the stash gate's quality function outweighs any cost savings from shortcutting it.\n\n## Recurring failures\n\n- **Frequency-based eviction (LFU, aging-LFU, threshold-gated LFU) consistently underperforms.** In every formulation tested (Cycles 1–3, 7), frequency discrimination either matched or degraded ClassicLRU. Workload-a's Zipfian-but-warm access distribution makes item position a stronger future-access signal than any counter variant; even a single in-tier hit is a reliable reuse predictor that aggressive frequency pruning destroys.\n- **Ghost/victim-cache re-admission is noise-level regardless of configuration.** Tested in multiple cycles (4–6) with varying position and trigger logic; the workload's reuse pattern does not cycle through the stash boundary frequently enough to benefit.\n- **Bypassing or weakening the stash gate regresses.** Front-door cold bypass (Cycle 4 v003) and promotion-threshold gating (Cycle 4 v001) both hurt hit-rate by allowing unqualified items to displace warm residents.\n- **Timestamp-based promotion interacts destructively with frequency-guard eviction.** In Cycle 7, using `last_accessed` rather than LRU position to select promotion candidates allowed cold items with newer creation timestamps to win promotion, cancelling the eviction-guard benefit.\n- **Raising the frequency-guard threshold beyond zero is harmful.** A single in-tier access is a sufficient warm signal on workload-a; treating single-touch items as eviction candidates shrinks the protected set too aggressively (Cycle 7).\n- **Probationary / LRU-entry insertion is consistently punishing.** Segmented-LRU asymmetry and probationary-zone probes (Cycles 3, 5) both regressed — workload-a's access distribution is warm throughout, so demotion buffer positions accelerate churn rather than improve selectivity.\n\n## Open questions\n\n- With frequency-guard eviction and second-chance stash recovery both active, what loss site remains? Stash geometry fine-tuning (e.g., sub-8 stash capacity) and `lfu_counter` staleness management (soft caps to prevent over-protection of aging items) are uninvestigated.\n- Does `lfu_counter` reset on stash demotion interact beneficially or harmfully with second-chance recovery when both are active simultaneously?\n- Could timestamp-based promotion be rehabilitated by recording timestamps only on the first *hit* (not on insertion), eliminating creation-time contamination?\n- Would frequency-based eviction, ghost sets, or probationary zones win on a different workload trace with heavier scan traffic or bursty cold-item arrivals — where the access distribution is not uniformly warm?\n- Is the current optimal geometry (24/8) robust across capacity regimes, or does a smaller total cache force a different hot/stash ratio?\n\n## Per-policy notes\n\n- **ClassicLRU / position-based recency:** Dominant throughout all cycles on workload-a. MRU insertion is consistently optimal; any deviation toward LRU/Front positioning regresses. This policy class should be the baseline for all future variants.\n- **LFU / frequency-based discrimination:** Structurally incompatible with workload-a. Classic LFU, aging-LFU (aggressive and threshold-gated), and raised frequency-guard thresholds all failed. Single-touch items carry genuine reuse signal and must not be treated as cold. Retire for workload-a; revisit only if a scan-heavy or frequency-skewed trace is introduced.\n- **Ghost / victim-cache re-admission:** Noise-level across every position (MRU, Back-insert) and trigger variant tested. The stash eviction boundary is not where meaningful hit-rate is lost. Permanently retired.\n- **Zero-access frequency guard (eviction selectivity):** The most impactful late-search gain. Protecting any item with at least one confirmed in-tier hit from eviction by zero-access newcomers is both necessary and sufficient — the zero-access cutoff is exactly right for this workload. Active in the current champion.\n- **Second-chance stash overflow recovery:** Real, positive, additive with the frequency guard because it targets a different pipeline stage (stash overflow vs. normal-tier eviction ordering). Active in the current champion.\n- **TimestampLRU:** Partially replicates topology and MRU-insertion gains already captured by prior winners; does not surpass them, and interacts destructively with frequency-guard eviction. Deprioritised unless a creation-time-clean variant can be specified.\n"
}
```

### 14:32:44 chat

`cross-cycle-insights.md` written. Here's a brief summary of what was synthesised across all seven cycles:

**Recurring wins** — Hot-tier topology expansion was the dominant early lever (peaking at 24/8), MRU insertion position universally wins, and the zero-access frequency guard was the standout late-cycle gain. Orthogonal mechanisms that target distinct pipeline stages stack cleanly (topology+re-entry; frequency-guard+second-chance).

**Recurring failures** — Frequency-based eviction (LFU in all forms) is structurally incompatible with workload-a's warm Zipfian distribution. Ghost/victim-cache re-admission is noise-level regardless of configuration. Bypassing the stash admission gate and any LRU/probationary insertion positioning consistently regress.

**Open questions** — Remaining loss sites with the current champion (counter staleness, sub-8 stash geometry), whether timestamp promotion can be salvaged by avoiding creation-time contamination, and whether any of the failed approaches would win on a scan-heavy or cold-burst workload trace.

**Per-policy notes** — ClassicLRU/position recency dominates; LFU and ghost sets are retired for this workload; the zero-access frequency guard and second-chance recovery are the current champion's two active mechanisms.


---

- finished: 2026-06-05T14:32:45+00:00
- duration: 69.22s
- status: ok
- input_tokens: 7
- output_tokens: 3023
- cache_read_tokens: 101040
- cache_write_tokens: 9084
- cost_usd: 0.114856
