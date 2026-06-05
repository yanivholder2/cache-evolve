# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-06-05T19:15:21+00:00

## timeline

### 19:15:47 thinking

Let me analyze the ledger digest provided. I need to read through all 6 cycles and identify patterns that have held up, approaches that consistently fail, unresolved threads, and observations grouped by policy.

Let me go through each cycle:

**Cycle 0001:**
- Stash hits merit fast-tracking: both winners (v001, v003) prioritized stash residents
- Positioning within a tier is meaningful: v001 benefited from Front→Back (MRU) insertion
- Tier bypass (v003) worked: skipping intermediate steps can outweigh filtering
- Frequency-based eviction (v002) failed: LFU doesn't work on recency-dominated workload
- Open: Why v001 over v003? Could they combine?

**Cycle 0002:**
- Fitness signals gate acceleration: successful variants (v001, v002) promoted only stash hits
- Threshold sensitivity: v002's 75% occupancy threshold worked, v003's 60% unconditional approach failed
- Two paths to stash-hit acceleration: deterministic (v001) and occupancy-adaptive (v002) both work
- MRU positioning remains relevant
- Open: Deterministic vs. adaptive? Does v002 handle high pressure better?

**Cycle 0003:**
- MRU positioning is universal across all transitions: v002, v003 both used back insertion
- Items with tier advancement history deserve MRU: demoted items benefit from re-promotion
- Occupancy-adaptive branching adds complexity without gain (v001): hurts hit rate
- Tier-by-tier progression + consistent positioning beats conditional tier-skipping
- Open: Do v002 and v003 combine? Why v003 over v002?

**Cycle 0004:**
- MRU positioning has asymmetric scope: works for demoted items, destabilizes newly promoted
- Intra-tier optimization is orthogonal: reordering within stash doesn't help (v002)


- Tier-skip + back-insertion conflict: they rest on incompatible assumptions (v003)
- Simple, singular strategies outperform composites
- Open: Local optimum or headroom? What's the asymmetry mechanism?

**Cycle 0005:**
- Insertion position isn't the bottleneck: v001 reverted to Front for promotion, Back for demotion, stayed at 0.684
- Gating candidacy alone doesn't move the needle: v002 maintained baseline without improvement
- Occupancy thresholds actually suppress performance: v003 dropped below baseline
- 0.684 appears to be a stable equilibrium rather than a temporary plateau
- Demotion recovery remains a stable foundation
- Open: Is 0.684 a true local optimum? What architectural factor constrains it?

**Cycle 0006:**
- Probing-tier capacity shows marginal returns: v002 gained only 0.0014 from rebalancing (18 probing vs 10 target)
- Expanding stash buffering doesn't help: v001 failed
- Tier-skip remains incompatible: v003 failed again, confirming cycle 0004
- 0.684 is defended by architectural asymmetries, not geometric tuning
- Open: Why only marginal gain from capacity skewing? Is 0.684 truly optimal?

Now let me synthesize:

## Recurring Wins

1. **Stash-hit acceleration to probing tier (cycles 0001-0002)**: Both deterministic tier-skipping and occupancy-adaptive approaches succeeded initially

, validating that fast-tracking stash hits is sound. However, this stopped working in later cycles due to architectural conflicts (cycle 0004-0006).

2. **Demotion back-insertion (cycles 0003-0006)**: The only sustained winner across all cycles. Inserting demoted items at Back (MRU) consistently improved or maintained hit rate and became the stable foundation.

3. **MRU positioning is a real effect (cycles 0001-0003)**: Applied correctly to demoted items and tier-advancing items, back insertion consistently drove improvements. The constraint emerged not from MRU positioning itself, but from asymmetric scope.

## Recurring Failures

1. **Frequency-based eviction (cycle 0001)**: LFU/frequency metrics don't work on this recency-dominated workload. Temporal locality trumps access count.

2. **Occupancy-based filtering without fitness checks (cycles 0002, 0005-0006)**: Threshold-gated acceleration that doesn't discriminate by item hotness suppresses performance or adds churn. Filtering needs to account for actual item fitness, not just occupancy thresholds.

3. **Unconditional tier-skipping (cycles 0004, 0006)**: Bypassing the tier-advancement filtering that justifies MRU insertion creates incompatibility with demotion-back-insertion discipline.

4. **MRU positioning for freshly promoted items (cycle 0004)**: Back insertion works for demoted items but destabilizes newly promoted items that haven't proven fitness at higher tiers. The asymmetry matters fundamentally.

5. **Occupancy-adaptive branching (cycle 0003, 0005)**: Conditional logic tied to occupancy levels introduces instability when occupancy fluctuates, causing inconsistent behavior across similar workload states.

6. **Intra-tier reordering (cycle 0004)**: Reordering within the stash tier doesn't improve hit rates on its own—gains only emerge when coupled with tier advancement.

7. **Capacity and geometric tuning (cycle 0006)**: Bucket allocation adjustments yielded minimal gains. These levers appear exhausted.

The 0.684 hit rate may represent a fundamental architectural ceiling given the current insertion and demotion mechanics, or untested levers like eviction ordering and saturation triggers could unlock further improvement. The core tension remains: MRU insertion recovers demoted items effectively but destabilizes freshly promoted ones, and I haven't found a unified rule that handles both cases well.

Tier-skip and demotion back-insertion continue to interfere despite being mechanically orthogonal—likely due to saturation or eviction cascade timing. Single-lever strategies from cycles 0004–0006 all hit interaction limits, suggesting either a hidden composition constraint or workload saturation under the current approach.

LRU-like positioning (front insertion) serves as a stable baseline but doesn't accelerate fresh promotions. MRU-like positioning (back insertion) shows strong recovery for demoted items but creates asymmetry elsewhere.

Frequency-based eviction failed early on a recency-dominated workload and wasn't revisited. Occupancy thresholds without fitness checks consistently suppress gains—geometric capacity rebalancing in cycle 0006 added only +0.0014. Tier-skip worked initially but became incompatible with demotion back-insertion by cycle 0004; tier-by-tier progression with steady MRU positioning proves more reliable.

### 19:15:47 chat

I'll analyze the ledger digest across all six cycles to extract cross-cycle patterns and write the insight file.

### 19:15:57 tool: Write

```json
{
  "file_path": "/tmp/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Demotion back-insertion (MRU positioning for demoted items)**: Consistently improved or maintained hit rate across cycles 0003–0006. Items with tier-advancement history benefit measurably from immediate re-promotion eligibility upon reinsertion. This is the only sustained single-lever win across the entire cohort.\n\n- **Stash-hit filtering before acceleration**: Early cycles (0001–0002) validated that fast-tracking stash hits—items that survived stash discipline—improves hit rate. Success requires fitness gating; unconditional acceleration fails (cycle 0002, v003).\n\n- **Deterministic positioning rules over conditional branching**: Across cycles 0003–0005, simple, unconditional MRU/LRU rules consistently outperformed occupancy-adaptive or threshold-gated insertion logic. Simplicity appears robust; conditional complexity adds noise.\n\n## Recurring failures\n\n- **Frequency-based eviction (LFU)**: Failed in cycle 0001 on recency-dominated workload. Access count is a weaker signal than temporal locality. Not revisited.\n\n- **Occupancy-driven promotion without fitness checks**: Cycles 0002, 0005–0006 repeatedly show that occupancy thresholds suppress rather than improve hit rate when decoupled from item hotness or stash-hit status. Threshold alone is not a fitness signal.\n\n- **MRU positioning for freshly promoted items**: Cycle 0004 revealed an asymmetry: back insertion benefits demoted items but destabilizes newly promoted items. Fresh promotion paths do not benefit from MRU acceleration; they require tier-level filtering first.\n\n- **Tier-skip combined with demotion back-insertion**: Cycles 0004 and 0006 show tier-skip (stash→probing bypass) fundamentally conflicts with demotion-back-insertion discipline. Tier-skip floods the hot tier with unvetted items lacking tier-advancement proof, polluting it and degrading hit rate. The mechanisms are architecturally incompatible.\n\n- **Intra-tier reordering and occupancy-adaptive branching**: Cycle 0004 (v002) showed that reordering within a static tier yields no hit-rate benefit. Cycle 0003 showed occupancy-adaptive destination selection adds complexity without gain. Gains couple only with tier advancement, not with position within a tier.\n\n- **Capacity and geometric tuning (marginal lever)**: Cycle 0006 tested stash expansion and bucket reallocation. Only capacity skewing toward the probing tier yielded improvement (+0.0014), indicating capacity is a secondary lever. Stash buffering expansion failed entirely.\n\n## Open questions\n\n- **Is 0.684 hit rate a true local optimum or a transient plateau?** Six cycles converging on this boundary across orthogonal levers (insertion position, eligibility timing, occupancy thresholds, capacity ratios) suggests architectural constraint. Is the ceiling enforced by tier saturation, eviction cascade timing, or an asymmetry in how promotion and demotion interact? Or is headroom unlocked by untested levers (eviction ordering policies, stash depth thresholds, bucket saturation semantics)?\n\n- **What is the mechanism of the promotion-demotion asymmetry?** Why does MRU insertion benefit demoted items (with proven tier-advancement history) but destabilize freshly promoted items? Is it eviction cadence, promotion candidacy eligibility, tier saturation, or a deeper interaction?\n\n- **Can single-lever strategies compose at all, or is the workload saturated?** Cycles 0004–0006 suggest successful mechanisms have reached interaction limits. Do the demotion back-insertion (v003 cycle 0003) and the marginal capacity skewing (v002 cycle 0006) combine additively, or do they saturate a single underlying constraint?\n\n- **Is tier-by-tier progression inherently superior to tier-skipping for this workload, or does the incompatibility point to a fixable architectural asymmetry?** Tier-skip failed when combined with demotion back-insertion, but succeeded briefly in isolation (cycles 0001–0002). Could a revised tier-skip policy (one that preserves probing-tier filtering or adjusts MRU semantics) unblock it?\n\n## Per-policy notes\n\n- **LRU-based insertion (Front)**: Adequate baseline for fresh promotions but not accelerating. Cycle 0005 confirms front insertion is not the bottleneck when demotion back-insertion is held constant; the constraint lies deeper.\n\n- **MRU-based insertion (Back)**: Highly effective for demoted items consistently across cycles 0003–0006. Harmful or neutral for freshly promoted items (cycle 0004 asymmetry). Only drives gains when coupled with tier advancement, not with intra-tier reordering.\n\n- **Occupancy-based thresholds and adaptive policies**: Repeatedly suppress or fail to improve hit rate (cycles 0002, 0003, 0005–0006). Without fitness gating (hotness, stash-hit status), occupancy is a poor access signal. Deterministic rules outperform conditional occupancy sensing.\n\n- **Tier-skip policy (stash→hot-tier bypass)**: Succeeded briefly in early cycles (0001–0002) as a direct acceleration path for stash hits. Failed in later cycles (0004, 0006) due to architectural conflict with demotion back-insertion. May be incompatible with the filtered tier discipline required for high-confidence hot-tier residents.\n\n- **Tier capacity allocation**: Marginal lever. Cycle 0006 v002 achieved +0.0014 improvement by skewing buckets toward probing (18 vs. 10), but demotion back-insertion remains the dominant mechanism. Capacity tuning alone cannot break the 0.684 ceiling.\n\n"
}
```

### 19:15:59 thinking

I need to write to the correct output directory. Let me check what the correct path should be. The error says the output directory should be in `/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/cross-cycle`. Let me write to that location.

### 19:16:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Demotion back-insertion (MRU positioning for demoted items)**: Consistently improved or maintained hit rate across cycles 0003–0006. Items with tier-advancement history benefit measurably from immediate re-promotion eligibility upon reinsertion. This is the only sustained single-lever win across the entire cohort.\n\n- **Stash-hit filtering before acceleration**: Early cycles (0001–0002) validated that fast-tracking stash hits—items that survived stash discipline—improves hit rate. Success requires fitness gating; unconditional acceleration fails (cycle 0002, v003).\n\n- **Deterministic positioning rules over conditional branching**: Across cycles 0003–0005, simple, unconditional MRU/LRU rules consistently outperformed occupancy-adaptive or threshold-gated insertion logic. Simplicity appears robust; conditional complexity adds noise.\n\n## Recurring failures\n\n- **Frequency-based eviction (LFU)**: Failed in cycle 0001 on recency-dominated workload. Access count is a weaker signal than temporal locality. Not revisited.\n\n- **Occupancy-driven promotion without fitness checks**: Cycles 0002, 0005–0006 repeatedly show that occupancy thresholds suppress rather than improve hit rate when decoupled from item hotness or stash-hit status. Threshold alone is not a fitness signal.\n\n- **MRU positioning for freshly promoted items**: Cycle 0004 revealed an asymmetry: back insertion benefits demoted items but destabilizes newly promoted items. Fresh promotion paths do not benefit from MRU acceleration; they require tier-level filtering first.\n\n- **Tier-skip combined with demotion back-insertion**: Cycles 0004 and 0006 show tier-skip (stash→probing bypass) fundamentally conflicts with demotion-back-insertion discipline. Tier-skip floods the hot tier with unvetted items lacking tier-advancement proof, polluting it and degrading hit rate. The mechanisms are architecturally incompatible.\n\n- **Intra-tier reordering and occupancy-adaptive branching**: Cycle 0004 (v002) showed that reordering within a static tier yields no hit-rate benefit. Cycle 0003 showed occupancy-adaptive destination selection adds complexity without gain. Gains couple only with tier advancement, not with position within a tier.\n\n- **Capacity and geometric tuning (marginal lever)**: Cycle 0006 tested stash expansion and bucket reallocation. Only capacity skewing toward the probing tier yielded improvement (+0.0014), indicating capacity is a secondary lever. Stash buffering expansion failed entirely.\n\n## Open questions\n\n- **Is 0.684 hit rate a true local optimum or a transient plateau?** Six cycles converging on this boundary across orthogonal levers (insertion position, eligibility timing, occupancy thresholds, capacity ratios) suggests architectural constraint. Is the ceiling enforced by tier saturation, eviction cascade timing, or an asymmetry in how promotion and demotion interact? Or is headroom unlocked by untested levers (eviction ordering policies, stash depth thresholds, bucket saturation semantics)?\n\n- **What is the mechanism of the promotion-demotion asymmetry?** Why does MRU insertion benefit demoted items (with proven tier-advancement history) but destabilize freshly promoted items? Is it eviction cadence, promotion candidacy eligibility, tier saturation, or a deeper interaction?\n\n- **Can single-lever strategies compose at all, or is the workload saturated?** Cycles 0004–0006 suggest successful mechanisms have reached interaction limits. Do the demotion back-insertion (v003 cycle 0003) and the marginal capacity skewing (v002 cycle 0006) combine additively, or do they saturate a single underlying constraint?\n\n- **Is tier-by-tier progression inherently superior to tier-skipping for this workload, or does the incompatibility point to a fixable architectural asymmetry?** Tier-skip failed when combined with demotion back-insertion, but succeeded briefly in isolation (cycles 0001–0002). Could a revised tier-skip policy (one that preserves probing-tier filtering or adjusts MRU semantics) unblock it?\n\n## Per-policy notes\n\n- **LRU-based insertion (Front)**: Adequate baseline for fresh promotions but not accelerating. Cycle 0005 confirms front insertion is not the bottleneck when demotion back-insertion is held constant; the constraint lies deeper.\n\n- **MRU-based insertion (Back)**: Highly effective for demoted items consistently across cycles 0003–0006. Harmful or neutral for freshly promoted items (cycle 0004 asymmetry). Only drives gains when coupled with tier advancement, not with intra-tier reordering.\n\n- **Occupancy-based thresholds and adaptive policies**: Repeatedly suppress or fail to improve hit rate (cycles 0002, 0003, 0005–0006). Without fitness gating (hotness, stash-hit status), occupancy is a poor access signal. Deterministic rules outperform conditional occupancy sensing.\n\n- **Tier-skip policy (stash→hot-tier bypass)**: Succeeded briefly in early cycles (0001–0002) as a direct acceleration path for stash hits. Failed in later cycles (0004, 0006) due to architectural conflict with demotion back-insertion. May be incompatible with the filtered tier discipline required for high-confidence hot-tier residents.\n\n- **Tier capacity allocation**: Marginal lever. Cycle 0006 v002 achieved +0.0014 improvement by skewing buckets toward probing (18 vs. 10), but demotion back-insertion remains the dominant mechanism. Capacity tuning alone cannot break the 0.684 ceiling.\n"
}
```

### 19:16:10 thinking

The error message says the path must be under `/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/cross-cycle`. So the file should be written to that directory. Let me adjust the path to go exactly to that directory. The path should be `/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/cross-cycle/cross-cycle-insights.md`.

### 19:16:19 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_18-13-40-938/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Demotion back-insertion (MRU positioning for demoted items)**: Consistently improved or maintained hit rate across cycles 0003–0006. Items with tier-advancement history benefit measurably from immediate re-promotion eligibility upon reinsertion. This is the only sustained single-lever win across the entire cohort.\n\n- **Stash-hit filtering before acceleration**: Early cycles (0001–0002) validated that fast-tracking stash hits—items that survived stash discipline—improves hit rate. Success requires fitness gating; unconditional acceleration fails (cycle 0002, v003).\n\n- **Deterministic positioning rules over conditional branching**: Across cycles 0003–0005, simple, unconditional MRU/LRU rules consistently outperformed occupancy-adaptive or threshold-gated insertion logic. Simplicity appears robust; conditional complexity adds noise.\n\n## Recurring failures\n\n- **Frequency-based eviction (LFU)**: Failed in cycle 0001 on recency-dominated workload. Access count is a weaker signal than temporal locality. Not revisited.\n\n- **Occupancy-driven promotion without fitness checks**: Cycles 0002, 0005–0006 repeatedly show that occupancy thresholds suppress rather than improve hit rate when decoupled from item hotness or stash-hit status. Threshold alone is not a fitness signal.\n\n- **MRU positioning for freshly promoted items**: Cycle 0004 revealed an asymmetry: back insertion benefits demoted items but destabilizes newly promoted items. Fresh promotion paths do not benefit from MRU acceleration; they require tier-level filtering first.\n\n- **Tier-skip combined with demotion back-insertion**: Cycles 0004 and 0006 show tier-skip (stash→probing bypass) fundamentally conflicts with demotion-back-insertion discipline. Tier-skip floods the hot tier with unvetted items lacking tier-advancement proof, polluting it and degrading hit rate. The mechanisms are architecturally incompatible.\n\n- **Intra-tier reordering and occupancy-adaptive branching**: Cycle 0004 (v002) showed that reordering within a static tier yields no hit-rate benefit. Cycle 0003 showed occupancy-adaptive destination selection adds complexity without gain. Gains couple only with tier advancement, not with position within a tier.\n\n- **Capacity and geometric tuning (marginal lever)**: Cycle 0006 tested stash expansion and bucket reallocation. Only capacity skewing toward the probing tier yielded improvement (+0.0014), indicating capacity is a secondary lever. Stash buffering expansion failed entirely.\n\n## Open questions\n\n- **Is 0.684 hit rate a true local optimum or a transient plateau?** Six cycles converging on this boundary across orthogonal levers (insertion position, eligibility timing, occupancy thresholds, capacity ratios) suggests architectural constraint. Is the ceiling enforced by tier saturation, eviction cascade timing, or an asymmetry in how promotion and demotion interact? Or is headroom unlocked by untested levers (eviction ordering policies, stash depth thresholds, bucket saturation semantics)?\n\n- **What is the mechanism of the promotion-demotion asymmetry?** Why does MRU insertion benefit demoted items (with proven tier-advancement history) but destabilize freshly promoted items? Is it eviction cadence, promotion candidacy eligibility, tier saturation, or a deeper interaction?\n\n- **Can single-lever strategies compose at all, or is the workload saturated?** Cycles 0004–0006 suggest successful mechanisms have reached interaction limits. Do the demotion back-insertion (v003 cycle 0003) and the marginal capacity skewing (v002 cycle 0006) combine additively, or do they saturate a single underlying constraint?\n\n- **Is tier-by-tier progression inherently superior to tier-skipping for this workload, or does the incompatibility point to a fixable architectural asymmetry?** Tier-skip failed when combined with demotion back-insertion, but succeeded briefly in isolation (cycles 0001–0002). Could a revised tier-skip policy (one that preserves probing-tier filtering or adjusts MRU semantics) unblock it?\n\n## Per-policy notes\n\n- **LRU-based insertion (Front)**: Adequate baseline for fresh promotions but not accelerating. Cycle 0005 confirms front insertion is not the bottleneck when demotion back-insertion is held constant; the constraint lies deeper.\n\n- **MRU-based insertion (Back)**: Highly effective for demoted items consistently across cycles 0003–0006. Harmful or neutral for freshly promoted items (cycle 0004 asymmetry). Only drives gains when coupled with tier advancement, not with intra-tier reordering.\n\n- **Occupancy-based thresholds and adaptive policies**: Repeatedly suppress or fail to improve hit rate (cycles 0002, 0003, 0005–0006). Without fitness gating (hotness, stash-hit status), occupancy is a poor access signal. Deterministic rules outperform conditional occupancy sensing.\n\n- **Tier-skip policy (stash→hot-tier bypass)**: Succeeded briefly in early cycles (0001–0002) as a direct acceleration path for stash hits. Failed in later cycles (0004, 0006) due to architectural conflict with demotion back-insertion. May be incompatible with the filtered tier discipline required for high-confidence hot-tier residents.\n\n- **Tier capacity allocation**: Marginal lever. Cycle 0006 v002 achieved +0.0014 improvement by skewing buckets toward probing (18 vs. 10), but demotion back-insertion remains the dominant mechanism. Capacity tuning alone cannot break the 0.684 ceiling.\n"
}
```

### 19:16:25 thinking

Great! The file has been created successfully. Let me verify this is the output the user expects by reviewing what I wrote:

The file `cross-cycle-insights.md` has been created with the four required sections:

1. **Recurring wins** - Patterns that have held up across multiple cycles:
   - Demotion back-insertion as the only sustained single-lever win
   - Stash-hit filtering before acceleration (early wins)
   - Deterministic positioning over conditional branching

2. **Recurring failures** - Approaches that keep not paying off:
   - Frequency-based eviction (LFU)
   - Occupancy-driven promotion without fitness checks
   - MRU for freshly promoted items (asymmetry)
   - Tier-skip with demotion back-insertion (architectural conflict)
   - Intra-tier reordering and occupancy-adaptive branching
   - Capacity and geometric tuning (marginal returns)

3. **Open questions** - Unresolved threads:
   - Is 0.684 a true local optimum?
   - What's the promotion-demotion asymmetry mechanism?
   - Can strategies compose at all?
   - Can tier-skip be salvaged?

4. **Per-policy notes** - Observations grouped by eviction/insertion policy:
   - LRU-based insertion
   - MRU-based insertion
   - Occupancy-based thresholds
   - Tier-skip policy
   - Tier capacity allocation

The analysis synthesizes findings across all 6 cycles, avoiding single-cycle observations and focusing on cross-cycle patterns. It's concise and uses qualitative terms rather than inventing numbers.

### 19:16:28 chat

Done! I've created `cross-cycle-insights.md` by analyzing all six completed cycles and extracting cross-cycle patterns.

## Summary of findings:

**Recurring wins:**
- Demotion back-insertion (MRU for demoted items) is the only sustained single-lever win across all cycles
- Stash-hit filtering before acceleration worked in early cycles when gated by fitness
- Deterministic positioning rules outperformed conditional branching across multiple cycles

**Recurring failures:**
- Frequency-based eviction (LFU) failed on this recency-dominated workload
- Occupancy-driven acceleration without fitness checks repeatedly suppresses hit rate
- MRU positioning for freshly promoted items destabilizes performance (asymmetry with demotion recovery)
- Tier-skip conflicts fundamentally with demotion back-insertion due to architectural incompatibility
- Intra-tier reordering and capacity tuning yield negligible gains

**Key constraint:**
The 0.684 hit-rate ceiling appeared consistently across six cycles of testing orthogonal levers (insertion position, eligibility timing, occupancy thresholds, capacity allocation), suggesting it's defended by architectural asymmetries rather than any single tunable parameter.

**Open questions:**
- Is 0.684 a true optimum or is there headroom via untested levers (eviction ordering, stash depth, saturation semantics)?
- Why does MRU help demoted items but hurt fresh promotions?
- Can strategies actually compose, or is the workload saturated?

The file is ready for future hypothesis-writer cycles to use as long-memory context.


---

- finished: 2026-06-05T19:16:28+00:00
- duration: 67.21s
- status: ok
- input_tokens: 26
- output_tokens: 7355
- cache_read_tokens: 74764
- cache_write_tokens: 13235
- cost_usd: 0.06611
