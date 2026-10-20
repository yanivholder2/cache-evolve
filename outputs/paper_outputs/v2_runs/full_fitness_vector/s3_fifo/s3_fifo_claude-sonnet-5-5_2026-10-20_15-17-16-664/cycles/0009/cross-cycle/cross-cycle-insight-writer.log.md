# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T16:30:59+00:00

## timeline

### 16:31:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving rewrites that remove a concrete cost with a clear mechanism are the only reliable wins. Examples: removing linear scans, a hash index for membership, an O(1) ring ghost, a tagged single index, a split ghost map, and a keyed single lookup. Hit rate stays identical.\n- Hash-map traffic was the real `put` bottleneck. Cutting the number of map probes per op moved the tail. Ghost maintenance mattered before that. Queue manipulation and main-eviction rewrites did not.\n- Lowering the small-queue ratio (10% → 5% → 3%) gave small hit-rate gains in the same direction each time. The gains shrink with each step, so the lever is saturating.\n- Tail-latency gains appear when an O(n) step is removed. Medians barely move.\n\n## Recurring failures\n\n- Anything that loosens main admission lowers hit rate on this trace. This includes a lower promotion threshold, a larger small queue, a freq head start for ghost re-admits, a larger ghost, recycling main evictions into the shared ghost, and a lower frequency cap. The small queue's one-hit-wonder filter matters most.\n- Eviction-policy tweaks give almost nothing: the rotation cap, the recent-main second chance, and adaptive small sizing. Their signals rarely fire here.\n- Micro-optimizations (frequency store, deque pre-sizing, hasher swaps, finalizers) are exhausted. They trade median against tail with no free win.\n- Selection through noisy latency keeps promoting Winners that have no mechanism. Latency shifts on paths a variant never touched are noise. Such Winners (several in cycles 1–9) should not be credited.\n- Bundling changes (pre-size plus finalizer, for example) keeps hiding which part caused an effect.\n\n## Open questions\n\n- Do the gains hold on other traces and cache sizes? Every result comes from one trace (workload-a) at one size.\n- Do latency gains survive repeated runs? The noise floor has never been measured.\n- What is the real heap footprint of the index, split ghost map and ring? The `metadata-size` metric shows only inline struct bytes.\n- Is the keyed single lookup (Cycle 9) compatible with the tagged index, split ghost map and Fx hasher, or does it duplicate them?\n- Does 3% (or 5%) small ratio hurt workloads with more mid-reuse objects?\n- Can a promotion or ghost-policy idea raise hit rate without loosening main admission? One option is an O(1) tagged variant of the existing ghost.\n- What is left in the `put` tail after the tagged index?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far):\n  - Current lineage: small ratio 3–5%, baseline promotion threshold, frequency cap 3, ring-buffer ghost, tagged index with split ghost map, Fx-style hasher with moderate sizing, keyed single lookup.\n  - Baseline promotion threshold, ghost size and frequency cap look near-optimal on this trace.\n  - Further parameter sweeps (small ratio, hasher, cap, adaptive sizing) are saturated or harmful. Stop them.\n  - Next work: promotion or ghost-policy ideas that keep main admission strict. Validate on multiple traces with repeated runs and an explicit heap measurement. Judge on hit rate and on latency changes that have a mechanism.\n"
}
```

### 16:31:19 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 cross-cycle output directory, with the four fixed sections in order. I built it only from the nine-cycle ledger digest in the prompt. I didn't read the previous insights file or any code.

- **Recurring wins:**
  - Behavior-preserving rewrites with a clear mechanism win reliably. The main cases are removing linear scans, the hash index, the ring ghost, the tagged index and the keyed single lookup.
  - Hash-map traffic was the real `put` bottleneck.
  - Lowering the small-queue ratio gave small hit-rate gains, with diminishing returns.
- **Recurring failures:**
  - Anything that loosens main admission lowers hit rate.
  - Eviction-policy tweaks give almost nothing, and micro-optimizations are exhausted.
  - Latency noise keeps promoting Winners that have no mechanism behind their gain.
  - Bundled changes hide which part caused an effect.
- **Open questions:**
  - All evidence comes from one trace at one cache size.
  - Latency gains have not been checked with repeated runs.
  - The real heap footprint is unmeasured.
  - It is unclear whether the keyed single lookup is compatible with the tagged index and Fx hasher.
  - It is unknown whether a hit-rate idea exists that keeps main admission strict.
- **Per-policy notes:** S3-FIFO is the only policy so far. The notes give its current lineage, say to stop parameter sweeps, and point the next work at promotion or ghost-policy ideas that keep main admission strict.

I kept the file qualitative and used no numbers beyond the small-ratio values (10%, 5%, 3%) that appear in the ledger.


---

- finished: 2026-10-20T16:31:20+00:00
- duration: 20.89s
- status: ok
- input_tokens: 4
- output_tokens: 1878
- cache_read_tokens: 40951
- cache_write_tokens: 12786
- cost_usd: 0.074895
