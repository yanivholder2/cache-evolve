# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T16:35:19+00:00

## timeline

### 16:35:32 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that remove redundant work with a clear mechanism keep paying off, with hit rate unchanged. Examples: scan consolidation, hash-index membership, O(1) ring ghost, tagged single index, split ghost map, keyed single lookup.\n- The biggest `put` gain came from cutting hash-map probes per `put` (Cycle 6). Ghost maintenance was the earlier lever (Cycle 5).\n- A cheap hasher helped `get` on the tagged index (Cycles 6–7).\n- Shrinking the small queue from 10% to 5% to 3% gave small hit-rate gains, with diminishing returns. It is the only hit-rate lever that has worked.\n- Gains with a mechanism show up in the tails (`put`/`get` p99). Medians are dominated by noise.\n\n## Recurring failures\n\n- Anything that loosens main admission lowered hit rate or gave nothing. This covers promotion at freq>=1, a larger small queue, ghost recycling, ghost head starts, a larger ghost, a recent-main second chance, freq reset and ghost age-gating. The small-queue filter matters most.\n- Parameter and policy micro-tweaks are saturated on this trace: ratio sweeps, frequency cap, adaptive sizing, and the rotation cap.\n- Adding per-entry state or counters (sequence numbers, carried hashes, adaptive counters) costs more latency than it returns.\n- Hasher, finalizer and pre-size work is exhausted. It trades `get` against `put` tail with no free win, and hashing small integer keys is not the bottleneck.\n- Reworking main-queue eviction and micro-optimizing the frequency store or deque showed no gain.\n- Since Cycle 8, Winners selected without a mechanism are likely noise-assisted. Latency drops on one-constant edits are noise.\n- Per-entry `metadata-size` hides whole-cache heap growth from the index and maps.\n\n## Open questions\n\n- What is the repeated-run noise floor for latency and hit rate? Which earlier gains survive it?\n- Do the 3–5% small ratio and the structural gains hold on other traces and cache sizes?\n- What is the real heap footprint of the tagged index, split ghost map and keyed lookup?\n- Do the Cycle 9 keyed lookup and the Cycle 6–7 tagged index and split ghost map conflict, and can they be merged cleanly?\n- What remains in the `put` tail (eviction, hash-map operations, small-to-main move)?\n- Is this trace near its ceiling for this policy family? Would a different workload shape expose gains from promotion or ghost policy?\n- Is special handling of main-evicted keys worthwhile? It would need an O(1) tagged design with evidence of faster reuse.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far): the current base is a small queue of about 3–5%, a tagged index, a separate ghost map, a ring-buffer ghost and a cheap hasher.\n- Hit rate has been nearly flat since the small-ratio gains. Every later gain is latency only.\n- The ghost works as a one-hit-wonder filter. Sharing it with main evictions dilutes it, and a head start or larger size hurts.\n- Main frequency cap 3 protects reused entries, and lowering it costs hit rate.\n- All evidence is from one trace (workload-a) at one size. Future cycles should test several traces, run repeats, measure heap, and use only hypotheses with a clear mechanism.\n"
}
```

### 16:35:37 chat

I wrote `cross-cycle-insights.md` in the output dir. It has the four fixed sections, in order, projected from the ledger digest alone.

- **Recurring wins:**
  - Changes that remove redundant work with a clear mechanism keep paying off, with hit rate unchanged.
  - Cutting hash-map probes per `put` was the biggest `put` gain.
  - Shrinking the small queue from 10% to 3% is the only hit-rate lever that has worked.
- **Recurring failures:**
  - Anything that loosens main admission hurt or did nothing.
  - Parameter tweaks and added per-entry state are saturated or costly.
  - Hasher work is exhausted.
  - Winners picked without a mechanism since Cycle 8 are likely noise.
- **Open questions:**
  - What the repeated-run noise floor is, and whether other traces and cache sizes confirm the gains.
  - What the real heap footprint is.
  - Whether the Cycle 9 keyed lookup conflicts with the earlier tagged index and split ghost map.
- **Per-policy notes:** S3-FIFO is the only policy so far. The notes cover the current base, the flat hit rate since the small-ratio gains, the ghost's filtering role, and the single-trace limit.

I used no numbers beyond the small-queue percentages (10%, 5%, 3%) and the main frequency cap of 3 that appear in the ledger.


---

- finished: 2026-10-20T16:35:37+00:00
- duration: 18.28s
- status: ok
- input_tokens: 4
- output_tokens: 1791
- cache_read_tokens: 41721
- cache_write_tokens: 13580
- cost_usd: 0.077355
