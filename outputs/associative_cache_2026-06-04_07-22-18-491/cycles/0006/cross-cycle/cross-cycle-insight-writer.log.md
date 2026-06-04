# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-06-04T05:38:36+00:00

## timeline

### 05:38:40 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me analyze the six cycles and synthesize patterns across them.

First, I need to find the output directory. Let me look for it.

### 05:38:40 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 05:38:40 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 05:38:42 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/"
}
```

### 05:38:44 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve"
}
```

### 05:38:47 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/cross-cycle"
}
```

### 05:39:04 thinking

The output directory is `/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/cross-cycle`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize the six cycles of data into the four sections:

**Recurring wins:**
1. Complementary mechanisms (admission + eviction) — the biggest wins came from combining probe admission with stash recovery
2. Structurally minimal changes that activate latent capacity — v001 cycle 1 used the already-present `lfu_counter` field at zero cost


3. Composition hypotheses validated across cycles — combining mechanisms proved additive when targeting different failure modes
4. Probationary admission emerged as the strongest single lever

**Recurring failures:**
1. Eviction-scoring refinements on stable workloads never triggered on Zipfian hot sets
2. Promotion credit above +2 overshoots survival thresholds
3. Ghost and extended eviction memory inverts the LFU signal
4. Stash intake filters rejecting zero-counter items removes recently inserted items that haven't been re-accessed yet, not genuinely cold ones
5. Larger stash caps accumulate noise instead of survivors
6. Applying LFU within the stash fails because the frequency gradient is too compressed to discriminate effectively

**Open questions:**
1. The metadata-size evaluation gap remains unresolved across six cycles — confirmed as an infrastructure issue
2. Whether these validated mechanisms generalize beyond stable Zipfian workloads
3. Whether probe zone sizing above half-capacity is additive or subject to diminishing returns
4. What structural possibilities exist beyond the probe-stash-LFU triangle

**Per-policy notes:**
- ClassicLRU serves as baseline but can't distinguish hot from cold traffic under burst conditions
- LFU eviction is neutral alone after probe admission and only gains value combined with stash recovery
- Probe-zone admission remains the strongest single mechanism found at half-capacity minimum
- FIFO stash maintains correct ordering while LFU within the stash becomes redundant
- Promotion credit of +2 hits the local optimum; +1 loses to re-eviction while +3 overshoots

Now I'm ready to write the file.

### 05:39:04 chat

Now I have the output directory. Let me synthesize the six-cycle ledger into the four fixed sections.

### 05:39:32 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Composing complementary mechanisms**: every winning composition targeted non-overlapping failure modes. LFU eviction selects a better victim; the FIFO stash recovers displaced warm items; probe admission blocks cold items before they compete for primary slots. Each layer added value only because the others handled what it could not.\n- **Structurally minimal changes that activate latent capacity**: the first win (LFU counter) piggybacked on a field already carried at zero cost. Later wins built on the same principle — changes that reuse existing structure consistently outperform changes that add new state.\n- **Composition hypotheses validated by the ledger itself**: both Cycle 2 and Cycle 6 confirmed compositions explicitly recommended by the prior cycle. Following the ledger's own open questions has a high hit rate.\n- **Probationary admission as the strongest single lever**: moving the filter to the admission boundary (before items enter primary) outperformed all eviction-side refinements tested across five preceding cycles, and did so from a simpler baseline.\n- **Wins cluster at structural boundaries**: the highest-gain changes operated at transitions — primary-to-stash (promotion credit), outside-to-primary (probe admission) — rather than inside a single subsystem.\n\n## Recurring failures\n\n- **Eviction-scoring refinements on a stable Zipfian hot set**: counter caps and frequency-times-recency product scores were never triggered by workload-a, whose hot-set membership is stable. Sound logic measured as neutral because the failure modes it targets do not occur on this trace.\n- **Promotion credit above +2**: flat +3 and frequency-proportional scaling both overshoot the one-burst-survival threshold. Counter-derived heuristics at the promotion boundary are unreliable because past LFU values reflect burst-displacement timing, not future reuse probability.\n- **Stash intake filters**: rejecting zero-counter evictees removes recently inserted items displaced before their confirming access, not cold waste. Stash value depends on holding all recent evictions, not a quality-filtered subset.\n- **Larger stash caps**: doubling the quarter-capacity default let cold items linger and earn spurious promotions, displacing medium-frequency primary residents. The FIFO cap is a self-cleaning mechanism, not a constraint.\n- **LFU ordering inside the stash**: every stash resident already arrived as the weakest LFU item in primary; the frequency gradient within the stash is too compressed to discriminate. FIFO's age-based ordering is better matched to the stash's role.\n- **Ghost / extended eviction memory**: giving re-inserted items a frequency head start based on ghost-tier history inverts the LFU signal on this workload and costs hit rate.\n\n## Open questions\n\n- **`metadata-size` evaluator gap (six cycles unresolved)**: confirmed in Cycle 5 as a harness infrastructure problem — the `evolve_metadata_size` binary is not wired into the evaluation pipeline. Probe, stash, and LFU counter fields add per-bucket memory overhead that remains entirely invisible to the fitness function. Any memory-efficiency hypothesis is unverifiable until this is fixed.\n- **Generalization beyond stable Zipfian workloads**: counter-cap, product-score, and ghost-tier hypotheses may be sound but untriggered. A trace with hot-set drift or bursty insertion pressure could surface gains that workload-a systematically suppresses.\n- **Probe zone sizing above half-capacity**: half-capacity is at or near the minimum viable window for this workload's inter-access gap. Whether the gain is monotone above that point or peaks near half-capacity has not been tested.\n- **Structural axes outside the probe-stash-LFU triangle**: six cycles have converged on one mechanism family. Key-space partitioning, time-windowed frequency, and workload-shift detection are unexplored directions that may matter more on non-stationary workloads.\n- **Dynamic promotion credit**: a multiplier that scales with how many times an item has cycled through the stash was raised but never tested; it may outperform the fixed +2 under sustained eviction pressure.\n\n## Per-policy notes\n\n- **ClassicLRU (baseline)**: cannot distinguish hot from cold traffic under cold-burst conditions; all subsequent gains trace to mechanisms that add discrimination the baseline lacks.\n- **LFU-weighted eviction**: neutral in isolation once probe admission is in place — the frequency distribution within primary is too flat after probe filtering. Only valuable in combination with the stash, where it selects a better victim for the stash to hold. The `lfu_counter` field was already present at zero cost; the win came from activating it, not adding it.\n- **FIFO stash at quarter-capacity**: correct internal ordering — LFU within the stash is redundant and adds no discrimination. The cap is calibrated; larger caps accumulate noise. The stash's role is bounded recovery of warm items at the primary boundary, not a secondary cache in its own right.\n- **Promotion credit +2**: confirmed local optimum across multiple probes. +1 loses promoted items to immediate re-eviction on the next cold burst; +3 and higher overshoot. The optimal value is workload-specific and tied to the typical burst length.\n- **Probe-zone admission at half-capacity**: strongest mechanism found. Half-capacity is the minimum window that allows a confirming re-access to arrive before the probe FIFO ejects the item. Quarter-capacity regressed; above half has not yet been tested.\n- **Full composition (probe + stash + LFU eviction + promotion credit)**: the mechanism set as of Cycle 6 baseline. Each layer addresses a different failure mode; removing any single element degrades the outcome. The design appears locally complete on this workload.\n"
}
```

### 05:39:41 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what was synthesized across the six cycles:

**Recurring wins** — The clearest pattern is that every significant gain came from mechanisms that operate at structural *boundaries* (admission, eviction, promotion transitions) and that address non-overlapping failure modes. Composition hypotheses explicitly recommended by the prior cycle's open questions had a consistently high hit rate.

**Recurring failures** — Eviction-scoring refinements (counter caps, product scores) never fired on workload-a's stable Zipfian hot set. Promotion credit above +2 consistently overshoots. Stash intake filters and larger stash caps both accumulate noise rather than value. LFU ordering inside the stash is redundant — the frequency gradient is too compressed.

**Open questions** — The `metadata-size` evaluator gap (confirmed as infrastructure, not variant-level) is now six cycles old and blocks all memory-efficiency hypotheses. Probe sizing above half-capacity and generalization to non-stationary workloads are the next productive unexplored axes.

**Per-policy notes** — Traces each mechanism's validated role: ClassicLRU as the undifferentiated baseline, LFU eviction as only valuable in combination with the stash, FIFO stash as self-calibrating at quarter-capacity, promotion credit +2 as the confirmed local optimum, and probe admission at half-capacity as the strongest single lever found.


---

- finished: 2026-06-04T05:39:41+00:00
- duration: 64.91s
- status: ok
- input_tokens: 7
- output_tokens: 2819
- cache_read_tokens: 100065
- cache_write_tokens: 9018
- cost_usd: 0.110423
