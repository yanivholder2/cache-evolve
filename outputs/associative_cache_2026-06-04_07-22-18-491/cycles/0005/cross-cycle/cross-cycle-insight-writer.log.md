# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-06-04T05:25:06+00:00

## timeline

### 05:25:10 thinking

Let me analyze the ledger digest across all 5 cycles and write a comprehensive `cross-cycle-insights.md` file. First, I need to find the output directory.

### 05:25:10 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 05:25:10 tool: Glob

```json
{
  "pattern": "**/output*"
}
```

### 05:25:12 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/",
  "description": "List project root directory"
}
```

### 05:25:15 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve"
}
```

### 05:25:17 tool: Glob

```json
{
  "pattern": "**/*",
  "path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/cross-cycle"
}
```

### 05:25:39 thinking

The output directory is `/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/cross-cycle`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize the 5 cycles of data:

**Recurring wins:**
1. Frequency-aware eviction (LFU) — latent capacity in the data model, won in Cycle 1
2. Second-chance stash — won in Cycle 1 (as v003), then composition with LFU won in Cycle 2


3. Promotion credit +2 — won in Cycle 3, confirmed as local optimum in Cycle 4
4. Probationary admission (probe zone) — won in Cycle 5

Patterns across cycles:
- Structural changes that exploit already-present data fields (LFU counter) win with minimal overhead
- Changes targeting the admission boundary (what enters primary) have been consistently productive
- FIFO ordering within the stash outperforms frequency-based ordering inside the stash
- Composition of complementary mechanisms (LFU + stash) reinforces rather than saturates

**Recurring failures:**
- Eviction-scoring refinements like counter caps and frequency-times calculations tend to underperform, suggesting the simpler mechanisms are already well-tuned
- Ghost memory and frequency head starts for evicted keys actually hurt hit rate by corrupting the LFU signal
- Expanding stash capacity backfires by letting cold items accumulate noise instead of filtering for useful survivors
- Frequency gradients inside the stash are too weak to be useful; FIFO ordering works better
- Filtering out zero-counter items removes legitimate candidates since they're often just-inserted rather than cold
- Promotion credits above +2 overshoot the survival threshold; both fixed and scaled approaches regress

There's still a measurement gap around metadata size across several stash and memory variants that I haven't quantified yet.

I'm also trying to understand whether probationary admission and stash-plus-LFU mechanisms target different failure modes and complement each other, how probe zone sizing affects the gain curve, whether these approaches generalize beyond stable hot-set workloads, and if dynamic promotion credits with repeat multipliers could beat the fixed +2 approach.

ClassicLRU can't separate cold-insert churn from hot traffic and gets vulnerable to burst displacement, while LFU eviction with per-item counters became the new baseline after proving it reduces cold-item displacement.

The LFU plus second-chance stash combination reinforces both mechanisms with a small FIFO cap and tuned promotion credit, but probationary admission emerged as the strongest single improvement by filtering one-time-access items upstream before they consume primary cache slots.

### 05:25:39 chat

Now I have the output directory. Let me write the insights file based on all 5 cycles.

### 05:26:07 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Admission-boundary interventions outperform eviction-side tuning.** The largest single-cycle gains have come from changing what enters primary (probationary probe zone, Cycle 5) rather than how primary evicts. Admission control discards one-time-access items before they ever displace residents.\n- **Exploiting latent data-model capacity wins immediately.** LFU eviction (Cycle 1) succeeded because the frequency counter was already carried on every item at zero marginal cost; structural changes with no new overhead convert into hit-rate gains without regressions.\n- **Composing complementary mechanisms reinforces rather than saturates.** LFU eviction reduced which cold items reached the stash; the stash recovered items that slipped through primary anyway. Composition (Cycle 2) produced additive gain, not diminishing returns.\n- **FIFO ordering inside the stash beats frequency-based ordering.** Every item in the stash arrived because LFU already identified it as lowest-frequency in primary; the within-stash frequency gradient is too compressed to discriminate. FIFO's age ordering is better matched to the stash's purpose.\n- **Promotion credit fixes round-trips at the stash-to-primary interface.** Raising the credit from +1 to +2 (Cycle 3) cleared the tie that caused freshly promoted items to be re-evicted immediately on the next cold burst, converting churn into genuine secondary hits.\n\n## Recurring failures\n\n- **Eviction-scoring refinements do not register on a stable hot-set workload.** Counter caps and frequency-times-recency product scores (Cycle 2) both measured neutral because workload-a's hot set is stable enough that counter inflation and insertion-pressure edge cases never occur on this trace. The logic is sound but the trigger conditions are absent.\n- **Extending eviction memory further back into history adds noise.** The ghost cache (Cycle 5) gave re-inserted cold items a frequency head start that inverted the LFU signal and cost hit rate. Items that aged through primary and stash without re-access are reliably cold, not unlucky hot items.\n- **Stash capacity expansion accumulates noise rather than useful survivors.** Doubling the stash cap (Cycle 3) let cold items linger long enough to earn spurious promotions that displaced medium-frequency primary residents. The small FIFO cap is a self-cleaning property, not a constraint.\n- **Stash intake filtering by eviction-time counter misreads the workload.** Zero-counter items at eviction time are frequently just-inserted items displaced before the workload could access them again, not inherently cold waste. Rejecting them removes legitimate stash candidates.\n- **Promotion credit above +2 overshoots.** Flat +3 and frequency-proportional scaling up to +4 (Cycle 4) both regressed; the threshold is the one-burst-survival boundary, not a gradient. Counter-derived heuristics at the promotion boundary are unreliable on Zipfian traces because past counter value reflects burst-displacement timing more than genuine reuse probability.\n\n## Open questions\n\n- **`metadata-size` measurement gap is a confirmed harness infrastructure problem.** Five cycles of stash and memory-efficiency variants have run with no quantification of per-bucket memory overhead. The `evolve_metadata_size` binary is not wired into the evaluation pipeline; variants targeting memory trade-offs remain unverifiable until this is fixed at the harness level.\n- **Do probationary admission and the stash-plus-LFU chain compose?** The two mechanisms target different failure modes — spurious admission versus premature eviction of re-accessed items — and may reinforce each other the same way LFU and the stash did in Cycle 2.\n- **What is the optimal probe zone size?** The half-capacity probe cap won in Cycle 5, but the gain curve above and below that ratio has not been characterised.\n- **Would workload-sensitive hypotheses show gains on non-stable traces?** Counter caps, product-score eviction, ghost caches, and stash intake filters all have coherent failure modes as their motivation; they have only been tested on a workload where those failure modes do not occur.\n- **Does dynamic promotion credit outperform fixed +2?** A repeat-promotion multiplier (scaling credit by how many times an item has cycled through the stash) remains untested.\n\n## Per-policy notes\n\n- **ClassicLRU (original baseline):** Cannot distinguish cold-insert churn from hot traffic; hot items are permanently displaced by cold-item bursts. Served as the branching point for Cycle 5's admission-control variant, which won directly from this baseline.\n- **LFU eviction (Cycle 1 winner, new baseline from Cycle 2):** Activates the latent frequency counter; reduces cold-item displacement with no new structural overhead. Became the foundation for two subsequent cycle winners.\n- **LFU eviction + FIFO second-chance stash + +2 promotion credit (Cycle 2–3 accumulated baseline):** The established compound baseline after Cycle 3. Stash cap at quarter bucket capacity; promotion credit fixed at +2. All further stash-internal and promotion-credit tuning regressed in Cycle 4; this configuration is the stash mechanism's local optimum on this workload.\n- **Probationary admission / probe zone (Cycle 5 winner):** Largest gain of any single cycle, and it branched from ClassicLRU rather than the accumulated stash chain. Silently discards single-access items in a probe FIFO before they compete for primary slots. Composition with the stash chain is the highest-priority unresolved hypothesis.\n"
}
```

### 05:26:16 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what the five-cycle synthesis produced:

**Recurring wins** — Four patterns held up repeatedly: admission-boundary interventions (largest gains), exploiting latent data-model capacity (LFU counter), composing complementary mechanisms (LFU + stash), FIFO ordering inside the stash, and the +2 promotion credit fixing stash-to-primary round-trips.

**Recurring failures** — Five approaches consistently failed: eviction-scoring refinements (counter caps, product scores) never trigger on the stable hot-set workload; ghost/extended eviction memory inverts the LFU signal; stash capacity expansion accumulates noise; stash intake filtering misidentifies just-inserted items as cold; promotion credit above +2 overshoots.

**Open questions** — The `metadata-size` evaluator gap is now a *confirmed* infrastructure problem (not variant-level), not merely suspected. The composition of probationary admission with the stash chain is the top unresolved hypothesis. Probe zone sizing and workload-sensitivity of previously neutral hypotheses also remain open.

**Per-policy notes** — Tracks ClassicLRU, LFU eviction, the accumulated LFU+stash+credit compound baseline, and the new probationary admission policy, with their respective roles and current status.


---

- finished: 2026-06-04T05:26:17+00:00
- duration: 70.54s
- status: ok
- input_tokens: 7
- output_tokens: 3039
- cache_read_tokens: 96577
- cache_write_tokens: 8564
- cost_usd: 0.110258
