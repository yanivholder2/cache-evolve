# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-06-04T05:05:55+00:00

## timeline

### 05:05:59 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first find the output directory, then write the file.

Let me look for the output directory structure.

### 05:06:00 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 05:06:00 tool: Bash

```json
{
  "command": "ls /home/yaniv/projects/cache-evolve/",
  "description": "List project root to find output dir"
}
```

### 05:06:02 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/**/*"
}
```

### 05:06:05 thinking

I need to find the output directory where `cross-cycle-insights.md` should be written. Let me look at the CLAUDE.md or look for specific output directories.

### 05:06:05 tool: Read

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/CLAUDE.md"
}
```

### 05:06:08 thinking

Now I know the output directory. I need to write `cross-cycle-insights.md` to:
`/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/cross-cycle/`

Let me check what's in that directory first.

### 05:06:09 tool: Glob

```json
{
  "pattern": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/cross-cycle/*"
}
```

### 05:06:30 thinking

Good. The output directory is `/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/cross-cycle/`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize the insights across all 4 cycles:

**Recurring wins:**
1. Frequency-aware eviction (LFU) - won in cycle 1, and the composition with stash won in cycle 2. Activating latent capacity (lfu_counter already existed) made it structurally minimal with immediate payoff.


2. Pairing LFU eviction with the second-chance stash mechanism creates mutual reinforcement between the two approaches.
3. Intervening at structural boundaries like the promotion interface rather than tweaking internals proved effective—cycle 3's win came from adjusting promotion-credit thresholds to smooth the transition back into primary cache.

**Recurring failures:**
1. Refining eviction scoring through counter caps and product scores stayed neutral in cycle 2 because workload-a's stable hot set never triggers these conditions.
2. Expanding stash capacity backfired in cycle 3 by allowing cold items to persist long enough to earn undeserved promotions.
3. Applying LFU logic inside the stash...

The frequency distribution within the stash is too compressed to meaningfully separate items. Using past counter values as promotion signals fails on Zipfian workloads because those counters capture burst-displacement timing rather than actual reuse likelihood. Filtering zero-counter items at intake also regressed—those items are recently-inserted entries displaced before they could be reused, not genuinely cold waste.

**Open questions:**
1. I still haven't quantified the per-bucket Vec overhead across four cycles of stash variants.
2. Whether these failed approaches would behave differently on a workload where the hot set membership shifts over time.
3. Whether there are structural improvements entirely outside the stash-and-promotion subsystem that haven't been explored yet.
4. The promotion-credit axis feels exhausted—what structural lever comes next?

**Per-policy notes:**
Looking at the progression: ClassicLRU had slack in both frequency and recency dimensions. LFU eviction was the first real win, activating the latent counter field with minimal structural change. Adding a second-chance FIFO stash with LFU pre-filtering established a new baseline. The current best is LFU plus stash plus a +2 promotion credit—a small, clean win at the stash-to-primary boundary. Pushing promotion credit to +3 or using frequency-proportional scaling both regressed, confirming the local optimum sits at +2. Filtering stash intake by counter value also regressed, so all recent evictions should feed into the stash regardless of their frequency count.

### 05:06:49 tool: Write

```json
{
  "file_path": "/home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Activating latent capacity in the existing data model (LFU counter already carried at zero cost) produced the first win with minimal structural change and no regressions.\n- Composing two complementary mechanisms — LFU eviction and a second-chance stash — yielded mutually reinforcing gains: LFU pre-filters the stash toward items with real reuse history, so stash promotions are productive far more often than a naive second-chance mechanism alone.\n- Wins consistently come from operating at structural boundaries (the eviction decision, the stash-to-primary promotion transition) rather than tuning internals of a mechanism already working well.\n- Changes that are structurally minimal and additive (rather than replacement-based) have a clean win-or-neutral profile with no regressions.\n\n## Recurring failures\n\n- Eviction-scoring refinements (frequency counter caps, frequency-times-recency product scores) repeatedly measure as neutral on workload-a because the hot set is stable — the failure modes these target (counter inflation, insertion-pressure edge cases) do not occur on this trace.\n- Enlarging stash capacity regressed hit rate: a larger cap lets cold items accumulate long enough to earn spurious promotions that displace medium-frequency primary residents.\n- Applying LFU selection inside the stash regressed hit rate: every item in the stash is already the lowest-frequency item from primary, so the frequency gradient within the stash is too compressed for LFU to discriminate; FIFO age-ordering is better matched to the stash's role.\n- Counter-derived heuristics at the promotion boundary (frequency-proportional credit, intake filtering by counter value) regressed hit rate: on Zipfian workloads, an item's LFU counter at eviction time reflects burst-displacement timing far more than genuine future reuse probability.\n- Filtering stash intake to exclude zero-counter items regressed hit rate: zero-counter items at eviction are recently inserted items displaced before the workload could access them again, not cold waste.\n\n## Open questions\n\n- The `metadata-size` dimension has been absent from fitness vectors for four consecutive cycles. The stash adds per-bucket `Vec` fields whose memory overhead has never been quantified; variants targeting memory efficiency remain unverifiable until this is resolved.\n- Would counter-cap, product-score, and other neutral-on-workload-a hypotheses show meaningful gains on a more adversarial trace with hot-set drift or bursty insertion patterns?\n- With the stash-and-promotion subsystem now well-calibrated, what is the next structural axis outside this subsystem that has gone unexplored?\n- Is there a workload-b or synthetic adversarial trace that would expose failure modes invisible on workload-a's stable Zipfian hot set?\n\n## Per-policy notes\n\n- **ClassicLRU (baseline):** Starting point; had slack in both frequency and recency dimensions simultaneously, making it susceptible to cold-burst displacement of hot items.\n- **LFU eviction only (cycle 1 winner):** Structurally minimal; no regressions; reduced cold-burst displacement immediately.\n- **LFU eviction + second-chance stash (cycle 2 winner, established baseline):** FIFO stash at bucket-quarter capacity; mutually reinforcing composition; this pairing is now the stable foundation across all subsequent cycles.\n- **LFU + stash + promotion credit +2 (cycle 3 winner, current best):** Small but clean win; +2 clears the tie that caused promoted items to be immediately re-evicted on the next cold burst; operates at the stash-to-primary interface rather than inside either mechanism.\n- **Promotion credit +3 and frequency-proportional scaling (cycle 4 regressions):** Confirm +2 is the local optimum; the promotion-credit axis is exhausted.\n- **Stash intake filter (cycle 4 regression):** Confirms all recent evictions should enter the stash unconditionally; counter-value-based filtering at intake is counterproductive.\n"
}
```

### 05:06:54 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what was synthesized across the four cycles:

**Recurring wins:** Activating latent data-model capacity wins cheaply; composing LFU eviction with a second-chance stash is mutually reinforcing; improvements consistently land at structural boundaries (eviction decision, stash-to-primary transition) rather than inside mechanisms.

**Recurring failures:** Eviction-scoring refinements (counter caps, product scores) are neutral on workload-a's stable Zipfian hot set; enlarging stash capacity and applying LFU inside the stash both regress; counter-derived heuristics at the promotion boundary are unreliable because past counter values reflect burst-displacement timing, not future reuse probability.

**Open questions:** The `metadata-size` measurement gap is now four cycles old and unresolved; neutral hypotheses may behave differently on a workload with hot-set churn; the stash-and-promotion subsystem is fully characterized, leaving the next structural axis unexplored.

**Per-policy notes:** Tracks each policy generation from ClassicLRU through to the current best (LFU + stash + promotion credit +2), with the promotion-credit axis now confirmed exhausted.


---

- finished: 2026-06-04T05:06:55+00:00
- duration: 60.11s
- status: ok
- input_tokens: 7
- output_tokens: 2677
- cache_read_tokens: 104163
- cache_write_tokens: 11126
- cost_usd: 0.116021
