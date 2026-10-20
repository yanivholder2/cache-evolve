# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\cross-cycle
- allow_edits: True
- started: 2026-07-21T09:27:02+00:00

## timeline

### 09:27:08 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look at the project structure to find the output directory.

### 09:27:08 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 09:27:08 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 09:27:10 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 09:27:11 tool: Bash

```json
{
  "command": "ls -la",
  "description": "List files in working directory"
}
```

### 09:27:14 tool: Read

```json
{
  "file_path": "/c/Users/yaniv/Projects/cache-evolve/task.md",
  "limit": 30
}
```

### 09:27:14 tool: Read

```json
{
  "file_path": "/c/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 09:27:17 thinking

The output directory is `C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0010\cross-cycle`. I should write the file there.

### 09:27:56 tool: Write

```json
{
  "file_path": "C:/Users/yaniv/Projects/cache-evolve/outputs/lru_cache_2026-07-19_19-26-43-995/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Structural compaction without behavioral change is the most reliable win.** Reducing Node size (u32 sentinel links → 16-byte node via link-array split, FNV-1a zero-sized hasher) has delivered clean, cost-free gains across multiple cycles. These changes improve cache-line utilization and never regress latency.\n- **Protected-quota enlargement reliably improves hit-rate** on the Zipfian workload. Each successive halving of the probationary pool (75% → 87.5%) produced a measurable, reproducible hit-rate gain confirmed across independent parent configurations.\n- **Simple, unconditional policy changes outperform conditional shortcuts.** The pure quota enlargement (v001 in cycles 0006–0007) beat every variant that added branches, flags, or multi-path logic to the hot get path.\n\n## Recurring failures\n\n- **CLOCK-style and sweep-based eviction approaches fail repeatedly.** Tested in cycles 0001, 0004, and later: each variant pays either cache-line pollution from the hand sweep or header bloat from the bitmap `Vec`, and amortised relinking never pays back at full-cache steady state where demotion and promotion rates are coupled.\n- **Two-hit / SEEN_BIT admission filtering is conclusively wrong for this workload.** Tested across cycles 0004–0006 in three distinct implementations (splice-on-first-hit, cost-free bit-OR, eviction-path rescue). All failed because the workload does not produce enough within-window repeat accesses for stricter graduation to be net beneficial.\n- **Frequency sketches and CountMin-based mechanisms provide no discrimination here.** Tested in cycles 0007 (CountMin graduation gate) and 0010 (probationary frequency-biased eviction). At the small probationary pool sizes implied by high protected quotas, counter aliasing and short item lifetimes reduce the sketch to noise. The hashing cost on every get to maintain counts adds measurable overhead.\n- **Quota compression above 87.5% always regresses per-op latency.** Confirmed across at least four independent strategies: intermediate quota, demotion batching, FNV-1a offset, and lazy-demotion flag bits. The coupling is structural to the current two-queue policy: any conditional that fires in proportion to overflow frequency surfaces above the timer floor regardless of how cheaply it is implemented.\n- **Forced inlining and `entry()` API tricks backfire.** Cycle 0002/v003 demonstrated that `#[inline(always)]` can push hot paths out of ideal instruction-cache placement, and the borrow-checker eviction-path constraint erases the promised single-probe benefit of the `entry()` API.\n- **MRU-tail guard generalisation failed.** The guard won in isolation (cycle 0002) but failed when composed with the 2Q layout in cycle 0003 because the guard's fire rate on the broader workload was too low to amortise the per-access check.\n- **Adaptive mechanisms are wasted on a stationary Zipfian distribution.** Adaptive quota sizing (cycle 0004), demotion batching (cycle 0009), and demotion reordering (cycle 0007) all paid unconditional overhead for logic that barely fired or measured no effect because the hot set is stable.\n\n## Open questions\n\n- **FNV-1a's struct-size win is invisible to the current fitness scorer**, which tracks Node size rather than `LruCache` instance size. Should the fitness function be extended to credit cache-struct footprint? This would unlock a class of improvements currently ranked as neutral.\n- **Is there a pointer-indirection depth reduction per promotion?** Encoding queue membership and LRU position in a single index update rather than two separate link-field writes might lower baseline get cost enough to open headroom for a modest quota increase above 87.5%.\n- **Is there a policy topology that avoids per-get demotion enforcement entirely?** A ghost-entry or clock-hand variant where the protected/probationary boundary is enforced lazily by position rather than active relinking was proposed but never tested. It is the last structurally distinct policy direction remaining.\n- **Does frequency-sketch-based eviction ordering become viable at larger probationary pools?** All tests have been at capacity/8 or smaller; the mechanism's failure is partly a consequence of deep-quota operation and might behave differently at shallower protected shares.\n- **Can the key-clone elision optimization be confirmed on non-`Copy` key types?** All benchmarks use `Copy` keys where `.clone()` is a register copy; the eviction-path refactor hypothesis remains untested under conditions where clone cost is non-trivial.\n\n## Per-policy notes\n\n- **Two-queue (2Q) with u32 sentinel links is the current structural baseline.** Established in cycle 0001 (sentinel win) and refined through cycle 0002 (protected-bit-in-prev encoding). Node size is 16 bytes after the cycle 0005 link-array split. The FNV-1a zero-sized hasher (cycle 0010/v001) is the latest settled improvement, reducing the `LruCache` struct by 16 bytes.\n- **Protected quota at 87.5% (capacity/8 probationary) is the confirmed best hit-rate-without-latency-penalty operating point.** Values above this (91.67%, 93.75%) consistently incur a measurable get p50 penalty regardless of what offsetting changes are applied alongside them. Values at or below 75% represent the prior structural baseline.\n- **CLOCK policy:** Failed in cycle 0001 due to hand-sweep cache-line pollution, and again in cycle 0004 due to `Vec` header bloat. The fundamental issue — amortised relinking only pays when promotions vastly outnumber demotions, which does not hold at full-cache steady state — appears inherent to this workload.\n- **Vanilla LRU (single queue):** Not explicitly tested after cycle 0001 established 2Q as superior. The 2Q + 87.5% quota is so far ahead on hit-rate that revisiting single-queue variants is not warranted.\n- **Admission filters overlaid on 2Q:** All forms tested (2Q, SEEN_BIT, CountMin) regressed hit-rate on this workload. Eviction ordering within the probationary queue (frequency-biased) also failed. The probationary pool is too small and item dwell time too short for any secondary signal to accumulate.\n"
}
```

### 09:28:05 chat

`cross-cycle-insights.md` written to `cycles/0010/cross-cycle/`. Here is what the four sections capture across all ten cycles:

**Recurring wins** — structural compaction (sentinel links, link-array split, FNV-1a zero-sized hasher) and incremental protected-quota growth to 87.5% are the two axes that have produced clean, repeatable gains. Simple unconditional changes consistently beat conditional shortcuts.

**Recurring failures** — CLOCK/sweep variants (two independent failures), all two-hit/SEEN_BIT admission variants (three implementations, same root cause), frequency sketches at small probationary pool sizes, any quota above 87.5% (confirmed from four independent offsetting strategies), and micro-optimization tricks that fight the borrow-checker or instruction cache.

**Open questions** — the fitness scorer's blind spot on `LruCache` struct size, the untested pointer-indirection depth reduction, a ghost-entry/lazy-boundary policy topology never yet implemented, and key-clone elision on non-`Copy` key types.

**Per-policy notes** — anchors the 2Q + 87.5% quota + FNV-1a + 16-byte Node as the settled current baseline and documents why CLOCK and single-queue variants are deprioritized.


---

- finished: 2026-07-21T09:28:05+00:00
- duration: 63.20s
- status: ok
- input_tokens: 6
- output_tokens: 2439
- cache_read_tokens: 107332
- cache_write_tokens: 13189
- cost_usd: 0.126754
