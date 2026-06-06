# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-06T01:45:56+00:00

## timeline

### 01:45:59 thinking

Let me analyze the ledger digest across all 10 cycles and write a fresh `cross-cycle-insights.md` file. I need to find the output directory first.

### 01:46:00 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 01:46:00 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 01:46:02 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/"
}
```

### 01:46:04 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve"
}
```

### 01:46:06 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/cross-cycle"
}
```

### 01:46:28 thinking

The output directory is `/home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/cross-cycle`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize the 10 cycles into the four fixed sections.

**Recurring wins:**
- Two-segment SLRU (protecting a reuse-proven hot set from probationary newcomers) — the foundation since cycle 1
- Frequency as the primary eviction signal — outperforms accumulated-only or pure-IRR signals on this workload
- Counter halving on demotion (preserving signal across segment boundary) — critical since cycle 3
- Scanning the LRU half of probationary (not just the quarter) — load-bearing since cycle 4
- Ghost admission complementing frequency-primary eviction — confirmed additive in cycles 2→10 (with the caveat that it's harmful with IRR)
- Staleness as tie-breaker — validated on two different baselines (cycle 9 and 10)

**Recurring failures:**
- Ghost admission with IRR eviction — structurally harmful because ghost-admitted entries lack a second timestamp, making their IRR = MAX
- Fixed-rate counter decay — collapses frequency distinctions whenever the decay constant misaligns with the inter-access distribution
- Expanding probationary beyond 20% — the hot working set grows too large, triggering excessive demotion churn
- Tightening scan-fraction in probationary — frequency and IRR variation spreads across the segment rather than concentrating near the LRU head
- Growing the ghost window past probationary capacity — distant events represent cold traffic, not deferred-warm candidates
- Secondary tie-breakers — collisions between equal candidates are too infrequent on this trace to provide meaningful gains
- Second-chance reprieve bit — freezing position severs the recency signal that makes LRU ordering effective
- IRR as a replacement for frequency — underperforms the frequency baseline on this workload

Still exploring whether the combined policy... Frequency-window signals proved too narrow in cycle 1 and weren't worth revisiting; SIEVE's visited-bit approach underperformed against IRR in cycle 7; ghost admission works well with frequency-based designs but breaks down with IRR since it lacks the second timestamp needed to make the mechanism effective.

### 01:46:28 chat

Now I have the output directory. Let me write the file based on my analysis of all 10 cycles.

### 01:46:54 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/paper_outputs/lru_cache/lru_cache_claude-sonnet-4-6_2026-06-04_23-57-58-353/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Two-segment SLRU** (protected + probationary) has been the load-bearing structure since cycle 1; every winning design builds on it.\n- **Frequency as the primary eviction signal** outperforms pure-IRR in absolute hit-rate terms on this workload; the frequency baseline's ceiling is higher than the IRR baseline's ceiling.\n- **Counter halving on demotion** — preserving frequency history across segment boundaries rather than resetting — has been essential since cycle 3; reset counters make the signal evaporate.\n- **Scanning the LRU half of probationary** (not the quarter) is load-bearing: IRR and frequency variation are distributed throughout probationary, not compressed near the LRU head.\n- **Ghost admission** is a confirmed net-positive shortcut within frequency-primary designs; it acts at insertion time and is structurally orthogonal to frequency-based victim selection.\n- **Staleness as a tie-breaker** over equal-frequency candidates has been validated on two separate baselines (IRR-primary in cycle 9, frequency-primary in cycle 10); it is defined from the first access and handles the single-access entry class that IRR cannot rank.\n- The **80/20 protected/probationary split** has survived stress tests from both directions (cycles 2 and 8); it reflects the density of the active working set on this workload.\n\n## Recurring failures\n\n- **Ghost admission combined with IRR eviction**: ghost-admitted entries re-enter protected without a second timestamp, making their IRR score the maximum sentinel — indistinguishable from the coldest newcomer — and the two mechanisms undermine each other.\n- **Widening probationary beyond 20%**: triggers excess demotion churn; the hot working set is dense enough that a protected segment below 80% is genuinely too small (cycles 2 and 8 confirmed independently).\n- **Expanding the ghost window beyond probationary capacity**: reuse events at greater reuse distances are cold traffic on this trace, not deferred-warm arrivals; full-capacity ghost (cycle 4) and half-capacity ghost (cycle 6) both regressed.\n- **Fixed-rate frequency or IRR decay**: any single decay constant collapses frequency distinction when its half-life does not match the trace's inter-access distribution; the failure is a parameter-matching problem but no constant has been found to generalize.\n- **Narrowing the probationary scan to a quarter**: frequency and IRR values among demoted entries span the full second quartile; cutting the scan there loses meaningful selection pressure.\n- **IRR as a tie-breaker over equal-frequency candidates**: equal-candidate collisions under the dominant signal are rare enough on this trace that secondary tie-breakers activate negligibly (cycles 9 and 10).\n- **Second-chance reprieve bits that freeze node position**: severing continuous recency updates degrades the protected segment's ordering and offsets any reprieve benefit (cycle 3).\n- **IRR as the sole eviction signal replacing frequency entirely**: two cycles of IRR-baseline optimization peaked below the frequency baseline; the IRR signal conflates historical and current warmth disadvantageously on this workload.\n\n## Open questions\n\n- **Additive combination of frequency + staleness + ghost admission**: cycles 9 and 10 validated each of the latter two independently on the frequency-primary baseline but never combined them; this is the highest-priority untested hypothesis.\n- **Multi-workload transferability**: all gains have been validated on a single LIRS-generated trace; it is unknown whether the policy is broadly strong or specifically matched to this trace's reuse-distance structure.\n- **Adaptive segment-ratio balancing** (ARC-style feedback): proposed repeatedly but never implemented; the fixed 80/20 split is robust but may not be optimal across varying workload intensities.\n- **Staleness + ghost interaction**: ghost-admitted entries have a finite and growing staleness from their prior residency; it is unclear whether this makes staleness tie-breaking more or less effective in the ghost-admission population.\n- **Whether any IRR-compatible ghost mechanism exists**: special-casing ghost entries at eviction time (e.g., giving them a synthetic IRR based on ghost-residency gap) might rescue the combination, but it has not been attempted.\n\n## Per-policy notes\n\n- **Plain LRU (baseline)**: scan-pollution vulnerable; any segmentation or reuse-aware mechanism improves on it; useful only as a floor.\n- **CLOCK and frequency-window eviction (cycle 1)**: eviction-time-only signals; weaker than insertion-time segmentation on scan-heavy workloads; not revisited after cycle 1.\n- **SIEVE visited-bit (cycle 7)**: suboptimal relative to IRR on this trace; not revisited.\n- **SLRU + frequency (cycles 1–6, 9–10)**: the dominant policy line; counter halving, LRU-quarter scan in protected, LRU-half scan in probationary, and ghost admission all proven additive within this family; staleness tie-breaker validated as a further improvement; currently the best-performing design.\n- **SLRU + IRR (cycles 7–8)**: strong relative gains from its own starting point but absolute ceiling below the frequency baseline; ghost admission is incompatible due to missing second-timestamp problem; scan fraction and segment-ratio conclusions mirror the frequency baseline.\n- **Ghost admission**: net-positive within frequency-primary designs (cycles 2 and 10), neutral to harmful in IRR designs (cycles 3–6 on frequency, cycles 8–10 on IRR); the mechanism is design-context-dependent, not universally positive or negative.\n"
}
```

### 01:47:02 chat

Written. Here's a summary of what went into each section:

**Recurring wins** — Seven patterns that held across multiple cycles: SLRU segmentation as the persistent foundation, frequency-primary eviction beating IRR in absolute terms, counter halving on demotion, the mandatory LRU-half probationary scan, ghost admission's additive role inside frequency designs, staleness as a validated tie-breaker on two baselines, and the robust 80/20 split.

**Recurring failures** — Eight approaches that consistently didn't pay off: ghost+IRR interaction (structurally harmful due to the missing-timestamp sentinel problem), probationary widening, ghost window expansion beyond probationary capacity, fixed-rate decay, narrow quarter-scans in probationary, secondary tie-breakers over equal-signal candidates, position-freezing reprieve bits, and IRR as a full frequency replacement.

**Open questions** — Five threads worth probing: combining frequency + staleness + ghost (the cycle-10 natural completion), multi-workload evaluation (overdue), adaptive segment-ratio balancing (proposed but never implemented), staleness/ghost interaction dynamics, and whether an IRR-compatible ghost mechanism is even possible.

**Per-policy notes** — Organized by policy family: plain LRU, CLOCK/frequency-window, SIEVE, SLRU+frequency (dominant line), SLRU+IRR, and ghost admission as a cross-cutting mechanism with context-dependent effectiveness.


---

- finished: 2026-06-06T01:47:03+00:00
- duration: 67.26s
- status: ok
- input_tokens: 7
- output_tokens: 3038
- cache_read_tokens: 118025
- cache_write_tokens: 12885
- cost_usd: 0.137172
