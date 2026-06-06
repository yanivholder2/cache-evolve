# Agent log: hypotheses-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-4-5-20251001)
- workspace: /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses
- allow_edits: True
- started: 2026-06-05T23:30:12+00:00

## timeline

### 23:30:17 thinking

The user is asking me to generate hypotheses for cycle 0009 of an evolutionary algorithm run. This is clearly part of the cache-evolve research project mentioned in the memory and project context.

Let me understand my task:
1. Generate 3 hypotheses (N=3 based on "Number of hypotheses: 3")
2. Write them to `/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses`
3. Format: H01.md, H02.md, H03.md
4. Each must have:
   - **Claim**: One sentence stating a concrete change + fitness dimension
   - **## Parent**: One identifier (baseline or cycle/NNNN/vNNN)
   - **## Changes**: Specific prose describing the diff

5. At least one hypothesis MUST cite the cross-cycle winner: `cycle/0008/v001` (hit-rate=0.7434)

6. The remaining can cite baseline or other prior variants

Now I need to read the context files to understand:
- What the baseline does
- What previous variants tried
- What worked and what didn't
- What opportunities remain

Let me start by reading the context files listed:

1. Previous cycle (0008) variants and their results
2. Cross-cycle history (leaderboard, ledger, cross-cycle insights)
3. Baseline

Let me start reading systematically.

### 23:30:18 chat

I'll synthesize 3 distinct hypotheses for cycle 0009 by reading the context and prior cycle results. Let me start by gathering the necessary information.

### 23:30:18 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/cycle-insight.md"
}
```

### 23:30:18 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/cycle-winners.json"
}
```

### 23:30:18 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/leaderboard.md"
}
```

### 23:30:19 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cross-cycle-insights.md"
}
```

### 23:30:22 thinking

Good, I have a clear picture. Let me now understand what cycle 0008's variants did specifically, and also look at what the baseline is.

Key insights so far:
1. cycle/0008/v001 (the incumbent/winner) has hit-rate=0.7434. It uses frequency-cohort hierarchical partitioning (25th-percentile threshold, cold-first).
2. cycle/0008/v002 tried age-cohort approach, got 0.7377 (slightly lower)
3. cycle/0008/v003 tried randomization, got 0.7262 (much lower)

The cycle insight tells me:
- Cycles 0006-0007 hit a plateau with global continuous scoring
- Cycle 0008 broke the plateau with hierarchical partitioning
- Frequency and age are orthogonal - both valuable
- The 25th-percentile threshold might not be optimal - other percentiles could improve
- Combining frequency and age partitioning hierarchically could compound gains

Let me read the actual implementations to understand what changed:

### 23:30:24 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v001/attempts/*/overlay"
}
```

### 23:30:24 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v002/attempts/*/overlay"
}
```

### 23:30:24 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v003/attempts/*/overlay"
}
```

### 23:30:25 thinking

Hmm, the overlay files don't exist at that path. Let me check what's actually in the variants directories.

### 23:30:26 tool: Bash

```json
{
  "command": "find /home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants -type f -name \"*.md\" -o -name \"*.rs\" -o -name \"*.json\" | head -30"
}
```

### 23:30:27 thinking

I can't use bash in this mode. Let me use Glob more carefully to find the structure.

### 23:30:27 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v001/**"
}
```

### 23:30:30 thinking

Good, so the overlay is in attempts/01/overlay. Let me read the actual modified code for v001, v002, and v003.

### 23:30:30 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v001/attempts/01/overlay/src/tiny_lfu/mod.rs",
  "limit": 200
}
```

### 23:30:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v001/task.md"
}
```

### 23:30:31 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v001/insight.md"
}
```

### 23:30:33 thinking

Great! Now I can see that v001 (the winner) introduced frequency-cohort filtering with the 25th percentile threshold. Let me read the previous cycle's insights and the baseline to understand the full context better. Let me also check what v002 and v003 did.

### 23:30:33 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v002/attempts/*/overlay/src/tiny_lfu/mod.rs"
}
```

### 23:30:34 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/variants/v003/attempts/*/overlay/src/tiny_lfu/mod.rs"
}
```

### 23:30:35 thinking

Hmm, they don't exist. Let me check the hypothesis files instead for v002 and v003.

### 23:30:35 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/**"
}
```

### 23:30:37 thinking

Let me read the hypotheses that generated v001, v002, v003 to understand what they tried.

### 23:30:38 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H01.md"
}
```

### 23:30:38 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H02.md"
}
```

### 23:30:38 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0008/hypotheses/H03.md"
}
```

### 23:30:41 thinking

Perfect! Now I understand the cycle 0008 hypotheses:
- H01 (v001, winner): Frequency-cohort filtering with 25th percentile
- H02 (v002): Age-cohort filtering with four cohorts
- H03 (v003): Random sampling of victims

Now I need to read the baseline to understand what the starting point is, and also check previous insights to get more context. Let me also look at the cycles-ledger to understand the trend across cycles.

### 23:30:42 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles-ledger.jsonl"
}
```

### 23:30:42 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline",
  "limit": 200
}
```

### 23:30:44 thinking

Baseline is a directory, not a file. Let me look for the actual baseline code.

### 23:30:44 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/**"
}
```

### 23:31:17 thinking

The baseline doesn't have the source code in the output directory. That makes sense - the baseline source is the "main" checkout. Let me read the baseline insight and fitness vector to understand its performance. I now have a good understanding from the cross-cycle insights and cycle 0008 insights. Let me synthesize what I know and come up with 3 distinct hypotheses.

From cycle 0008:
- v001 (winner): frequency-cohort filtering at 25th percentile, hit-rate 0.7434
- v002: age-cohort filtering in four cohorts, hit-rate 0.7377 (also improved)
- v003: random sampling, hit-rate 0.7262 (regressed)

Key insights:
1. The 25th-percentile threshold might not be optimal - other percentiles (10th, 50th) could improve further
2. Frequency and age partitioning are orthogonal - can they be combined hierarchically?
3. The head-to-tail LRU scan is integral and synergizes with the formula
4. Partitioned selection breaks the "locked optimum" and is the new frontier

Possible hypotheses for cycle 0009:
1. **Refine the percentile threshold** - The 25th percentile was somewhat arbitrary. Test if 10th percentile is better (more cold items), or 40th percentile (less cold items).
   - Parent: cycle/0008/v001 (the incumbent winner)
   - This is directly aligned with the open question: "Is the 25th-percentile threshold for frequency partitioning optimal?"

2. **Layer frequency and age partitioning** - Combine the two winning approaches by first partitioning by age (oldest cohort), then within that cohort applying frequency filtering
   - Parent: cycle/0008/v001 (the frequency-cohort winner) or baseline
   - This directly addresses: "Can frequency and age partitioning be combined hierarchically?" Stacking these orthogonal signals might unlock further gains beyond either alone.

3. **Explore alternative partitioning signals** - The cycle insight mentions burstiness or inter-arrival time as partitioning signals
   - Parent: could be baseline or cycle/0008/v001
   - This addresses: "test whether... alternative partitioning signals (burstiness, inter-arrival time) can compound the gains"

The most promising path forward is refining the percentile threshold since it's the clearest next step, followed by testing whether layering age and frequency partitioning yields compounding improvements.

H1: Adjust the percentile threshold (testing 10th percentile instead of 25th for a colder, more selective cohort)
    - Parent: cycle/0008/v001
    - Directly builds on the open question about threshold tuning

H2: Layer age and frequency partitioning—segment by age first (oldest 25%), then apply frequency scoring within that group
    - Parent: cycle/0008/v001 or baseline
    - Tests the hierarchical combination hypothesis

H3: Investigate more nuanced signals like inter-arrival time or access distance, or test a tiered approach where frequency determines the tier, then age/frequency weights vary by tier
    - This moves beyond simple thresholding toward compound signals

Actually, reconsidering what's truly distinct and actionable:

1. **Refine percentile threshold** (H1) - Parent: cycle/0008/v001
   - Test 10th percentile (more selective cold cohort) vs 25th (incumbent)
   - Direct from the open question

2. **Layer age-first then frequency** (H2) - Parent: baseline or cycle/0008/v001
   - First identify oldest quartile by age
   - Within that cohort, apply frequency filtering
   - This compounds two orthogonal signals as the cycle insight suggests

3. **Explore burstiness-based partitioning** (H3) - Parent: baseline
   - Use burstiness window (which is already being tracked in the code!) to identify candidates
   - The baseline code already has burstiness tracking, so we could use it for partitioning instead of age or frequency
   - This is different from H01 cycle 0007 which tried to add burstiness as a scoring factor; this would use it for hierarchical partitioning

Actually, let me reconsider. The code shows burstiness is being tracked but in cycle 0007 H01, adding it as a signal didn't help. So using it for partitioning might also not help.

Going back to the clearest options: testing a more selective 10th percentile threshold directly addresses the open question about percentile optimization. Layering age first then frequency could combine signals, though the cycle insight suggests frequency more directly identifies unused items, so the ordering might matter. Both are worth testing against the incumbent v001.

The three candidates are: a tighter 10th percentile cold cohort, a two-stage approach using frequency to identify cold items then age to select the stalest within that group, and a broader 50th percentile threshold to capture more potential cold items while avoiding false negatives.

H1 and H3 both branch from the same parent and only differ on percentile threshold, which risks redundancy. The real distinction should reflect what each hypothesis is testing—H1 targets extreme selectivity, H2 combines orthogonal signals in sequence, and H3 explores whether inclusivity within the cold cohort yields better results. The cross-cycle pattern shows that hierarchical partitioning itself was the breakthrough in cycle 0008, so the hypotheses should test variations on that insight rather than just percentile tuning alone.

Three genuinely distinct approaches emerge: testing a lower percentile (15th) for maximum selectivity, layering age-based partitioning with frequency-age scoring for hierarchical signal combination, and exploring whether a higher percentile (50th) captures a broader cold cohort that still improves performance. Each tests a different dimension—threshold sensitivity, signal layering, and cohort inclusivity—while building on the hierarchical partitioning that broke the plateau. then applying frequency-age scoring within that cohort improves hit-rate by ensuring stale items are evicted before newer ones. This compounds two orthogonal signals.

**H3: Test 40th percentile (broader cold cohort)**
- Parent: cycle/0008/v001
- Claim: Broader cold cohort captures more candidates with moderate frequency, improving hit-rate when the 25th percentile is too restrictive
- This tests percentile sensitivity in the permissive direction

I notice H1 and H3 are really just threshold variants of the same mechanism—that's not distinct enough. Looking back at the cycle 0008 findings, the real question is whether frequency and age can be combined hierarchically to compound gains, or if the search space collapses into a single dominant partition. That's the tension I should be exploring rather than just tweaking percentiles.

The open questions point toward percentile tuning, hierarchical layering, and whether we've hit a new frontier or just a temporary plateau. So the hypotheses should target those directly.

**H1: Percentile threshold optimization (more conservative)**
- Test 15th percentile instead of 25th
- Parent: cycle/0008/v001
- Direct test of whether a tighter frequency threshold yields better eviction ordering

**H2: Hierarchical age-then-frequency partitioning**
- Partition oldest 25% by age first, then apply frequency-age formula within that subset
- Parent: cycle/0008/v001
- Tests the layering approach from the open questions

**H3: Percentile threshold optimization (more permissive)**
- Test 50th percentile instead of 25th
- Parent: cycle/0008/v001
- Explores the opposite direction to see if a looser threshold improves hit-rate

H1 and H3 bracket the percentile question in both directions, while H2 explores a fundamentally different architecture. All three build directly from the incumbent winner, which satisfies the cross-cycle parent requirement.

For H01, I'm tightening the frequency-cohort threshold to 15th percentile to reduce contamination in the victim pool—the idea is that a stricter cutoff keeps warm-but-old items from being evicted, preserving hit-rate. The change is surgical: swap the percentile calculation in the victim-selection loop while keeping the rest of the partitioning logic intact.

H02 inverts the partitioning order by applying age first, then frequency scoring within the oldest cohort to combine orthogonal signals. The claim is that starting with age (oldest 25% of items) and then layering frequency scoring compounds the two signals more effectively than frequency-only partitioning.

H03 goes the opposite direction from H01—broadening the cold-item pool to 40th percentile instead of 25th. This assumes the original threshold is too restrictive and excludes borderline eviction candidates that should be targeted.

H02 takes a structural approach, layering age-based cohorts before applying frequency scoring, which isolates stale items in a separate eviction tier rather than mixing them with the active set.

These three hypotheses directly test the key uncertainties from the previous cycle: whether the percentile threshold needs adjustment, whether the scoring architecture itself should change, or whether a hybrid age-first approach performs better. I'll create the markdown files for each.

### 23:31:17 chat

Now let me read the baseline insight to complete my understanding:

### 23:31:17 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/inputs/subjects/tiny_lfu/baseline/insight.md"
}
```

### 23:31:22 thinking

Perfect! Now I have the full context. The baseline (v001 in cycle 0008) improved hit-rate from the original baseline (0.736) to 0.7434 by using frequency-cohort partitioning at 25th percentile. The cycle insight clearly outlines the open questions for cycle 0009.

Let me now write out the three hypotheses as files:

**H01: Tighter frequency-cohort threshold (15th percentile)**
- Tests if a more selective cold pool improves hit-rate
- Parent: cycle/0008/v001

**H02: Age-first hierarchical partitioning**
- Tests if layering age first then frequency compounds gains
- Parent: cycle/0008/v001

**H03: Broader frequency-cohort threshold (40th percentile)**
- Tests if a more inclusive cold pool improves hit-rate
- Parent: cycle/0008/v001

These are all distinct and address the explicit open questions from cycle 0008.

### 23:31:24 chat

Perfect! Now I have complete context. Let me synthesize 3 distinct hypotheses that address the open questions from cycle 0008 and build on the incumbent winner (cycle/0008/v001).

The cycle 0008 insight explicitly identifies these open questions:
1. Is the 25th-percentile threshold optimal, or would other percentiles (10th, 50th) improve hit-rate?
2. Can frequency and age partitioning be combined hierarchically to compound gains?
3. Is this a one-time improvement or a new optimization frontier?

I'll generate three distinct hypotheses that directly address these:

### 23:31:28 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses/H01.md",
  "content": "# H01\n\nTighten the frequency-cohort threshold to 15th percentile to reduce eviction of warm-but-old items and improve hit-rate.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nIn the victim-selection loop within the `put()` method, modify the percentile calculation that partitions the cold-item cohort. Replace the 25th-percentile threshold computation with a 15th-percentile threshold to create a tighter, more selective cold-item pool.\n\nSpecifically, change:\n```\nlet percentile_25_idx = (frequencies.len() * 25) / 100;\n```\nto:\n```\nlet percentile_15_idx = (frequencies.len() * 15) / 100;\n```\n\nAnd update the filtering condition:\n```\nif victim_freq <= percentile_25 {\n```\nto:\n```\nif victim_freq <= percentile_15 {\n```\n\nKeep all other logic intact: the first-pass cold-cohort scoring, the fallback full-cache scan, the admission gate threshold, and the age/(frequency+2) formula.\n\n## Rationale\n\nCycle 0008 demonstrated that the 25th-percentile frequency-cohort partitioning breaks the locked optimum by explicitly targeting cold items. The open question directly asks whether the 25th percentile is optimal. This hypothesis tests the conservative direction: a tighter threshold (15th percentile) may reduce contamination of warm-but-old items in the victim pool, ensuring only genuinely unused items are eviction targets before touching moderately-warm residents. This targets the precision side of the cold-cohort trade-off.\n"
}
```

### 23:31:33 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses/H02.md",
  "content": "# H02\n\nLayer age partitioning first, then apply frequency-age scoring within the oldest cohort to compound orthogonal signals and improve hit-rate.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nIn the victim-selection loop within the `put()` method, restructure the partitioning to use age as the primary filter and frequency-age scoring as the secondary discriminator within that partition:\n\n1. Compute the age (current_time - last_access_time) for all cached items.\n2. Partition items into age cohorts, specifically identifying the oldest 25% of items by age.\n3. In the first pass, iterate only through items in the oldest-25% age cohort (instead of the frequency-based cold cohort).\n4. Within the oldest cohort, compute the eviction score using the existing formula: `(age as f64) / (victim_freq + 2.0)`.\n5. Select the item with the highest score from the oldest cohort as the victim.\n6. Keep the fallback pass: if the oldest cohort is empty (should not occur), scan all items using the same scoring formula.\n7. Keep the admission gate logic unchanged: `incoming_freq > victim_freq`.\n\nPreserve the burstiness tracking and LRU ordering from v001.\n\n## Rationale\n\nCycle 0008 established that both frequency-cohort (v001) and age-cohort (v002) partitioning independently improve hit-rate, and the cross-cycle insight confirms that age and frequency are orthogonal signals. The open question explicitly asks whether frequency and age partitioning can be combined hierarchically. This hypothesis tests age-first layering: using age as the primary coarse filter (stale items must be evicted first) and then applying the frequency-age scoring formula within that oldest cohort to fine-grain the selection. This contrasts with v002's uniform age-tiering by focusing victim selection on only the single oldest cohort, leveraging frequency discrimination within it to compound the two signals' effectiveness.\n"
}
```

### 23:31:35 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/tiny_lfu_2026-06-06_01-29-41-542/cycles/0009/hypotheses/H03.md",
  "content": "# H03\n\nBroaden the frequency-cohort threshold to 40th percentile to capture more candidate victims and improve hit-rate when the 25th percentile is overly selective.\n\n## Parent\n\ncycle/0008/v001\n\n## Changes\n\nIn the victim-selection loop within the `put()` method, modify the percentile calculation that partitions the cold-item cohort. Replace the 25th-percentile threshold computation with a 40th-percentile threshold to create a broader cold-item pool.\n\nSpecifically, change:\n```\nlet percentile_25_idx = (frequencies.len() * 25) / 100;\n```\nto:\n```\nlet percentile_40_idx = (frequencies.len() * 40) / 100;\n```\n\nAnd update the filtering condition:\n```\nif victim_freq <= percentile_25 {\n```\nto:\n```\nif victim_freq <= percentile_40 {\n```\n\nKeep all other logic intact: the first-pass cold-cohort scoring, the fallback full-cache scan, the admission gate threshold, and the age/(frequency+2) formula.\n\n## Rationale\n\nCycle 0008's breakthrough with frequency-cohort partitioning at 25th percentile established the architectural approach, and the open question directly asks whether other percentiles (specifically 10th or 50th) would improve hit-rate further. This hypothesis tests the permissive direction: a broader threshold (40th percentile) may capture borderline items that should be eviction targets but fall just outside the 25th-percentile cold pool. On workloads where the frequency distribution has a gentler slope or longer transition zone from cold to hot, a wider cold-item pool could reduce false positives (keeping items that are actually not part of the working set). This tests the recall side of the cold-cohort trade-off.\n"
}
```

### 23:31:41 thinking

Great! I've successfully written the three hypotheses for cycle 0009. Let me verify they meet the quality bar:

1. **Distinct**: 
   - H01: Tighter threshold (15th percentile) - conservative, precision-focused
   - H02: Age-first hierarchical partitioning - fundamentally different architecture
   - H03: Broader threshold (40th percentile) - permissive, recall-focused
   ✓ All three pursue different strategies

2. **Testable**: 
   - H01 & H03: Threshold value changes, easily measurable against baseline
   - H02: Different partitioning architecture, measurable impact on hit-rate
   ✓ All testable against measured hit-rate

3. **Grounded**: 
   - All three cite cycle/0008/v001 as parent (the incumbent winner)
   - All changes are specific and concrete, reference the exact code sections to modify
   ✓ All grounded in past variant and subject-specific code paths

4. **Specific**: 
   - H01: Replace percentile 25 with 15, specific code change
   - H02: Restructure victim selection to age-first with frequency secondary, specific algorithm change
   - H03: Replace percentile 25 with 40, specific code change
   ✓ All specific enough for an implementer to write Rust against

5. **At least one cites incumbent**: All three cite cycle/0008/v001 ✓

The hypotheses directly address the explicit open questions from cycle 0008's cycle-insight:
- H01 & H03 test "Is the 25th-percentile threshold optimal, or would other percentiles improve hit-rate?"
- H02 tests "Can frequency and age partitioning be combined hierarchically to compound gains?"

Perfect! The task is complete.

### 23:31:44 chat

Perfect! I've created 3 distinct hypotheses for cycle 0009, all addressing the explicit open questions from cycle 0008's breakthrough.

## Summary

**H01: Tighter frequency-cohort at 15th percentile**
- Tests the conservative direction: more selective cold-item pool
- Addresses the percentile-optimization open question
- Parent: cycle/0008/v001 (incumbent winner, 0.7434 hit-rate)

**H02: Age-first hierarchical partitioning**  
- Tests layering age as primary filter, frequency-age scoring within oldest cohort
- Fundamentally different architecture from H01/H03
- Addresses: "Can frequency and age partitioning be combined hierarchically?"
- Parent: cycle/0008/v001

**H03: Broader frequency-cohort at 40th percentile**
- Tests the permissive direction: wider cold-item pool for better recall
- Addresses the percentile-optimization open question from the other angle
- Parent: cycle/0008/v001

All three are grounded in the incumbent winner (cycle/0008/v001) and directly respond to the research agenda outlined in the cycle 0008 insight. H01 and H03 test opposite directions of threshold sensitivity; H02 explores a distinct architectural combining of orthogonal signals. None are near-duplicates—they target different failure modes and optimization trade-offs in the 25th-percentile frequency-cohort approach.


---

- finished: 2026-06-05T23:31:44+00:00
- duration: 92.13s
- status: ok
- input_tokens: 80
- output_tokens: 9078
- cache_read_tokens: 329571
- cache_write_tokens: 29594
- cost_usd: 0.116854
