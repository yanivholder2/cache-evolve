# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0003\cross-cycle
- allow_edits: True
- started: 2026-07-19T17:20:06+00:00

## timeline

### 17:20:10 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look for the output directory in the project structure.

### 17:20:10 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 17:20:11 tool: Glob

```json
{
  "pattern": "output*/**"
}
```

### 17:20:13 tool: Bash

```json
{
  "command": "ls /c/Users/yaniv/Projects/cache-evolve/",
  "description": "List project root"
}
```

### 17:20:15 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 17:20:18 tool: Read

```json
{
  "file_path": "task.md"
}
```

### 17:20:20 thinking

Now I need to find the output directory. Let me look at the log file to get some context, and also check the directory structure.

### 17:20:21 tool: Read

```json
{
  "file_path": "cross-cycle-insight-writer.log.md"
}
```

### 17:20:21 tool: Glob

```json
{
  "pattern": "**/*"
}
```

### 17:20:50 thinking

The workspace is `C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0003\cross-cycle`. The output directory should be the same as the workspace since the task says "in the output dir". Looking at the task, it says to write `cross-cycle-insights.md` in the output dir. Given that the workspace is the cross-cycle directory, and both files (task.md and the log) are in the current working directory, I should write the file in the current directory.
Now I'm synthesizing the key patterns across all three cycles. The structural node compaction using u32 sentinel links emerged as the most consistent win—it reduced node size and improved cache-line utilization while maintaining eviction ordering precision. Beyond that, the behavioral and policy changes like the 2Q protected-bit encoding and MRU-tail guard proved more effective than micro-optimizations, suggesting that algorithmic shifts outperform code-level tuning.

On the flip side, I'm seeing clear failure patterns: micro-optimizations like `#[inline(always)]`, the `entry()` API, and CLOCK-style approaches all backfired by introducing hidden costs—cache-line pollution, I-cache displacement, and borrow-checker friction that negated their intended benefits. And when I tried stacking the two winners from an earlier cycle together, they didn't compound; the combination produced no net improvement, which suggests that winners don't always compose cleanly.

Now I'm realizing that orthogonal single-path optimizations accumulate branch and path-length overhead when the workload is mixed. The MRU-tail guard is a good example—its benefit hinges entirely on how often it fires, and on diffuse workloads it fires rarely enough that the per-call check cost dominates, making it a fragile approach without workload-specific fire-rate tuning.

I'm left with some open questions: whether adaptive quota sizing could dynamically shift the protected/probationary boundary based on hit rates to capture the benefits of a larger protected quota without the permanent cost of more unlinks and relinks, and whether key-clone elision on non-`Copy` types like `String` or `Arc<str>` would actually reduce allocations in the eviction path.

There's also the question of how much further node packing can go toward a single cache line before hitting diminishing returns, whether CLOCK's cache-line pollution could be mitigated with a lazy-sweep variant instead of the eager hand sweep, and whether the MRU-tail guard could be extended to the probationary tail or redesigned with a higher fire rate on diffuse workloads.

Looking at the per-policy results, LRU with the u32 sentinel baseline remains the structural winner—exact ordering precision pays off on reuse-heavy workloads, and compaction that preserves the chain is worth the cost. Two-queue won in cycle 0002 by folding queue membership into a spare bit.

### 17:21:07 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Structural node compaction preserving the LRU chain** pays off reliably. The u32 sentinel baseline (cycle 0001) improved cache-line utilization and get latency without sacrificing eviction precision, and has held up as the canonical structural foundation across all subsequent cycles.\n- **Meaningful policy or algorithmic changes outperform code-level hints.** The 2Q protected-bit-in-prev encoding and the MRU-tail guard both won in cycle 0002 by changing runtime behavior; every attempt at low-level hints (`#[inline(always)]`, `entry()` API, CLOCK) produced the opposite effect.\n- **Spare-bit encoding enables policy enrichment at zero node cost.** Folding 2Q queue membership into the `prev` field's spare bit kept the node at 24 bytes while unlocking full two-queue semantics — compaction and richer policy are not mutually exclusive.\n\n## Recurring failures\n\n- **Algorithm-complexity micro-optimizations consistently backfire.** CLOCK hand sweeps introduced cache-line pollution; `#[inline(always)]` on small helpers pushed hot code out of I-cache; the `entry()` API hit borrow-checker walls on the dominant eviction path (occupied `VacantEntry` must be released before `map.remove()`). All three added hidden cost that erased their structural benefit.\n- **Composing prior-cycle winners does not produce additive gains.** Stacking 2Q and the MRU-tail guard in cycle 0003 accumulated cumulative branch and path-length overhead that outweighed each individual contribution on the mixed workload. Orthogonal single-path wins do not compose freely.\n- **Guard-style shortcuts are workload-conditional, not universal.** The MRU-tail guard's value is entirely determined by its fire rate. On a diffuse workload the guard fires rarely, making the per-call check the dominant cost. Guards should not be carried forward without fire-rate analysis matched to the target trace.\n\n## Open questions\n\n- **Adaptive quota sizing**: can dynamically shifting the protected/probationary boundary based on observed demotion pressure capture the hit-quality benefit of a larger protected budget without permanently paying the higher unlink/relink cost?\n- **Key-clone elision on non-`Copy` types**: the eviction-path refactor could not be validated with `Copy` keys (`.clone()` is a register copy). A `String` or `Arc<str>` key benchmark is needed before the idea is confirmed or discarded.\n- **Node size floor**: no cycle has directly probed the density ceiling. How far can the node be packed (toward a single cache line per entry) before returns diminish?\n- **CLOCK with lazy or segmented sweep**: cache-line pollution appeared inherent to the eager hand sweep; a lazy-sweep or arena-segmented variant has not been evaluated and may recover the structural win.\n- **Guard fire-rate engineering**: can the MRU-tail guard be extended to the probationary tail, or reformulated around a condition with a higher fire rate on diffuse workloads, to make it a robust rather than workload-specific win?\n\n## Per-policy notes\n\n- **LRU (u32 sentinel baseline)**: the structural winner of cycle 0001 and the ongoing baseline. Exact ordering precision is valuable on reuse-heavy workloads; compaction that preserves chain traversal (rather than replacing it with a sweep) consistently pays off on the get path.\n- **Two-queue (2Q)**: won cycle 0002 via spare-bit encoding with no node growth. Protected-quota expansion in cycle 0003 confirmed hit-rate sensitivity to the protected budget, but quality and throughput trade off monotonically as the quota grows — the lever is real but expensive at the extremes. Adaptive quota sizing is the main open hypothesis.\n- **CLOCK**: underperformed in every evaluation. Eager hand sweeps cause cache-line pollution that erases structural gains; single-bit recency signals reduce hit rate on reuse workloads. Deprioritized unless a concrete mechanism to avoid sweep-induced pollution is proposed.\n- **MRU-tail guard (shortcut within LRU/2Q)**: a workload-conditional optimization. Won on cycle 0002's specific trace; failed to generalize in cycle 0003 on a more diffuse workload. Should not be reinstated without a fire-rate justification for the target workload.\n"
}
```

### 17:21:13 chat

`cross-cycle-insights.md` written. Here's a summary of what the four sections capture across the three completed cycles:

**Recurring wins** — node compaction that preserves the LRU chain (u32 sentinel baseline) is the single most reliable lever; spare-bit policy encoding achieves richer eviction semantics at zero node-size cost; behavioral/policy changes consistently beat code-level hints.

**Recurring failures** — algorithm-complexity micro-optimizations (CLOCK, `#[inline(always)]`, `entry()` API) all added hidden cost that erased their intended benefit; composing prior-cycle winners produced cumulative overhead rather than additive gains; guard-style shortcuts are workload-conditional and fail when fire rate is low.

**Open questions** — adaptive quota sizing for 2Q; key-clone elision validation against non-`Copy` key types; node size floor; CLOCK with lazy/segmented sweep; guard fire-rate engineering for diffuse workloads.

**Per-policy notes** — LRU baseline remains solid; 2Q's main open lever is adaptive quota sizing; CLOCK is deprioritized; MRU-tail guard is flagged as workload-conditional, not universal.


---

- finished: 2026-07-19T17:21:14+00:00
- duration: 67.81s
- status: ok
- input_tokens: 8
- output_tokens: 2990
- cache_read_tokens: 124939
- cache_write_tokens: 9821
- cost_usd: 0.121898
