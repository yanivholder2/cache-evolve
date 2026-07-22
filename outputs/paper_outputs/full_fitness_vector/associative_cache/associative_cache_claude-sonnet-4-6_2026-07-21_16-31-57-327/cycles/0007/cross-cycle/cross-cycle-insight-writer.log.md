# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-4-6)
- workspace: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\cross-cycle
- allow_edits: True
- started: 2026-07-21T17:45:00+00:00

## timeline

### 17:45:06 thinking

The user wants me to write a `cross-cycle-insights.md` file based on the ledger digest provided. I need to find the output directory first, then write the file with the four fixed sections.

Let me look for the output directory in the project structure.

### 17:45:06 tool: Glob

```json
{
  "pattern": "**/cross-cycle-insights.md"
}
```

### 17:45:06 tool: Glob

```json
{
  "pattern": "output/**"
}
```

### 17:45:08 tool: Bash

```json
{
  "command": "ls /c/Users/yaniv/Projects/cache-evolve/",
  "description": "List project root"
}
```

### 17:45:09 tool: Glob

```json
{
  "pattern": "**/*.md"
}
```

### 17:45:11 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/cross-cycle-insight-writer.log.md"
}
```

### 17:45:11 tool: Read

```json
{
  "file_path": "/Users/yaniv/Projects/cache-evolve/task.md"
}
```

### 17:45:37 thinking

I now know the output directory: `C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\cross-cycle`

I need to write `cross-cycle-insights.md` in that directory.

Now let me synthesize the insights from all 7 cycles and write the file.

## Recurring wins

Looking across cycles:
1. **32-byte aligned struct** - Confirmed in cycle 0002 as fix for p99 spikes. Every cycle that drops it regresses. This is a structural invariant now.
2. **3-swap bounded promotion loop** - Cycle 0005 proved this is the sweet spot for Zipfian workloads, with early-break keeping the hot path to a single iteration. Cycle 0006 showed that 5-swaps regresses performance.
3. **Second-chance eviction scan with write-once flag** - Emerged as the strongest hit-rate lever in cycle 0006.
4. **Strict tail eviction without frequency signals** - Cycles 0005 and 0006 consistently showed that applying frequency information at eviction time hurts hit rates.
5. **Position-0 guard** - Validated in cycle 0003 as a critical protection mechanism.

The pattern of failures is equally instructive: swap-to-back and single-swap promotion degrade ordering quality, two-zone partitioning causes boundary thrashing without latency gains, per-get frequency writes introduce tail-latency risk, halving the bucket count replicates earlier regressions, decoupling the second-chance scan from 3-swap promotion breaks the mechanical coupling that makes them work together, and frequency signals at the eviction boundary consistently harm performance.

The unresolved questions center on what triggered the put p99 regression in cycle 0006, whether there's a write-free mechanism to maintain the hot/cold separation the 3-swap loop provides, whether frequency information can be sampled or epoch-based rather than per-get, and whether the 100ns timer floor is masking meaningful latency improvements below that threshold.

For LRU, the 3-swap unconditional promotion with the position-0 early-return guard is the best configuration, while reducing bucket count consistently hurts performance through increased lookup cost. For LFU, per-get counter writes create tail-latency pressure despite improving eviction quality, so write-once binary flags at first-touch are the only viable frequency mechanism. The second-chance scan combined with the 3-swap loop achieved the best hit-rate improvement in cycle 0006, though the scan and promotion are mechanically coupled—reducing swaps degrades scan quality by letting hot items age into the cold tail, which introduced the latency regression.

Two-zone partitioning and two-tail frequency comparison at eviction have both been exhausted across multiple cycles and consistently fail, so those approaches are closed off.

### 17:45:37 chat

Now I have the output path. Let me synthesize all 7 cycles and write the file.

### 17:46:02 tool: Write

```json
{
  "file_path": "C:\\Users\\yaniv\\Projects\\cache-evolve\\outputs\\associative_cache_2026-07-21_16-31-57-327\\cycles\\0007\\cross-cycle\\cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **32-byte aligned struct is a required structural invariant.** Confirmed in cycle 0002 as a precise fix for p99 cache-line straddling; every variant that drops it reintroduces footprint bloat and put-path latency. Must be carried forward explicitly in every variant.\n- **3-swap bounded promotion loop is the confirmed optimal promotion budget.** Early-break on a failed swap keeps the Zipfian hot-key path at one iteration. Cycle 0005 confirmed the win cleanly; cycle 0006 confirmed five swaps regress. The useful promotion-budget exploration range is exhausted.\n- **Position-0 early-return guard reliably cuts tail latency.** Skipping the promotion loop for already-head items halved p99 in cycle 0003 without harming throughput. The guard is a safe, composable addition to any positional scheme.\n- **Second-chance eviction with a write-once binary flag is the strongest hit-rate lever found.** First-touch marking lets the eviction scan skip warm items and target genuinely cold newcomers. It avoids hot-path write cost and won in cycle 0006 when paired with the 3-swap loop.\n- **Strict positional tail eviction (no frequency signal at eviction time) consistently outperforms frequency-augmented alternatives** on the Zipfian/LIRS workloads used throughout. The eviction-quality gain comes from promotion ordering, not from counter-based selection at the eviction site.\n\n## Recurring failures\n\n- **Per-get frequency counter writes reliably introduce tail-latency pressure.** Tested in cycles 0001, 0005, and 0006. The write itself — not the conditional logic — is the cache-line liability. No frequency-counter approach applied on the hot get path has survived the latency constraint.\n- **Two-tail / frequency-gated eviction selection consistently degrades hit rate.** Cycle 0005 (v002) and cycle 0006 both showed that comparing frequency at the eviction boundary ejects recently-promoted items before genuine LRU victims. This avenue is closed.\n- **Two-zone (hot/cold) partitioning is definitively closed.** Boundary thrashing degrades hit rate faster than the hot-zone bypass saves work. Tested in cycle 0003 and not worth revisiting.\n- **Swap-to-back / single-swap promotion is definitively closed.** Ordering approximation hurts hit quality on Zipfian; the linear shift it replaced was not the latency bottleneck at these bucket sizes. Closed in cycle 0002.\n- **Bucket count reduction consistently hurts.** Fewer buckets raise collision density beyond what increased depth can absorb. Replicates across cycles 0001 and 0006; no reformulation has helped.\n- **Decoupling second-chance scan from 3-swap promotion fails.** Exhaustively tested in cycle 0007 by reducing to one swap and zero swaps. Hot items age into the scan window alongside cold items when promotion is curtailed, stripping the scan of its discrimination power. The two mechanisms are coupled, not independent.\n\n## Open questions\n\n- **What caused the put p99 regression in cycle 0006 v003?** Swap-elimination experiments in cycle 0007 falsified promotion write-pressure as the cause. The regression source is undiagnosed and must be identified before the cycle 0006 v003 configuration can be declared a clean winner.\n- **Is there a write-free mechanism to maintain hot/cold separation?** Positional promotion (3-swap) is currently the only confirmed way to keep hot items out of the eviction scan window. Whether a write-free alternative exists is unresolved.\n- **Can frequency information reach the eviction path without per-get writes?** Epoch-based or put-time counters that update only on eviction or insertion — not on every get — remain untested. Cycle 0005 identified this as the natural next frequency experiment.\n- **Does the 100 ns timer floor mask real latency differences below a meaningful threshold?** Cycle 0004 first surfaced this; cycle 0007 confirmed that eliminating all promotion swaps bought no measurable improvement. Distinguishing genuine parity from measurement noise is now a prerequisite for any further latency-targeted hypothesis.\n\n## Per-policy notes\n\n- **Positional LRU with bounded promotion**: The confirmed base configuration is: 32-byte aligned struct + position-0 guard + 3-swap unconditional loop + strict tail eviction. This combination won in cycle 0005 and serves as the stable foundation. Promotion budgets above three swaps regress; below three swaps sacrifice mid-bucket convergence.\n- **LFU / frequency-ordered**: Frequency ordering's advantage is eviction-candidate quality, not get-path speed. Every attempt to maintain per-get counters has introduced latency risk. The write-once binary \"ever-accessed\" flag is the only frequency encoding that avoids hot-path write cost and produces a measurable hit-rate lift (cycle 0006 v002 winner).\n- **Second-chance scan**: Mechanically coupled to the 3-swap promotion loop — the scan only discriminates hot from cold when promotion keeps genuinely hot items out of the scan window. Scan window size (8-slot) and flag placement (first-touch vs. position-0 arrival) are both confirmed as neutral on this workload.\n- **Zone-based / multi-tail policies**: Two-zone partitioning and two-tail frequency comparison have both been tested across multiple cohorts and uniformly failed. These policy families should not be revisited without a fundamentally different structural motivation.\n"
}
```

### 17:46:13 chat

Written to `C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_16-31-57-327\cycles\0007\cross-cycle\cross-cycle-insights.md`.

**Summary of what the file captures across all 7 cycles:**

**Recurring wins** — The four durable building blocks: 32-byte aligned struct (required invariant), 3-swap bounded promotion (confirmed optimal ceiling), position-0 guard (reliable p99 cut), and write-once second-chance eviction flag (strongest hit-rate lever found). Strict tail eviction without frequency signals at the eviction site is also a cross-cycle positive.

**Recurring failures** — Six closed avenues: per-get counter writes (consistent latency risk), two-tail/frequency-gated eviction (degrades hit rate), two-zone partitioning, swap-to-back promotion, bucket count reduction, and — the cycle 0007 conclusion — decoupling the second-chance scan from the 3-swap loop.

**Open questions** — Four unresolved threads: the mystery put p99 regression from cycle 0006 v003 (source still undiagnosed after cycle 0007 ruled out swap write-pressure), write-free hot/cold separation, frequency signals via non-get-path writes (epoch/put-time), and whether the 100 ns timer floor is masking real latency differences.

**Per-policy notes** — Groups findings by LRU/positional, LFU/frequency-ordered, second-chance scan, and zone-based families so future hypothesis writers can quickly locate what has and hasn't worked within each policy class.


---

- finished: 2026-07-21T17:46:14+00:00
- duration: 74.34s
- status: ok
- input_tokens: 7
- output_tokens: 3253
- cache_read_tokens: 114745
- cache_write_tokens: 10195
- cost_usd: 0.126705
