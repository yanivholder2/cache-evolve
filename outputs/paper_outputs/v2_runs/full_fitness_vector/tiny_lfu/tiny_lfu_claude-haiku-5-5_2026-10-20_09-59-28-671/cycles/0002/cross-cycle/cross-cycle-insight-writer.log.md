# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T10:12:42+00:00

## timeline

### 10:12:49 thinking



### 10:12:54 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_09-59-28-671/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Constant-factor work that leaves eviction order untouched is the most reliable source of gains. Both the newest-key relink guard (cycle 1) and the single-hash reuse on the put path (cycle 2) cut per-op time without a meaningful hit-rate change.\n- Metadata trimming can be done independently of policy. Packing the protected state into existing node fields (cycle 2) shrank bookkeeping without moving hit rate, which makes it a safe base for any segmentation work.\n\n## Recurring failures\n\n- Recency-based handling of rejected admission victims hurt both hit rate and per-op time (cycle 1). The frequency-based admission gate already does the protective work, so extra recency rotation adds cost without benefit.\n- Extra state in front of the admission sketch has not paid off. The doorkeeper (cycle 2) barely moved hit rate while adding handle state and per-put work, so one-hit pollution does not look like the limiting factor on the tested trace.\n- Added bookkeeping has to earn its keep. Segmentation's small hit-rate gain (cycle 1) came with per-node and per-cache metadata growth, and the latency drop it showed is not explained by that bookkeeping.\n- Changes that look behaviour-preserving may not be. Hoisting the hash out of the put path (cycle 2) moved hit rate, so the speedup cannot be counted as a pure cost change until equivalence is confirmed.\n\n## Open questions\n\n- Does any hit-rate gain hold beyond lirs workload-a? Every result so far comes from one trace, so no gain has been shown to generalise.\n- Is segmentation's latency drop real or measurement variance? Its bookkeeping would predict a slowdown, not a speedup.\n- Do the precomputed-hash helpers produce the same counter indices as the public sketch methods? The answer decides whether the hash-reuse hit-rate drop is a bug or a real trade-off.\n- How much of the newest-key relink saving depends on how often accesses hit the newest key? A trace with a different share of such accesses would show whether the gain is structural.\n- Is the doorkeeper's memory cost fully accounted for? Its storage does not appear in the metadata breakdown.\n- Does the doorkeeper show any effect on a trace where one-hit keys dominate?\n\n## Per-policy notes\n\n- TinyLFU admission (frequency sketch, admission gate, doorkeeper): the gate appears to do the protective work already. Hash-reuse on the admission and sketch-update path is worth pursuing, but only after equivalence is verified. Front-end filters such as the doorkeeper have not helped on the tested trace.\n- Segmented resident list (probation/protected split): the only policy change that moved hit rate, but the movement is small and unconfirmed on a second trace. Its metadata cost should be reduced via field packing before further builds on it.\n- Victim and recency handling (MRU-tail rotation of rejected victims, newest-key relink skipping): rotation of rejected victims is a dead direction. Relink guards that preserve eviction order are a productive direction for speed.\n"
}
```

### 10:12:56 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output dir with the four required sections in order. It is generated only from the two-cycle ledger digest in the prompt; I did not read any previous version of the file or any implementation code.

The main signals across the two cycles:

- **Recurring wins:** Constant-factor changes that keep eviction order intact have produced speedups in both cycles. Packing the protected state into existing node fields cut metadata without moving hit rate.
- **Recurring failures:** Recency rotation of rejected victims hurt hit rate and per-op time. The doorkeeper barely moved hit rate and added state. Hoisting the hash out of the put path moved hit rate, so it is not behaviour-preserving as written.
- **Open questions:** Every result comes from one trace (lirs workload-a), so no gain has been shown to generalise. Segmentation's latency drop is unexplained, and it is unconfirmed whether the precomputed-hash helpers match the public sketch methods.
- **Per-policy notes:** Grouped under TinyLFU admission, the segmented resident list, and victim/recency handling.


---

- finished: 2026-10-20T10:12:57+00:00
- duration: 14.33s
- status: ok
- input_tokens: 4
- output_tokens: 2490
- cache_read_tokens: 35020
- cache_write_tokens: 7683
- cost_usd: 0.003132
