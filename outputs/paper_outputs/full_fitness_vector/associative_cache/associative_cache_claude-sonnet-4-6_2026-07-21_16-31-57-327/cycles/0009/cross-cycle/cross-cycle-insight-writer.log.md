# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\cross-cycle
- allow_edits: True
- started: 2026-07-21T19:54:27+00:00

## timeline

### 19:54:31 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. Let me first find the output directory, then synthesize insights across all 9 cycles.

Let me look for the output directory structure.

### 19:54:32 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 19:54:32 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 19:54:34 tool: Bash

```json
{
  "command": "find /c/Users/yaniv/Projects/cache-evolve -maxdepth 4 -type d 2>/dev/null | head -40",
  "description": "Find project directory structure"
}
```

### 19:54:37 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 19:54:39 tool: Read

```json
{
  "file_path": "/c/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 19:54:39 tool: Read

```json
{
  "file_path": "/c/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 19:55:07 thinking

The output directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\cross-cycle`. I need to write `cross-cycle-insights.md` there.

Now let me synthesize the insights from all 9 cycles:

**Recurring wins:**
1. 32-byte aligned struct (from Cycle 2 onward) — eliminated p99 tail spikes from cache-line straddling; must be carried forward in every variant
2. 3-swap bounded promotion loop — found optimal in Cycle 4/5, confirmed ceiling in Cycle 6; early-break makes it effectively O(1) on hot Zipfian keys
3. Second-chance eviction scan with write-once flag and 8-slot window using LRU-among-cold ordering — emerged as the strongest hit-rate lever in Cycles 6-8
4. LRU-among-cold scan direction — delivers free hit-rate improvement with no latency penalty (Cycle 8)
5. Unconditional positional recency on the get path outperforms frequency-gated promotion

**Recurring failures:**
1. Per-get frequency counter writes — consistently degrade p99 tail latency across multiple cycles (Cycles 1, 2, 5)
2. Two-zone partitioning approaches — tested in Cycle 3, rejected due to boundary instability
3. Swap-to-back promotion — caused double regression (Cycle 2)
4. Bucket geometry tuning — halving buckets regressed performance (Cycle 6), wider layouts increased scan overhead (Cycle 1)
5. Frequency signals at eviction time — two-tail frequency comparison hurt hit rate (Cycles 5, 6)
6. Compact struct omission — recurring issue in Cycles 7 and 9 where dropping the item.rs overlay reverts to the 40-byte baseline
7. Sub-3-swap or zero-swap promotion — single swap proved too conservative (Cycle 7), zero swap offered no latency gain and hurt hit rate
8. Scan depth tuning — 4-slot window too narrow, full-depth unbounded; 8-slot confirmed as optimal (Cycle 8)
9. Fallback index mi

-inference — Cycle 9 inverted layout direction with anti-LRU selection; generation counters and flag expiry on position both rejected

Still investigating the put p99 regression from Cycle 6's v003 — swap elimination ruled out write pressure, but the root cause (metadata layout artifact or scan-fallback edge case) remains unclear. The 3-swap plus 8-slot plus LRU-among-cold plus write-once-flag configuration might be near a local optimum, but I'm uncertain whether there are structurally distinct axes like insertion-point policy or adaptive scan depth still unexplored. I'm also wondering if frequency information can be incorporated without per-get writes, perhaps through sampled or epoch-based counters updated only on eviction or put, and the 100 ns measurement floor is masking potential latency gains.

Per-policy analysis shows LRU remains the correct baseline with strict tail eviction; LFU variants achieve real hit-rate gains on scan-heavy traces but hit O(n) scan costs or per-get counter write overhead, while the binary "ever-accessed" flag is the closest LFU-inspired approach that avoids write-path overhead. The second-chance mechanism with write-once flag, 8-slot window, and LRU-among-cold ordering is the best configuration found so far, and these mechanisms are tightly coupled. Two-zone partitioning has been definitively ruled out.

### 19:55:07 chat

Now I have enough context. The output directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0009\cross-cycle`. Let me synthesize across all 9 cycles and write the file.

### 19:55:45 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_16-31-57-327\\cycles\\0009\\cross-cycle\\cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **32-byte aligned struct is a required invariant.** Padding the compact item to 32 bytes eliminated cache-line straddling and halved p99 tail latency (Cycle 2). Every variant that has omitted this layout — including three consecutive cycles (7, 9) — reverted to 40-byte baseline footprint and immediately regressed on put-path latency. It must be carried explicitly in every file overlay.\n- **3-swap bounded promotion loop is the confirmed optimal promotion budget.** One swap is too conservative (items age before converging); five swaps causes slight hit-rate regression by displacing items that cycle through the top positions. Early-break on a failed comparison makes the common Zipfian hot-key case exit after one iteration, keeping the loop effectively O(1) in practice (Cycles 4–6).\n- **Second-chance write-once flag + 8-slot window + LRU-among-cold ordering is the highest-value hit-rate configuration found.** The binary \"ever-accessed\" marker requires one write at first touch only; the 8-slot depth is the confirmed sweet spot (4 too narrow, full-depth adds no benefit); LRU-among-cold direction (evict the oldest cold candidate, not the newest) converted reuse-eligible items that LIFO discarded too early. Together these three parameters form the confirmed eviction base (Cycles 6–8).\n- **Unconditional positional promotion outperforms frequency-gated promotion on this workload.** Removing the LFU neighbour-check gate and always advancing mid-bucket items kept the cold tail genuinely cold for eviction. The frequency gate stranded recently-reused items below cold neighbours and degraded hit rate (Cycles 4–5).\n- **Position-0 early-return guard is a valid tail-latency lever.** Skipping the bubble loop when the accessed item is already at the head eliminated unnecessary writes and halved p99 for hot keys — the principle is sound even though capping at exactly one swap was too conservative (Cycle 3).\n\n## Recurring failures\n\n- **Per-get frequency counter writes consistently raise p99 tail latency.** Across Cycles 1, 2, and 5, any variant that incremented an LFU counter on the hot get path surfaced p99 pressure; the write itself, not conditional logic around it, appears to be the cache-line liability. Frequency signals must not touch the get path.\n- **Frequency signals at eviction time degrade hit rate on this Zipfian workload.** Two-tail frequency comparison and generation-counter priority passes both overrode clean positional recency and evicted recently-promoted items before genuine LRU victims. No frequency-at-eviction formulation has improved upon strict positional tail eviction across two separate cycles of testing (Cycles 5–6, 9).\n- **Decoupling the second-chance scan from 3-swap promotion destroys eviction quality.** Reducing to one swap or eliminating promotion entirely lets hot items age into the scan window alongside cold ones, stripping the scan's discriminating power. The two mechanisms are coupled: promotion maintains hot/cold separation that the scan relies on (Cycle 7).\n- **Two-zone hot/cold partitioning is definitively closed.** Boundary items are continuously pushed back into the cold zone with every insertion; the hot-zone bypass saves no real work because baseline LRU already keeps hot items where removal is O(1). Tested in Cycle 3, failure replicated in principle in Cycle 6 (bucket-count halving). Do not revisit.\n- **Swap-to-back (in-place swap to avoid remove+push) is a double regression.** The bounded linear shift is not the latency bottleneck at small bucket sizes, and the ordering approximation degrades hit quality on Zipfian workloads (Cycle 2). This axis is closed.\n- **Compact struct omission is the most persistent variant-authoring failure.** Variants that overlay only `eviction_policy.rs` without explicitly carrying the `item.rs` layout have silently reverted to the 40-byte baseline in Cycles 7 and 9. Must be treated as an explicit checklist item, not an incidental fix.\n- **Flag-lifetime modifications (expiry on position, generation-based gating) all regressed.** Position is not a reliable proxy for access staleness on Zipfian workloads; legitimately warm items accumulate near the cold tail through low-but-nonzero frequency. The binary ever/never distinction already captures the meaningful split (Cycle 9).\n- **Bucket-count scaling in either direction is closed.** Halving buckets raised collision density faster than depth could absorb; wider geometry increased entries touched per lookup. The current bucket geometry is not a productive tuning axis (Cycles 1, 6).\n\n## Open questions\n\n- **The Cycle 6 v003 put p99 regression has never been explained.** Swap-elimination experiments in Cycle 7 ruled out write pressure as the cause. The actual source — metadata-layout artifact from a missed struct carry-forward, scan-fallback edge case, or scan-window behavior on put-heavy patterns — remains undiagnosed and blocks understanding of what combinations are safe.\n- **Has the confirmed base converged near a local optimum?** The 3-swap + 8-slot + LRU-among-cold + write-once-flag combination has been stable for three cycles. Structurally distinct axes — insertion-point policy (placing new items somewhere other than the conventional tail) and adaptive scan depth — have not been explored and represent the next meaningful frontier.\n- **Can frequency information be incorporated without per-get writes?** Sampled or epoch-based counters updated only on eviction or put paths have not been tested. The write-once flag is the closest approximation found; true frequency-weighted eviction within the cold window without get-path overhead is still an open hypothesis.\n- **The 100 ns timer-resolution floor masks latency differentiation.** Per-op-time results have been flat at the floor for multiple cycles; v003 in Cycle 5 produced the only clear p99 signal by generating enough write pressure to clear the noise. Whether genuine sub-floor latency gains exist requires either higher operation count or a trace long enough to amortise measurement noise.\n\n## Per-policy notes\n\n- **LRU (strict positional recency):** Baseline policy; strict tail eviction is the eviction rule that all successful variants are built on top of. No frequency signal applied at eviction time has outperformed it across six cycles of testing (Cycles 1, 5, 6, 9).\n- **LFU-inspired variants:** Hit-rate gains are real on scan-heavy traces (Cycle 1) but have been blocked in every formulation by either O(n) scan cost or per-get counter write overhead. The write-once \"ever-accessed\" binary flag is the only LFU-derived mechanism that avoids write-path overhead and has produced durable gains.\n- **Second-chance (confirmed base):** The current best configuration combines write-once flag + 8-slot scan window + LRU-among-cold ordering + 3-swap promotion. These four parameters are confirmed individually and as a unit. The mechanisms are coupled: scan quality depends on promotion maintaining hot/cold separation. Any future variant must carry all four components.\n- **Two-zone partitioning:** Definitively closed after Cycle 3. Hot/cold boundary instability is structural, not tunable, at these bucket sizes.\n- **Swap-to-back promotion:** Closed after Cycle 2. Not the latency bottleneck; harms ordering quality at small associativity.\n"
}
```

### 19:55:55 chat

`cross-cycle-insights.md` written to `outputs/associative_cache_2026-07-21_16-31-57-327/cycles/0009/cross-cycle/`. Here's what the four sections capture across all nine cycles:

**Recurring wins** — five durable patterns: the 32-byte aligned struct as a required base, the 3-swap promotion loop as the confirmed optimal budget, the second-chance scan's three confirmed parameters (write-once flag + 8-slot window + LRU-among-cold order), unconditional positional promotion beating frequency-gated promotion, and the position-0 early-return guard as a valid tail-latency lever.

**Recurring failures** — eight closed avenues: per-get counter writes, frequency signals at eviction time, decoupling scan from promotion, two-zone partitioning, swap-to-back promotion, compact struct omission (the most persistent authoring failure), flag-lifetime modifications, and bucket-count scaling in either direction.

**Open questions** — four unresolved threads: the Cycle 6 put p99 regression (still undiagnosed), whether the confirmed base has hit a local optimum (insertion-point policy and adaptive scan depth are unexplored), frequency integration without get-path writes, and the measurement-floor ambiguity.

**Per-policy notes** — observations grouped by LRU, LFU-inspired variants, second-chance (confirmed base), two-zone partitioning, and swap-to-back promotion.


---

- finished: 2026-07-21T19:55:55+00:00
- duration: 88.12s
- status: ok
- input_tokens: 7
- output_tokens: 3739
- cache_read_tokens: 122129
- cache_write_tokens: 12192
- cost_usd: 0.145159
