# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:42:38+00:00

## timeline

### 15:42:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving rewrites that remove a specific O(n) cost (scan consolidation, hash-index membership, O(1) ring-buffer ghost) are the only changes with a credible mechanism for their gains. Hit rate stays identical and `put`/`get` cost drops.\n- Targeting the actual cost source works. Ghost maintenance was the `put` tail driver. Main-queue eviction and membership checks were not.\n- A smaller small queue (10% then 5%) gave small hit-rate gains twice, from different parents. The curve is flattening.\n- Keeping the small queue as a strict one-hit-wonder filter preserves hit rate.\n\n## Recurring failures\n\n- Loosening main admission has hurt in every cycle. Examples: lower promotion threshold, larger small queue, freq head start for main-evicted keys, longer ghost history, and recycling main evictions into the shared ghost.\n- Ghost-size and ghost-policy tuning has repeatedly failed. The baseline looks near a sweet spot, and the early separate-ghost gain is doubtful.\n- Rotation caps and CLOCK-style rewrites of main eviction gave nothing, because the freq counter already bounds rotation.\n- Latency-only wins with no mechanism keep getting selected: one-constant edits with large drops, and `get` shifts in variants that never touched `get`. Treat these as noise.\n- Infrastructure failures (Maven 429) can hide ideas without testing them.\n- Refactors have deleted tests, and per-entry `metadata-size` hides the growth in whole-cache footprint from indexes and rings.\n\n## Open questions\n\n- Do the 5% ratio and the other small hit-rate gains survive repeated runs, other traces, and other cache sizes? All evidence comes from one trace at one size.\n- Is the `get` p50 shift real, or a layout effect? Does the combined index + ring + scan stack regress `get`?\n- What is left in the `put` tail now that ghost trimming is O(1)? Candidates are hash-map operations and the small-to-main move.\n- What is the real memory cost of the index and ring?\n- Is any special handling of main-evicted keys worthwhile? It would need an O(1) tagged-ghost design with evidence of faster reuse.\n- Can the deleted tests be restored without losing gains?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy so far):\n  - Current base stack: scan consolidation, hash-index membership, O(1) ring ghost, and a 5% small queue (pending confirmation).\n  - The promotion threshold stays at baseline. The main-eviction logic is settled and should be left alone.\n  - Next gains should come from the remaining `put` tail outside ghost trimming.\n  - Judge on hit rate over multiple traces, plus latency changes with a clear mechanism.\n"
}
```

### 15:42:51 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four required sections in the fixed order. I didn't re-read the file after writing it.

The main points it records, all from the five-cycle ledger:
- **Recurring wins:** Behavior-preserving rewrites that remove a specific O(n) cost are the only changes with a clear mechanism. Examples are the scan consolidation, the hash index, and the ring-buffer ghost. A smaller small queue (10%, then 5%) gave small hit-rate gains twice.
- **Recurring failures:** Anything that loosens main admission has hurt hit rate every time. Ghost tuning and main-eviction rewrites have also not paid off. Latency-only wins with no mechanism keep getting picked, and I treat those as noise.
- **Open questions:** All evidence comes from one trace at one cache size, so the 5% gains and the ledger's other small hit-rate gains are unconfirmed. The file also lists the unexplained `get` p50 shift, the remaining `put` tail, the index and ring memory cost, and the deleted tests.
- **Per-policy notes:** S3-FIFO is the only policy so far, so this section records its current base stack and what to leave alone.


---

- finished: 2026-10-20T15:42:51+00:00
- duration: 13.82s
- status: ok
- input_tokens: 4
- output_tokens: 1490
- cache_read_tokens: 37589
- cache_write_tokens: 9220
- cost_usd: 0.056078
