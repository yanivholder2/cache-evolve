# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0008\cross-cycle
- allow_edits: True
- started: 2026-07-21T04:32:57+00:00

## timeline

### 04:33:02 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me analyze the patterns across all 8 cycles and synthesize insights for the four fixed sections.

First, let me find the output directory.

### 04:33:03 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 04:33:03 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 04:33:05 tool: Bash

```json
{
  "command": "ls /c/Users/yaniv/Projects/cache-evolve/ 2>/dev/null || ls ~/Projects/cache-evolve/ 2>/dev/null || echo \"not found\"",
  "description": "List project root contents"
}
```

### 04:33:07 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 04:33:10 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 04:33:10 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 04:33:40 thinking

Now I know the output directory: `C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0008\cross-cycle`. I should write `cross-cycle-insights.md` there.

Let me now analyze the ledger digest across all 8 cycles and synthesize the insights.

**Recurring wins:**
1. MRU-end insertion (Cycle 1) — improved hit-rate and per-op time, held across all subsequent cycles as a stable anchor
2. Timestamp/struct field removal (Cycle 1) — dual dividend: smaller footprint + eliminated syscalls, zero hit-rate penalty
3. Stacking orthogonal mechanisms additively across cycles — independently-validated wins on non-overlapping subsystems reliably compose without interference
4. Vacancy-gating as cascade eliminator (Cycle 3) — structural constraint prevents cascade evictions entirely, outperforming threshold tuning which only softens the problem
5. Two-chance clock early-exit on Zipfian workloads (Cycle 6) — delivers frequency-aware eviction without O(n) scan overhead

**Recurring failures:**
1. O(n) frequency scans for LFU-minimum victim selection reliably inflate put p99 every time they appear across Cycles 5 and 6
2. Stash tier interventions across multiple cycles — the stash is too small a fraction of capacity and the conditions that move items there produce low-frequency items, yielding no hit-rate payoff despite repeated attempts
3. Counter decay on stationary workloads undermines promotion gates and adds O(n) scan overhead with no workload-matched benefit
4. Raising frequency threshold beyond ≥2 causes net hit-rate loss because genuinely re-usable items cluster in the 2-3 range
5. Probing re-entry vacancy gate — structurally sound but fires too rarely at high cache occupancy
6. Clock and frequency-counter sharing corrupts both signals since clock assumes counter-zero cold items while promotion machinery requires non-zero counters

Now I'm considering whether a separate boolean flag set only when an item is rescued from eviction could decouple clock eviction from the frequency promotion signal, or if a sparser CLOCK_BIT condition tied to higher counter thresholds could give the clock discriminating power without corrupting the counter. I'm also wondering if there are other structural overhead fields in the item struct beyond timestamp, and whether combining recency and frequency or applying counter decay over time could complement the ≥2 vacancy gate.

The effectiveness of the stash tier and probing re-entry mechanisms depends on the cache-to-working-set ratio, and non-stationary workloads where counter decay or stash bypass become relevant will need separate investigation. Looking at the per-policy results: LRU's baseline placement at the LRU-end was actively harmful until flipping to MRU-end, LFU-minimum eviction confirmed hit-rate improvements but the O(n) bucket scan consistently inflates put p99, and the two-chance clock policy emerged as the best frequency-aware eviction approach — early exit on Zipfian cold-tail eliminates scan cost and timestamp removal multiplies the gain, though it becomes O(k·n) when all counters are non-zero.

Vacancy-gated frequency promotion with a ≥2 threshold provides reliable hit-rate lift without regressions and stacks cleanly with MRU-end insertion and timestamp removal, while counter decay proved harmful on the stationary workload and stash cold-preferring eviction remained latency-cheap but hit-rate neutral since the stash tier is too small to shift the aggregate miss curve. Now I'm writing up the results.

### 04:33:40 chat

Now I have the output path. I'll synthesize the 8-cycle ledger and write the insights file.

### 04:34:08 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\dash_2026-07-19_11-08-02-674\\cycles\\0008\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **MRU-end insertion** (Cycle 1, stable anchor ever since): flipping insertion from LRU-end to MRU-end improved hit-rate and per-op time simultaneously; the baseline's placement was actively working against the tier hierarchy's warm-up logic.\n- **Struct field elimination** (Cycle 1): dropping dead-weight per-item metadata (e.g., `timestamp`) produces a dual dividend — smaller footprint and eliminated syscalls — with zero hit-rate penalty; the wins survive stacking with every subsequent mechanism.\n- **Orthogonal mechanisms stack additively**: wins on non-overlapping subsystems (insertion position, struct layout, promotion gating, eviction selection) have reliably composed without interaction when applied to clearly separated code paths; stacking was empirically validated across Cycles 1→2 and 5→6.\n- **Vacancy-gating as cascade prevention** (Cycle 3): constraining promotion fires to open slots eliminates cascade evictions by construction, delivering regression-free hit-rate lifts that threshold tuning alone cannot match.\n- **Two-chance clock on Zipfian workloads** (Cycle 6): rotating warm items to MRU-end and stopping at the first counter-zero cold item terminates early where cold-tail items accumulate; this achieves frequency-aware eviction without an O(n) scan, improving all three measured dimensions simultaneously.\n\n## Recurring failures\n\n- **O(n) LFU-minimum victim scans**: every variant that scans a full bucket for the global minimum inflates put p99 consistently (Cycles 5, 6); the hit-rate gain is real but the cost is non-trivial and reproducible regardless of what else is combined with it.\n- **Stash-tier interventions** (Cycles 2, 3, 4, 8 — bypass, cold eviction, frequency gates): the stash represents too small a fraction of capacity, and cascade-evicted items arriving there carry low frequency counts, so gate conditions fire too infrequently to shift the aggregate miss curve.\n- **Raising frequency threshold above ≥ 2** (Cycle 4): real re-usable items sit in the 2–3 access range; stricter filtering ejects them, producing a net hit-rate loss rather than the expected gain.\n- **Counter decay on a stationary workload** (Cycle 6): post-eviction counter halving suppresses legitimately earned promotion eligibility and adds a second O(n) scan; no tested scenario on the current trace benefits from aging out popularity.\n- **Shared `lfu_counter` for clock warmth and promotion gating** (Cycle 7): the clock assumes counter-zero means cold; the promotion gate assumes non-zero means warm; using the same field for both purposes corrupts both signals and turns the clock's early-exit into O(k·n) rotation when the base is counter-rich.\n- **Probing re-entry vacancy gate** (Cycles 3, 5, 7): structurally sound and always regression-free, but persistently thin — the joint condition (item at threshold AND target has vacancy) fires too rarely at high cache occupancy to produce a measurable hit-rate gain.\n\n## Open questions\n\n- Can a separate boolean \"chance bit\" — set only when an item is rescued from eviction, not on every access — decouple clock eviction from the frequency promotion counter, resolving the Cycle 7 conflict?\n- Is there a sparser CLOCK_BIT condition (e.g., set only when `lfu_counter` crosses a second, higher boundary) that gives the clock enough discriminating power without over-marking warm items on every non-hottest access?\n- Are there additional dead-weight struct fields beyond `timestamp` still present in the item layout that could be eliminated analogously to Cycle 1's discovery?\n- Do eviction policy improvements (clock or LFU-min) interact differently at lower cache-to-working-set ratios, where cold-tail items are a smaller fraction of resident items and the clock's early-exit advantage shrinks?\n- At what cache occupancy or workload shape do the stash-bypass and probing re-entry vacancy-gate mechanisms begin firing at a rate sufficient to produce measurable hit-rate changes?\n- Are there promotion axes orthogonal to the frequency counter — for example, recency combined with frequency, or tier-residency duration — that could improve selectivity without a threshold knob?\n\n## Per-policy notes\n\n- **LRU (baseline)**: MRU-end insertion flip is the single highest-impact change found; baseline's LRU-end placement was its most costly structural choice and was corrected in Cycle 1.\n- **LFU-minimum eviction**: confirmed independent hit-rate lift (Cycle 5); stacks with vacancy-gated promotion (Cycle 5); consistently inflates put p99 due to O(n) scan — the gain is real but the latency cost is structural and has not been eliminated by any variant tested.\n- **Two-chance clock eviction**: best frequency-aware eviction policy found to date (Cycle 6); timestamp removal multiplies the gain by improving cache-line utilization during traversal; advantage degrades to O(k·n) when nearly all bucket items have non-zero counters (counter-rich base from promotion machinery), making decoupling of the warmth signal a prerequisite for composing it with frequency-gated promotion.\n- **Vacancy-gated frequency promotion (≥ 2 threshold)**: reliable regression-free hit-rate lift; ≥ 2 appears near-optimal for this workload — raising to ≥ 3 caused a marginal net loss (Cycle 4); stacks cleanly with MRU-end insertion and struct reduction.\n- **Counter decay**: harmful on a stationary popularity distribution; untested on a shifting workload — deprioritized until a non-stationary trace is available.\n- **Stash cold-preferring eviction** (Cycle 8 v003): latency-safe because it fires only at full-bucket stash insertions rather than on every access; hit-rate neutral on current workload because the stash tier is too small; could become relevant on a workload where stash holds a larger share of total capacity.\n"
}
```

### 04:34:17 chat

`cross-cycle-insights.md` written to `outputs/dash_2026-07-19_11-08-02-674/cycles/0008/cross-cycle/`. Here's a summary of what the file captures across all eight cycles:

**Recurring wins** — five durable patterns: MRU-end insertion (Cycle 1, never reversed), struct-field elimination, additive stacking of orthogonal mechanisms, vacancy-gating as cascade prevention, and the two-chance clock's early-exit on Zipfian cold-tail items.

**Recurring failures** — six anti-patterns: O(n) LFU-minimum scans (consistently inflate put p99), stash-tier interventions (tier too small, gate conditions too rare), threshold-tightening beyond ≥ 2 (ejects genuinely reusable items), counter decay on a stationary workload, sharing `lfu_counter` between clock and promotion gate (corrupts both signals), and the probing re-entry vacancy gate (structurally safe but persistently thin at high occupancy).

**Open questions** — focus on the Cycle 7/8 unsolved conflict: can a separate "chance bit" (set only on eviction rescue, not every access) decouple the clock from the frequency counter? Also flags the possibility of more dead-weight struct fields and the conditions under which stash/probing re-entry become effective.

**Per-policy notes** — individual assessments of LRU baseline, LFU-minimum, two-chance clock, vacancy-gated promotion, counter decay, and stash cold eviction — grouped by policy rather than cycle.


---

- finished: 2026-07-21T04:34:18+00:00
- duration: 80.51s
- status: ok
- input_tokens: 7
- output_tokens: 3507
- cache_read_tokens: 121505
- cache_write_tokens: 12194
- cost_usd: 0.14143
