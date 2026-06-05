# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-06-05T18:48:51+00:00

## timeline

### 18:49:06 thinking

Let me analyze the ledger digest across all 4 cycles to identify patterns, recurring wins/failures, open questions, and per-policy observations.

## Analysis:

### Cycle 0001:
- **Wins**: Stash hits merit fast-tracking (v001, v003); positioning within tier matters (v001); tier-bypass works (v003)
- **Failures**: Frequency-based eviction doesn't work (v002) on recency-dominated workload
- **Key insight**: Temporal locality > access count

### Cycle 0002:
- **Wins**: Fitness signals gate acceleration (v001 deterministic skip, v002 adaptive skip); both preserve stash-hit signals
- **Failures**: Occupancy-driven relief without fitness check creates churn (v003 at 60%)
- **Key insight**: Occupancy alone is not a fitness signal; stash-hit detection is the true discriminator

### Cycle 0003:
- **Wins**: MRU back insertion is universal (v002, v003); demoted items deserve MRU treatment; consistent positioning > conditional branching
- **Failures**: Occupancy-adaptive branching adds complexity without gain (v001)
- **Key insight**: Deterministic MRU positioning is better than conditional logic

### Cycle 0004:
- **Wins**: None - all variants hurt hit rate
- **Failures**: All combination attempts failed:
  - MRU back insertion hurts when applied to freshly promoted items (v001)
  - Intra-tier optimization doesn't translate to gains (v002)
  - Tier-skip + back-insertion create conflicting classes of probing residents (v003)
- **Key insight**: Asymmetry in how MRU works for demotion vs. fresh promotion; simple singular strategies outperform composites; local optimum reached

## Recurring Wins (across cycles):
1. **Stash hits as acceleration signal** - Cycles 0001-0002 both showed that detecting stash hits and accelerating them (either via deterministic skip or adaptive occupancy-aware promotion) improves hit rate


2. **MRU positioning for specific paths** - Cycles 0002-0003 demonstrated that back-insertion to MRU works reliably for promotion and demotion flows. **Fitness signals gate acceleration** - Occupancy checks alone fail; stash-hit or recency signals are required to gate promotion. **Simplicity & determinism** - Deterministic positioning beats conditional adaptive branching; Cycle 0004 confirmed singular strategies outperform composites.

## Recurring Failures:
1. **Frequency-based eviction** - Cycle 0001 definitively showed LFU doesn't work on this recency-dominated workload
2. **Occupancy-as-relief-valve** - Without fitness checks, occupancy-driven promotion creates churn rather than improvement

. Conditional destination logic based on occupancy adds complexity without measurable benefit. Combining successful individual strategies causes interference, suggesting composition requires asymmetric rules.

## Open Questions:
1. Is cycle 0003's baseline (0.6839) a local optimum or is there headroom?
2. Why does MRU insertion help demotion but hurt fresh promotion?
3. Are there completely different levers—tier capacity, eviction policy, bucket structure—worth exploring?
4. Why was v001 selected over v003 in cycles 0001-0002 despite similar results?
5. What's the optimal balance between fast-tracking and tier-by-tier filtering?

## Per-policy Notes:
Stash-hit detection drives promotion; occupancy-adaptive variants don't improve outcomes. Demotion benefits from MRU positioning, indicating demoted items retain value and warrant faster re-promotion eligibility. Frequency-based eviction fails while recency-based approaches dominate; occupancy alone isn't a fitness signal. Insertion position—particularly MRU placement—matters for demotion and certain promotion paths.

MRU insertion helps demoted items but creates friction for newly promoted ones. I should document these cross-cycle patterns in the insights file.

### 18:49:06 chat

Looking at the ledger digest across cycles 0001–0004, I'll identify patterns, recurring wins/failures, open questions, and per-policy observations.

### 18:49:14 tool: Write

```json
{
  "file_path": "/tmp/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Stash-hit detection as acceleration signal**: Cycles 0001–0002 consistently showed that identifying stash-resident items and prioritizing their promotion (deterministically or adaptively) improves hit rate. This is the single most reliable lever tested so far.\n- **MRU positioning on demotion paths**: Cycles 0002–0003 converged on back insertion for demoted items, treating re-entry as an opportunity to restore high promotion eligibility. This positioning rule consistently improved hit rate when applied to demotion cascades.\n- **Fitness signals gate promotion**: Cycles 0001–0002 validated that stash-hit or recency signals must precede acceleration. Occupancy-threshold-only checks without fitness signals trigger harmful churn (cycle 0002 v003 failure).\n- **Deterministic rules outperform conditional branching**: Cycles 0002–0003 showed that consistent, simple positioning rules beat adaptive occupancy-triggered destination selection, even when the conditional logic is semantically reasonable.\n\n## Recurring failures\n\n- **Frequency-based eviction on recency workloads**: Cycle 0001 definitively ruled out LFU; access count is a weaker signal than temporal locality on this traffic pattern.\n- **Occupancy-adaptive branching**: Cycles 0002–0003 both showed that gating tier selection or insertion position based on occupancy thresholds adds complexity without hit-rate gain; the signal is noise compared to fitness checks (stash hit, proven promotion history).\n- **Combining successful strategies**: Cycle 0004 revealed hard boundaries: stash-hit tier-skip, MRU back insertion, and deterministic positioning all work in isolation but interfere when combined, suggesting the baseline has reached a local optimum within the current rule set.\n- **MRU positioning on fresh promotion**: Cycle 0004 v001 showed that the same back-insertion rule beneficial for demoted items destabilizes newly promoted items in higher tiers, revealing an asymmetry in how the mechanism should be applied across promotion vs. demotion paths.\n\n## Open questions\n\n- **Local optimum or headroom?** Cycle 0004 hit rate (0.6839) shows three separate interference patterns; is this a basin floor, or do fundamentally different levers (tier capacity, eviction policy, bucket structure, or asymmetric insertion rules) exist?\n- **Why the MRU asymmetry?** Demotion-path back insertion helps; fresh-promotion back insertion hurts. Is the difference eviction cadence, promotion eligibility timing, or something about how the system filters candidates at each tier?\n- **Intra-tier vs. cross-tier scope:** Cycle 0004 v002 showed that repositioning within a static tier (stash MRU ordering) yields no gain. Does the MRU principle apply only to tier-advancement transitions, or are there other cross-tier paths where it hasn't been tested?\n\n## Per-policy notes\n\n- **Stash-hit acceleration (promotion policy)**: Both deterministic tier-skip (cycle 0002 v001) and occupancy-adaptive tier-skip (cycle 0002 v002) work; the core mechanism is stash-hit detection, not the adaptation strategy. Occupancy checks are secondary.\n- **Demotion policy**: MRU back insertion on demotion consistently improves hit rate (cycle 0003 v003). The rule suggests demoted items retain high re-promotion eligibility, validating that the system should reward survivors rather than penalize them.\n- **Eviction and composition boundaries**: Simple, singular strategies (one mechanism per cycle) outperform composites (cycle 0004 all failures). Suggests the workload's hit-rate surface is dominated by a single mechanism at a time, and layering mechanisms creates conflicts that erase gains.\n"
}
```

### 18:49:16 thinking

I need to write to the correct output directory. The error message indicates the file should be at `/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle-insights.md`. Let me write to that path.

### 18:49:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Stash-hit detection as acceleration signal**: Cycles 0001–0002 consistently showed that identifying stash-resident items and prioritizing their promotion (deterministically or adaptively) improves hit rate. This is the single most reliable lever tested so far.\n- **MRU positioning on demotion paths**: Cycles 0002–0003 converged on back insertion for demoted items, treating re-entry as an opportunity to restore high promotion eligibility. This positioning rule consistently improved hit rate when applied to demotion cascades.\n- **Fitness signals gate promotion**: Cycles 0001–0002 validated that stash-hit or recency signals must precede acceleration. Occupancy-threshold-only checks without fitness signals trigger harmful churn (cycle 0002 v003 failure).\n- **Deterministic rules outperform conditional branching**: Cycles 0002–0003 showed that consistent, simple positioning rules beat adaptive occupancy-triggered destination selection, even when the conditional logic is semantically reasonable.\n\n## Recurring failures\n\n- **Frequency-based eviction on recency workloads**: Cycle 0001 definitively ruled out LFU; access count is a weaker signal than temporal locality on this traffic pattern.\n- **Occupancy-adaptive branching**: Cycles 0002–0003 both showed that gating tier selection or insertion position based on occupancy thresholds adds complexity without hit-rate gain; the signal is noise compared to fitness checks (stash hit, proven promotion history).\n- **Combining successful strategies**: Cycle 0004 revealed hard boundaries: stash-hit tier-skip, MRU back insertion, and deterministic positioning all work in isolation but interfere when combined, suggesting the baseline has reached a local optimum within the current rule set.\n- **MRU positioning on fresh promotion**: Cycle 0004 v001 showed that the same back-insertion rule beneficial for demoted items destabilizes newly promoted items in higher tiers, revealing an asymmetry in how the mechanism should be applied across promotion vs. demotion paths.\n\n## Open questions\n\n- **Local optimum or headroom?** Cycle 0004 hit rate (0.6839) shows three separate interference patterns; is this a basin floor, or do fundamentally different levers (tier capacity, eviction policy, bucket structure, or asymmetric insertion rules) exist?\n- **Why the MRU asymmetry?** Demotion-path back insertion helps; fresh-promotion back insertion hurts. Is the difference eviction cadence, promotion eligibility timing, or something about how the system filters candidates at each tier?\n- **Intra-tier vs. cross-tier scope:** Cycle 0004 v002 showed that repositioning within a static tier (stash MRU ordering) yields no gain. Does the MRU principle apply only to tier-advancement transitions, or are there other cross-tier paths where it hasn't been tested?\n\n## Per-policy notes\n\n- **Stash-hit acceleration (promotion policy)**: Both deterministic tier-skip (cycle 0002 v001) and occupancy-adaptive tier-skip (cycle 0002 v002) work; the core mechanism is stash-hit detection, not the adaptation strategy. Occupancy checks are secondary.\n- **Demotion policy**: MRU back insertion on demotion consistently improves hit rate (cycle 0003 v003). The rule suggests demoted items retain high re-promotion eligibility, validating that the system should reward survivors rather than penalize them.\n- **Eviction and composition boundaries**: Simple, singular strategies (one mechanism per cycle) outperform composites (cycle 0004 all failures). Suggests the workload's hit-rate surface is dominated by a single mechanism at a time, and layering mechanisms creates conflicts that erase gains.\n"
}
```

### 18:49:23 thinking

The error message says "writes must stay under /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle." - it looks like there's a specific directory constraint. Let me check the exact path. It seems the output directory might be `/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle/` (with a trailing slash). Let me try writing to that.

Actually, I think I misread the error. It says the path is "outside the workspace" and "writes must stay under" a certain path. This suggests the workspace is /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle/ or something similar. Let me try the path with "cross-cycle" as a directory:

### 18:49:29 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Stash-hit detection as acceleration signal**: Cycles 0001–0002 consistently showed that identifying stash-resident items and prioritizing their promotion (deterministically or adaptively) improves hit rate. This is the single most reliable lever tested so far.\n- **MRU positioning on demotion paths**: Cycles 0002–0003 converged on back insertion for demoted items, treating re-entry as an opportunity to restore high promotion eligibility. This positioning rule consistently improved hit rate when applied to demotion cascades.\n- **Fitness signals gate promotion**: Cycles 0001–0002 validated that stash-hit or recency signals must precede acceleration. Occupancy-threshold-only checks without fitness signals trigger harmful churn (cycle 0002 v003 failure).\n- **Deterministic rules outperform conditional branching**: Cycles 0002–0003 showed that consistent, simple positioning rules beat adaptive occupancy-triggered destination selection, even when the conditional logic is semantically reasonable.\n\n## Recurring failures\n\n- **Frequency-based eviction on recency workloads**: Cycle 0001 definitively ruled out LFU; access count is a weaker signal than temporal locality on this traffic pattern.\n- **Occupancy-adaptive branching**: Cycles 0002–0003 both showed that gating tier selection or insertion position based on occupancy thresholds adds complexity without hit-rate gain; the signal is noise compared to fitness checks (stash hit, proven promotion history).\n- **Combining successful strategies**: Cycle 0004 revealed hard boundaries: stash-hit tier-skip, MRU back insertion, and deterministic positioning all work in isolation but interfere when combined, suggesting the baseline has reached a local optimum within the current rule set.\n- **MRU positioning on fresh promotion**: Cycle 0004 v001 showed that the same back-insertion rule beneficial for demoted items destabilizes newly promoted items in higher tiers, revealing an asymmetry in how the mechanism should be applied across promotion vs. demotion paths.\n\n## Open questions\n\n- **Local optimum or headroom?** Cycle 0004 hit rate (0.6839) shows three separate interference patterns; is this a basin floor, or do fundamentally different levers (tier capacity, eviction policy, bucket structure, or asymmetric insertion rules) exist?\n- **Why the MRU asymmetry?** Demotion-path back insertion helps; fresh-promotion back insertion hurts. Is the difference eviction cadence, promotion eligibility timing, or something about how the system filters candidates at each tier?\n- **Intra-tier vs. cross-tier scope:** Cycle 0004 v002 showed that repositioning within a static tier (stash MRU ordering) yields no gain. Does the MRU principle apply only to tier-advancement transitions, or are there other cross-tier paths where it hasn't been tested?\n\n## Per-policy notes\n\n- **Stash-hit acceleration (promotion policy)**: Both deterministic tier-skip (cycle 0002 v001) and occupancy-adaptive tier-skip (cycle 0002 v002) work; the core mechanism is stash-hit detection, not the adaptation strategy. Occupancy checks are secondary.\n- **Demotion policy**: MRU back insertion on demotion consistently improves hit rate (cycle 0003 v003). The rule suggests demoted items retain high re-promotion eligibility, validating that the system should reward survivors rather than penalize them.\n- **Eviction and composition boundaries**: Simple, singular strategies (one mechanism per cycle) outperform composites (cycle 0004 all failures). Suggests the workload's hit-rate surface is dominated by a single mechanism at a time, and layering mechanisms creates conflicts that erase gains.\n"
}
```

### 18:49:31 thinking

Great! The file was created successfully. Now let me verify it was written correctly by reading it back to confirm the content.

### 18:49:33 chat

Done. I've written `cross-cycle-insights.md` to the output directory.

The file synthesizes patterns across all four completed cycles:

**Recurring wins** identify three proven levers: stash-hit detection for acceleration, MRU positioning on demotion paths, fitness signals gating promotion, and deterministic rules over adaptive branching.

**Recurring failures** document what hasn't worked: LFU eviction on recency workloads, occupancy-adaptive branching without fitness checks, combining successful strategies (hard interference boundaries in cycle 0004), and asymmetric MRU positioning rules.

**Open questions** flag the most pressing unknowns: whether the cycle 0003 baseline is a local optimum, why MRU helps demotion but hurts fresh promotion, and the scope of the MRU principle across different tier transitions.

**Per-policy notes** group findings by the promotion/demotion/eviction mechanisms being tested, emphasizing that stash-hit detection is the core accelerator, demotion-back insertion is consistently beneficial, and simple singular strategies outperform composite approaches.


---

- finished: 2026-06-05T18:49:34+00:00
- duration: 42.41s
- status: ok
- input_tokens: 26
- output_tokens: 4663
- cache_read_tokens: 65904
- cache_write_tokens: 9134
- cost_usd: 0.044729
