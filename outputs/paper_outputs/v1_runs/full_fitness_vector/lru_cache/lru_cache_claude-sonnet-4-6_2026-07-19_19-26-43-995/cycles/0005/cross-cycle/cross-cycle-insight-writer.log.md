# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\cross-cycle
- allow_edits: True
- started: 2026-07-19T18:03:07+00:00

## timeline

### 18:03:11 thinking

Let me analyze the ledger digest across all 5 cycles and write the cross-cycle-insights.md file. I need to find the output directory first.

### 18:03:12 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 18:03:12 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 18:03:13 tool: Bash

```json
{
  "command": "ls /",
  "description": "List root directory"
}
```

### 18:03:13 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 18:03:15 tool: Read

```json
{
  "file_path": "/task.md"
}
```

### 18:03:17 thinking

The output dir is `C:\Users\yaniv\Projects\cache-evolve\outputs\lru_cache_2026-07-19_19-26-43-995\cycles\0005\cross-cycle`. Let me write the file there.

### 18:03:50 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\lru_cache_2026-07-19_19-26-43-995\\cycles\\0005\\cross-cycle\\cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Structural compaction of node metadata** has been the most reliable lever across all cycles. Encoding spare information into existing fields (u32 sentinel links, protected-bit in prev pointer, SEEN_BIT for graduation) consistently avoids layout regressions while unlocking behavioral improvements.\n- **Simplicity over complexity in the hot path.** The cleanest wins have come from removing work (MRU-tail guard skipping no-op promotes, SEEN_BIT deferred graduation reducing splice cost) rather than adding logic. Every cycle where extra branching was introduced on the common path paid a measurable per-op penalty.\n- **\"Do no harm\" is a winning bar at this resolution.** The per-op budget is tight enough that cost-neutral structural changes (link-array split, bit-OR-only first-access path) count as wins even without positive latency signals, because they preserve headroom for future composition.\n- **Two-queue semantics fit this workload's reuse pattern.** Across cycles 0001–0005 the 2Q layout consistently outperformed or equalled pure LRU on hit-rate, and the protected-bit encoding proved that 2Q need not increase node size.\n\n## Recurring failures\n\n- **CLOCK/bitmap-backed deferred relinking has failed twice** (cycles 0001 and 0004) for the same structural reason: amortising relinking only pays when promotions vastly outnumber demotions, but on a full cache those rates are coupled, and the bitmap or hand-sweep overhead dominates.\n- **Adaptive/reactive policy mechanisms are wasted on a stationary Zipfian trace.** Adaptive quota sizing (cycle 0004) and MRU-tail guards (cycle 0003) fire too rarely on the stable hot set to amortise their per-op check cost. Logic designed for workload phase shifts is invisible overhead on a stationary distribution.\n- **Micro-optimisation hints (`#[inline(always)]`, `entry()` API) backfire.** Forced inlining can push a hot path out of ideal I-cache placement; the borrow-checker's eviction-path constraint erases the single-probe benefit of the `entry()` API entirely. Both traps were confirmed in cycle 0002.\n- **Composing two independent wins does not guarantee a net win.** Cycles 0003 and 0004 both tried layering prior-cycle winners; each time cumulative branch and path-length overhead outweighed the individual gains on the same workload.\n- **Key-clone elision remains unvalidated.** The benchmark uses `Copy` keys, so the hypothesis is structurally sound but empirically untestable in the current fixture. Every attempt to carry it forward has stalled at the fixture problem.\n\n## Open questions\n\n- **Can the SEEN_BIT two-hit filter and an enlarged protected quota be composed cleanly?** The hypothesis is that a deeper protected pool compensates for stricter admission: items that survive to a second hit are shielded long enough to accumulate further hits. This is the most actionable open thread.\n- **Does the quota-enlargement per-op regression persist on the link-array baseline?** The cycle 0005 signal was ambiguous (possible noise from a near-zero p50 floor). Retesting on the v002 base — where the measurement floor is non-zero — would settle whether the cost is real or artefact.\n- **What is the right benchmark fixture for key-clone elision?** A `String`- or `Arc<str>`-keyed workload is needed before this direction can be confirmed or discarded. Without it, a potentially meaningful put-path win remains in limbo.\n- **Can the two-hit filter be made workload-sensitive?** Applying strict two-hit admission uniformly hurts hit-rate for items that will not recur; an adaptive or frequency-informed threshold might recover quality without reverting to single-hit graduation.\n\n## Per-policy notes\n\n- **Pure LRU (baseline):** The structural starting point. Defeated primarily by hit-rate on Zipfian traces, not by per-op cost. Never the right eviction policy for this workload, but remains the throughput reference.\n- **Two-queue (2Q):** The dominant policy throughout. Protected-bit encoding in `prev` kept node size flat. Hit-rate superiority over LRU is consistent. Main remaining tension is the cost of unlink/push_back_prot on the protected path, which grows as quota enlarges.\n- **CLOCK / single-bit recency:** Failed in both forms tried (hand-sweep in cycle 0001, bitmap in cycle 0004). Cache-line pollution from the sweep, or Vec-header bloat, offset any structural gain. Not worth revisiting without a concrete mechanism to avoid sweep-induced cache pressure.\n- **Two-hit probationary admission (SEEN_BIT):** Structurally validated in cycle 0005 — bit-OR on first access is free. Policy is miscalibrated for this trace's hit frequency distribution: enough single-access items exist that strict two-hit graduation hurts hit-rate. Best pursued in combination with a larger protected quota.\n- **MRU-tail guard (promote shortcut):** Won in cycle 0002 on its own workload slice, failed in cycle 0003 when composed with 2Q. Fire rate is workload-dependent; workload-a is too diffuse for the guard to cover enough accesses. Do not revisit without evidence of a higher fire rate.\n- **Adaptive quota:** Directionally correct in theory; practically inert on a stationary Zipfian trace. No adjustment epoch was ever triggered in testing. Revisit only if a phase-shifting or bursty workload is introduced.\n"
}
```

### 18:04:00 chat

Written `cross-cycle-insights.md` to the output directory. Here's a summary of what the five-cycle projection captured:

**Recurring wins:** Node metadata compaction (u32 sentinels, protected-bit encoding, SEEN_BIT) is the most reliable lever — every successful structural change reused an existing field rather than adding bytes. Two-queue semantics consistently beat LRU on this workload. The "do no harm" bar is surprisingly meaningful given the tight per-op budget.

**Recurring failures:** CLOCK/deferred-relink approaches have failed twice for the same structural reason. Adaptive mechanisms are invisible overhead on a stationary Zipfian trace. Composing two independent winners has never produced a net win — cumulative branch overhead dominates.

**Open questions:** The most actionable thread is composing the SEEN_BIT two-hit filter with a larger protected quota (the natural next cycle). Key-clone elision and the quota-enlargement noise question are the other unresolved threads.

**Per-policy notes:** Summarize each policy's track record — 2Q is the durable winner, CLOCK has failed in both forms, the MRU-tail guard is too workload-sensitive to rely on, and SEEN_BIT admission is structurally solved but not yet policy-tuned.


---

- finished: 2026-07-19T18:04:00+00:00
- duration: 53.57s
- status: ok
- input_tokens: 6
- output_tokens: 2121
- cache_read_tokens: 90337
- cache_write_tokens: 8053
- cost_usd: 0.093423
