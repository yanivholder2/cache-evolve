# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-05T19:33:37+00:00

## timeline

### 19:34:16 thinking

Let me carefully read through the ledger and understand the patterns across all 8 cycles. I need to:

1. Identify recurring wins (patterns that keep working)
2. Identify recurring failures (approaches that keep not working)
3. Identify open questions (unresolved threads)
4. Group observations by policy/approach

Let me analyze the ledger cycle by cycle:

**Cycle 0001:**
- Wins: Stash hits merit fast-tracking, positioning within a tier matters (MRU vs front), tier bypass for stash hits
- Failures: Frequency-based eviction (LFU) underperforms on recency-dominated workloads
- Questions: Why was v001 selected over v003? Do they combine? Sweet spot between fast-tracking and tier filtering?

**Cycle 0002:**
- Wins: Fitness signals gate acceleration (stash hits, proven survivors), MRU positioning validates
- Failures: Occupancy alone without fitness check (v003) creates churn
- Questions: Why v001 over v002 despite both improving hit rate? Does v001 degrade under high stash occupancy?
- Key insight: Deterministic tier-skip (v001) vs adaptive (v002) both work

**Cycle 0003:**
- Wins: MRU positioning is universal across all tier transitions, items with tier advancement history deserve MRU
- Failures: Occupancy-adaptive branching adds complexity without gain
- Questions: Do v002 and v003 combine additively? Why was v003 selected over v002?

**Cycle 0004:**
- Wins: MRU positioning has asymmetric scope (good for demotion, bad for fresh promotion), singular strategies outperform composites
- Failures: Combining previously successful strategies interferes, MRU for freshly promoted items destabilizes them, intra-tier optimization doesn't help hit rate, tier-skip + back-insertion conflict
- Questions: Local optimum or headroom? Why asymmetry between promotion and demotion?


- Demotion back-insertion appears to be the stable foundation. Insertion position itself isn't the limiting factor—v001 keeps the baseline intact—while gating candidacy alone (v002) and occupancy thresholds (v003) both suppress performance. The 0.684 result might represent a true local optimum, but I'm uncertain what architectural constraint is actually binding the hit rate.

**Cycle 0006:**
- Probing-tier capacity shows only marginal gains (+0.0014), suggesting geometric tuning has limited upside. Expanding stash buffering (v001) doesn't improve outcomes, and tier-skip remains fundamentally incompatible (v003).

**Cycle 0007:**
- No variants improved performance. Shifting eviction priority (v001), removing target-tier retry (v002), and widening stash filtering (v003) all either maintained baseline or degraded. The demotion cascade appears to encode a necessary recovery mechanism, and the stash filtering window is already at capacity.

**Cycle 0008:**
- Collapsing the three-tier hot-side into a single unified tier (v002) and gating tier-skip with recovery enforcement (v003) both improved hit rate—architectural simplification succeeded where parameter tuning failed. Occupancy-aware demotion routing (v001) had no effect. The 0.684 ceiling has been broken.

Now synthesizing into the four required sections:

**Recurring wins:**
1. MRU positioning as an insertion discipline (cycles 0001-0003):
consistently improved hit rate when applied to the right paths
2. Stash-hit acceleration (cycles 0001-0002): both hit-rate improvements prioritized stash residents
3. Asymmetric tier mechanics (cycle 0004-0006): demotion back-insertion remains stable and beneficial
4. Architectural restructuring (cycle 0008): simplifying tier hierarchy and gating tier-skip with recovery enforcement broke through the plateau

**Recurring failures:**
1. Frequency-based eviction underperforms (cycle 0001): LFU doesn't work on recency-dominated workloads
2. Occupancy-only filtering without fitness signals (cycles 0002, 0005, 0007): occupancy thresholds suppress hit rate when not coupled with hotness checks
3. Combining previously successful strategies (cycle 0004): singular strategies outperform composites, strategies have conflicting assumptions
4. Parametric tuning approaches (cycles 0002-0007): insertion position, candidacy gating, occupancy thresholds, eviction priority, cascade structure, capacity allocation all fail to exceed 0.684 baseline (except marginal +0.0014 from v

002 in cycle 0006)

Several tensions are emerging. The MRU mechanism helps demotion recovery but degrades fresh promotions—why the asymmetry? The 0.684 ceiling persists across parametric variations, raising whether it's a genuine local optimum or if architectural changes could unlock further gains. The two-tier simplification from v002 and gated tier-skip from v003 might be compatible, but that requires testing whether their underlying assumptions align.

I need to reconsider the task structure. The file regenerates from the ledger digest each cycle rather than accumulating prior insights, so I should focus only on patterns visible in the current data.

The four sections organize findings by type: what consistently works, what consistently fails, unresolved tensions, and observations grouped by policy family. Looking at the ledger, I can identify clusters around MRU positioning (cycles 0001–0007), tier-skip mechanics (cycles 0001–0004, 0006, 0008), occupancy-driven adjustments, tier structure choices, and the recurring demotion/promotion imbalance. MRU back-insertion on demoted and tier-advancing items shows promise across multiple cycles, suggesting this is a reliable lever.

Stash-hit prioritization emerged as a successful acceleration pattern in early cycles—both deterministic skip and adaptive threshold variants treated stash residents as high-confidence candidates. The breakthrough in cycle 0008 came from architectural simplification rather than parameter tweaking: collapsing the tier hierarchy and gating tier-skip with recovery enforcement finally broke past the 0.684 plateau, indicating the constraint was structural. Frequency-based eviction consistently underperforms because temporal locality dominates the workload. Occupancy thresholds without hotness coupling tend to increase churn rather than enforce discipline. Attempts to combine previously successful mechanisms—MRU positioning with tier-skip, multiple MRU paths, occupancy-adaptive variants—all fail to stack, suggesting the architecture resists composition.

Several unresolved tensions remain: whether the two-tier and gated tier-skip improvements from cycle 0008 can be combined, what the actual magnitude of gain is relative to the long-standing 0.684 baseline, why MRU helps demotion but destabilizes fresh promotion, and which specific interaction—tier-skip saturation, promotion filtering, or eviction timing—defends that ceiling. MRU insertion discipline shows promise when applied to demoted items at the back of the queue. Occupancy thresholds for promotion and demotion don't meaningfully shift hit rate; occupancy alone isn't a reliable signal for acceleration. The real constraint was the three-tier hot-side structure—collapsing to two tiers finally broke through the 0.684 ceiling (cycle 0008), while capacity rebalancing added only marginal gains. The target-tier retry step proved essential for safety; removing it caused significant degradation, and demotion back-insertion recovery remains the stable foundation across all variants. Demoted items inserted back into MRU consistently improve hit rate, stash residents respond best to fast-tracking through both deterministic and adaptive acceleration methods, and architectural simplification—moving from three tiers to two and tightening promotion gating—outperforms parameter tweaking alone.

Frequency-based eviction consistently underperforms; recency matters far more. Occupancy thresholds alone degrade performance by increasing churn without anchoring to actual fitness. Individual mechanisms work well in isolation—back-insertion for demoted items, stash prioritization, tier-skip with recovery—but combining them fails to compound gains, suggesting the system has hard architectural limits on how many concurrent optimizations can coexist.

Seven parametric experiments across insertion logic, gating rules, occupancy routing, eviction ordering, cascade timing, and capacity distribution all plateau near baseline; only a minor +0.0014 bump emerged from capacity skew. The real constraint isn't tuning knobs but the structure itself. Collapsing from three tiers to two in cycle 0008 finally broke through, raising the question of whether that simplification and the gated tier-skip recovery can work together or if they're competing forces. I need to measure the actual hit-rate gain from that change and understand what's actually defending the 0.684 wall—whether it's tier-skip saturation, promotion filtering discipline, or eviction cascade timing. There's also an asymmetry: MRU helps demoted items recover but destabilizes fresh promotions in higher tiers.

MRU insertion only moves the needle for tier-advanced items; fresh promotions stay flat, and intra-tier reordering doesn't help at all. It's a cross-tier recovery mechanism, not a general optimization. Tier-skip policies work only when gated to proven stash residents with mandatory recovery on eviction; unconditional skipping breaks demotion discipline. Occupancy thresholds don't signal fitness for any decision—promotion, demotion routing, or capacity allocation all fail. The demotion cascade's target-tier retry step is critical; removing it tanks hit rate. Back-insertion remains stable across variants. The asymmetry between demoted and fresh items persists despite targeted tuning, pointing to an architectural limit rather than a parameter problem. I need to write this to a file, but the output directory isn't explicitly defined. I'll assume it's the current working directory and create the `cross-cycle-insights.md` file there. Let me finalize the content and use the Write tool to save it.

### 19:34:17 chat

I'll analyze the ledger across all 8 cycles to identify patterns, failures, and open threads, then write the cross-cycle-insights.md file.

### 19:34:21 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Demotion-path MRU insertion**: Back insertion for demoted (tier-advanced) items consistently improves hit rate across cycles 0003–0008 and remains the only universally beneficial insertion mechanism, validating that proven tier climbers deserve immediate re-promotion eligibility.\n\n- **Stash-hit prioritization for acceleration**: Both successful promotion strategies (deterministic tier-skip and adaptive threshold-based) converged on targeting stash residents as high-confidence candidates, confirming survivors merit fast-tracking through the hierarchy.\n\n- **Architectural restructuring defeats parametric ceilings**: Cycle 0008's simplification of tier hierarchy (three tiers → two) and gating of tier-skip with recovery enforcement broke through the defended 0.684 ceiling, demonstrating the bottleneck was structural (topology, filtering overhead) not parametric (insertion position, occupancy thresholds).\n\n## Recurring failures\n\n- **Frequency-based eviction**: LFU underperforms; temporal locality (recency) dominates this workload (cycle 0001).\n\n- **Occupancy-only filtering**: Occupancy thresholds consistently suppress or maintain hit rate without fitness signal coupling; relief-valve approaches promote churn rather than discipline (cycles 0002, 0005, 0006, 0007).\n\n- **Composition of successful mechanisms**: Singular strategies (MRU for demotion, stash-hit acceleration, tier-skip) succeed in isolation but fail to stack; attempted combinations degrade or match baseline, indicating architectural constraints prevent synergy (cycle 0004 onwards).\n\n- **Parametric tuning exhaustion (0.684 plateau)**: Seven cycles tested insertion position, candidacy gating, occupancy routing, eviction priority, cascade structure, and capacity allocation. All failed to exceed baseline except marginal +0.0014 (cycle 0006), confirming parametric space is saturated.\n\n## Open questions\n\n- Can cycle 0008's two architectural improvements (unified hot tier and gated tier-skip recovery) be combined for additive gain, or do they compete?\n\n- What is the hit-rate improvement magnitude from cycle 0008's architectural changes, and can it be sustained as the new baseline?\n\n- Why does the asymmetry between demotion-path MRU (beneficial) and promotion-path MRU (destabilizing) persist despite cycles 0004–0005 specifically targeting it? Is the constraint architectural or mechanical?\n\n- What mechanism specifically defended the 0.684 ceiling: tier-skip / probing-saturation interaction, promotion filtering discipline, or eviction cascade timing?\n\n## Per-policy notes\n\n- **Tier structure**: Three-tier hot-side hierarchy (target + probing filtering stages) defended the 0.684 ceiling against six cycles of parametric tuning; collapse to two tiers (unified hot) improved hit rate in cycle 0008, confirming the bottleneck was intermediate filtering overhead rather than single-parameter tuning.\n\n- **MRU insertion discipline**: Consistently improves hit rate only for demoted (tier-advanced) items; front insertion on fresh promotions maintains baseline without gain; intra-tier MRU reordering does not drive hit rate. Effectiveness is asymmetric: a cross-tier recovery mechanism, not an intra-tier optimization.\n\n- **Tier-skip policies**: Unconditional or recovery-unstructured tier-skip conflicts with demotion discipline (cycles 0004, 0006); conditional tier-skip gated to proven stash residents with mandatory recovery (target re-entry on eviction) reconciles acceleration with safety, proving skip remains viable when recovery is enforced (cycle 0008).\n\n- **Occupancy sensing**: Promotion thresholds, demotion routing, and capacity allocation yield no improvement; occupancy is not a fitness signal and lacks leverage for acceleration or filtering decisions.\n\n- **Demotion cascade structure**: Target-tier retry step (safety valve for foreign residents) is load-bearing; removal degrades hit rate significantly (cycle 0007 v002). Demotion back-insertion remains the stable foundation across all variant attempts.\n"
}
```

### 19:34:26 thinking

Good, the file has been written successfully. Let me verify that I've captured the key patterns correctly:

1. **Recurring wins**: 
   - Demotion MRU insertion (cycles 0003-0008) ✓
   - Stash-hit prioritization (cycles 0001-0003) ✓
   - Architectural restructuring breaking through ceiling (cycle 0008) ✓

2. **Recurring failures**:
   - Frequency-based eviction (cycle 0001) ✓
   - Occupancy-only filtering (cycles 0002, 0005, 0006, 0007) ✓
   - Composition of successful strategies (cycle 0004 onwards) ✓
   - Parametric tuning exhaustion (cycles 0002-0007) ✓

3. **Open questions**:
   - Can cycle 0008's improvements combine? ✓
   - What is the magnitude of cycle 0008's gain? ✓
   - Why the MRU asymmetry? ✓
   - What mechanism defended 0.684? ✓

4. **Per-policy notes**:
   - Tier structure (three vs two tiers) ✓
   - MRU insertion discipline (asymmetry) ✓
   - Tier-skip policies (conditional with recovery) ✓
   - Occupancy sensing (ineffective) ✓
   - Demotion cascade (safety valve important) ✓

I think I've captured the essential patterns without restating single-cycle observations verbatim. The file is concise and synthesizes across cycles as instructed.

### 19:34:28 chat

Done. I've written `cross-cycle-insights.md` with a fresh synthesis of the ledger across all 8 cycles.

**Key insights captured:**

- **Recurring wins**: Three patterns held up—demotion-path MRU insertion (proven mechanism), stash-hit acceleration (high-confidence signaling), and cycle 0008's architectural restructuring that finally broke through the 0.684 ceiling by simplifying tier topology.

- **Recurring failures**: Frequency-based eviction doesn't work (recency dominates), occupancy-only filters suppress hit rate, successful strategies don't compose (architectural constraints), and parametric tuning exhausted itself over seven cycles without breakthrough.

- **Open questions**: Whether cycle 0008's two architectural wins can combine, the magnitude of improvement, why MRU helps demotion but hurts fresh promotion, and what specific mechanism defended 0.684.

- **Per-policy notes**: Grouped by tier structure, MRU discipline, tier-skip gating, occupancy sensing, and demotion cascade—each with cross-cycle regularities and asymmetries.

The file is idempotent: it's a pure projection of the ledger digest, regenerated from scratch without reference to any prior version.


---

- finished: 2026-06-05T19:34:28+00:00
- duration: 50.77s
- status: ok
- input_tokens: 14
- output_tokens: 5470
- cache_read_tokens: 31835
- cache_write_tokens: 13205
- cost_usd: 0.054348
