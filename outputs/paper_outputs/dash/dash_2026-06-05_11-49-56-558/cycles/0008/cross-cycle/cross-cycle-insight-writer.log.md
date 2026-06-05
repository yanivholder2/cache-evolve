# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-06-05T14:56:04+00:00

## timeline

### 14:56:07 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first find the output directory, then write the file.

Let me look for the output directory structure.

### 14:56:07 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 14:56:07 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 14:56:09 tool: Bash

```json
{
  "command": "ls -la",
  "description": "List current working directory"
}
```

### 14:56:12 tool: Read

```json
{
  "file_path": "/root/task.md",
  "limit": 30
}
```

### 14:56:14 tool: Bash

```json
{
  "command": "pwd",
  "description": "Show current working directory"
}
```

### 14:56:40 thinking

Now I know the output directory. I need to write `cross-cycle-insights.md` in this directory: `/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/cross-cycle/cross-cycle-insights.md`

Let me now synthesize the insights from all 8 cycles and write the file.

**Recurring wins:**
1. Hot-tier expansion (normal/stash topology) — most dominant lever; each increase from 50/50 → 75/25 → 7/8 produced gains, though with diminishing returns


2. MRU insertion position proved universally beneficial across all cycles, while withholding MRU status consistently hurt performance
3. The zero-access frequency guard emerged as a confirmed winner in later cycles, effectively protecting items with at least one in-tier hit from being evicted by unproven newcomers
4. Second-chance stash recovery stacked cleanly on top of the frequency guard, providing additive gains without interference
5. Multiple compound tests showed that these orthogonal improvements combine without negative interactions

**Recurring failures:**
Frequency and LFU-based eviction strategies repeatedly underperformed simple recency on this workload despite its Zipfian distribution, suggesting the access patterns are fundamentally recency-dominant.

Ghost sets added only noise across different configurations, indicating the stash boundary isn't where meaningful cache hits are actually lost. Attempts to use LRU or front insertion for probationary items backfired, as did aggressive aging policies that evicted mildly-warm items or penalized items with multiple hits. Weakening stash traversal allowed cold items to displace warm residents, confirming the stash functions as a critical admission filter. Placing second-chance items at MRU immediately displaced earned residents, so the LRU end is definitively the correct position.

**Open questions:**
Beyond the 7/8 split I've tested, there might be an inflection point where the stash becomes too shallow to effectively buffer burst traffic.

I'm also wondering whether v001's geometry could compound with stricter promotion gates for additional gains, or if soft-capping the frequency counter to prevent staleness would help once the geometry changes. Different workloads with scan traffic or bursty cold arrivals might unlock frequency-based or admission-control benefits that workload-a doesn't show. Capacity stress testing with a smaller total cache could also make probationary schemes more competitive.

From the policy testing so far, ClassicLRU with MRU insertion dominates workload-a because recency is a stronger signal than any frequency counter variant I've tried. LFU and frequency-based eviction are structurally incompatible with this workload—even with thresholds, soft decay, or single-access gates, they consistently underperform. Frequency guards that filter zero-access items show promise though.

Second-chance recovery works well when paired with frequency guards, especially when inserting at the LRU end rather than MRU, and it outperforms ghost sets because it's unconditional. Ghost sets themselves add only noise. Timestamp-based promotion creates destructive interactions with frequency guards due to creation-time contamination letting cold items outrank hot ones. Segmented LRU with asymmetric promotions and demotions actually hurts performance—displacing warm items costs more than the selectivity gains. Admission control approaches like stash-bypass and LFU-gated promotion both regressed, suggesting the stash itself is doing critical quality-gate work.

### 14:56:40 chat

Now I have enough context. Let me write the file synthesizing all 8 cycles.

### 14:57:05 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Hot-tier topology expansion** has been the single highest-leverage lever across cycles 1–3 and 8. Each increase in the normal/stash split (equal halves → 75/25 → 7/8) produced positive gains; the working set consistently fits more comfortably as hot-tier capacity grows. Returns are now clearly diminishing but not yet zero.\n- **MRU insertion position** is universally beneficial on workload-a. Every cycle that experimented with non-MRU placement for newly arrived, re-admitted, or re-promoted items regressed. Recency position encodes a stronger future-access signal than any alternative tested.\n- **Eviction selectivity via zero-access frequency guard** (protecting items with ≥1 in-tier hit from eviction by never-accessed newcomers) produced the largest single-cycle gain in cycles 6–8. The zero-access cutoff is empirically correct for this workload; raising it hurts.\n- **Unconditional second-chance stash recovery** (re-inserting stash-evicted items at the LRU end of their target bucket) stacks cleanly with the frequency guard because the two mechanisms target distinct pipeline stages — probing eviction selectivity vs. stash overflow recovery.\n- **Stacking orthogonal mechanisms** consistently pays off. Topology + MRU re-entry, frequency guard + second-chance recovery, and topology + promotion gating all compounded without destructive interference when the mechanisms addressed different loss sites.\n\n## Recurring failures\n\n- **LFU / frequency-based eviction in any form** has never beaten recency on workload-a. Per-hit co-resident aging, threshold-gated decay, and soft-decay variants all regressed or tied. The access distribution punishes frequency discrimination; workload-a is recency-dominant.\n- **Ghost set re-admission** has been noise-level across all position and size configurations tested in cycles 4–5. The stash boundary is not a meaningful hit-loss site on this workload; conditional event-driven re-admission provides no net benefit.\n- **Withholding MRU for probationary filtering** (LRU entry for newly inserted, re-admitted, or demoted items) consistently hurt. Items displaced from prime position are re-accessed before eviction on workload-a, so deferral delays useful promotion without filtering cold noise.\n- **Weakening or bypassing stash traversal** (front-door cold bypass, direct-insert modes) always regressed. The stash functions as a load-bearing admission filter; removing it allows cold items to displace warm residents at a net cost.\n- **MRU-end insertion for second-chance items** regressed sharply; it grants unproven items immediate promotion candidacy and displaces earned residents. LRU-end second-chance insertion is definitively correct and should not be revisited.\n- **Timestamp-based promotion** interacted destructively with the frequency-guard baseline because creation-time contamination allows cold-but-recently-inserted items to outcompete hot-but-older ones. The net interaction is harmful, not independent.\n\n## Open questions\n\n- Is there an inflection point in the normal/stash split (beyond 7/8) where stash becomes too shallow to buffer burst arrivals and the marginal loss from reduced stash headroom overtakes the geometry gain?\n- Would soft-capping `lfu_counter` to prevent counter staleness yield any improvement once geometry ceases to help? This is the clearest untested policy axis remaining.\n- Could geometry (7/8 split) and the stricter two-hit promotion gate stack for further gain, given they target distinct sites (capacity vs. promotion selectivity)?\n- Would timestamp-based promotion recover if creation-time contamination were eliminated — e.g., by only recording a timestamp on the first *hit* rather than on insertion?\n- Do frequency-based or admission-control schemes unlock real gains on a workload with heavy scan traffic or bursty cold-item arrivals, where these mechanisms are structurally better matched?\n\n## Per-policy notes\n\n- **ClassicLRU / recency (MRU insertion):** The dominant policy class for workload-a without exception. Recency signal is reliable across the entire access distribution; every departure from it has cost hits.\n- **LFU / frequency eviction:** Structurally incompatible with workload-a in all forms tested — per-hit aging, threshold-gated decay, and access-count-gated variants all lost. Should remain retired unless a different workload trace motivates revisiting.\n- **Zero-access frequency guard:** High-value selective eviction mechanism; confirmed correct at the zero-access threshold. Single-access protection is the right boundary — raising the cutoff over-shrinks the protected set. Counter resets on stash demotion have not been confirmed harmful.\n- **Second-chance stash recovery:** Additive with the frequency guard; unconditional is better than ghost-set conditional; LRU-end insertion is mandatory — MRU-end insertion regressed clearly and is definitively contraindicated.\n- **Ghost set:** Retired. Noise-level across multiple configurations in cycles 4–5. The conditional re-admission mechanism cannot compensate for the stash boundary not being a meaningful loss site.\n- **Timestamp-based LRU promotion:** Shows some standalone benefit under Zipfian distribution but is not compatible with the frequency-guard baseline due to creation-time contamination. Not a drop-in supplement; would need a hit-time-only timestamp variant to be viable.\n- **Segmented-LRU asymmetry (promotions at MRU, demotions at LRU):** Contraindicated on workload-a; demoted items retain residual heat and losing MRU position in the destination tier accelerates churn rather than improving selectivity.\n- **Stash-admission gating (LFU-gated promotion, bypass policies):** Both forms regressed. The stash's role as an admission filter is load-bearing; filtering at the stash boundary removes a productive quality gate.\n"
}
```

### 14:57:13 chat

Written `cross-cycle-insights.md` to the output directory. Here's what the file captures across all eight cycles:

**Recurring wins** — Hot-tier topology expansion has been the dominant lever since cycle 1, with consistent positive returns at each split increase (50/50 → 75/25 → 7/8), though with clearly diminishing returns. MRU insertion is universally beneficial; the zero-access frequency guard delivered the biggest single-cycle gain (cycles 6–8); second-chance stash recovery stacks cleanly with it; and orthogonal mechanisms have compounded reliably throughout.

**Recurring failures** — LFU/frequency-based eviction in every form has failed on workload-a across all eight cycles. The ghost set was retired after proving noise-level. Withholding MRU for probationary filtering consistently hurts. Stash bypass or weakening always regressed. MRU-end second-chance insertion is definitively contraindicated.

**Open questions** — Whether there's a stash-shallowness breakpoint beyond 7/8; whether soft-capping `lfu_counter` helps; whether the geometry + stricter-gate compound pays off; whether timestamp promotion is salvageable with hit-time-only timestamps; and whether different workloads unlock frequency-based gains.

**Per-policy notes** — Grouped observations for ClassicLRU/recency, LFU, the frequency guard, second-chance recovery, the ghost set, timestamp promotion, segmented-LRU asymmetry, and stash-admission gating.


---

- finished: 2026-06-05T14:57:13+00:00
- duration: 69.56s
- status: ok
- input_tokens: 7
- output_tokens: 3196
- cache_read_tokens: 104775
- cache_write_tokens: 9992
- cost_usd: 0.12265
