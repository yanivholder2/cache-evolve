# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:40:42+00:00

## timeline

### 14:40:48 thinking



### 14:40:53 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving cost refactors (resident-key indexing, hasher or fingerprint swaps, folded membership bits, early returns) have repeatedly held hit-rate flat, as intended. They are safe to land, but none has yet shown a measured cost gain.\n- The only hit-rate movement in the ledger came from a ghost-side change: feeding main-queue evictions at zero frequency into the ghost queue. It is marginal and rests on one trace at one capacity.\n\n## Recurring failures\n\n- Relaxing the small-queue promotion bar lost hits, and tightening it to three hits was neutral. The promotion bar looks saturated on this trace, and timing-only changes to warm-key promotion did not alter residency.\n- Ghost re-admission has been pushed several ways and has not paid off as a standalone lever: widening the ghost window lost hits, routing ghost hits through the small queue lost hits, and giving re-admissions an extra main-queue pass or frequency of 1 did not help. Re-admitted keys appear to displace resident entries.\n- The frequency axis is inert here. Changes to the main-queue frequency cap and decay step were flat across cycles.\n- Sizing levers have not helped: an adaptive small-queue size, driven by ghost hits, lost hit-rate, and the ghost signal looks too noisy to steer the split.\n- Retaining warm main-queue residents longer, via a tail second pass instead of head reinsert, lost hits. The existing reinsert position is not leaving value on the table.\n\n## Open questions\n\n- Does the main-eviction ghost feed hold on other traces and at other capacities, and does its gain survive the unchanged ghost scans?\n- Do the cost-side refactors (resident index, hasher change, bit folding, early return, ghost mirror removal, ghost pre-filter) reduce per-op-time or metadata-size? No cycle has reported either dimension, so none can be ranked.\n- Is the ghost linear scan the dominant miss-path cost? Any cost gain from indexing is bounded until it is addressed.\n- Were the intended overlay edits actually present in the built code for each variant? This verification gap has persisted across most cycles, and several flat or negative results cannot yet be tied to their hypotheses.\n\n## Per-policy notes\n\n- S3-FIFO, small queue: promotion-bar and promotion-timing changes have been flat or negative. Routing re-admissions through this queue lost hits.\n- S3-FIFO, main queue: frequency cap, frequency decay, and reinsert-position changes have been flat or negative. Feeding main evictions into the ghost queue is the one change with a measured gain.\n- S3-FIFO, ghost queue: window size, admission routing, and re-admission frequency have not helped. Ghost-side changes that alter which keys enter the ghost queue, rather than how re-admissions are handled, are the only ones that have moved hit-rate.\n- S3-FIFO, resident and ghost indexes: all changes here are cost-side and have held hit-rate flat. Their per-op-time and metadata claims remain unmeasured.\n- Evidence across all policy groups comes from one trace (lirs workload-a) at one capacity (2048).\n"
}
```

### 14:40:55 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 cross-cycle output directory, with the four required sections in order. I built it only from the inlined ledger digest and did not open the previous file.

The main takeaways:

- **Recurring wins:** Behavior-preserving cost refactors have consistently held hit-rate flat, but none has shown a measured cost gain. The only hit-rate movement is the main-eviction ghost feed from cycle 0009, which is marginal and rests on one trace.
- **Recurring failures:** Promotion-bar changes, ghost re-admission routing and window changes, frequency cap and decay tweaks, adaptive sizing, and tail reinsert all lost hits or stayed flat.
- **Open questions:** Whether the ghost feed holds on other traces and capacities, whether any cost-side change reduces per-op-time or metadata-size, and whether the intended overlay edits actually landed in the built code.
- **Per-policy notes:** Grouped under S3-FIFO's small queue, main queue, ghost queue, and resident/ghost indexes. Only one policy appears in the ledger.

Every observation rests on one trace (lirs workload-a) at one capacity (2048), and the file says so.


---

- finished: 2026-10-20T14:40:56+00:00
- duration: 13.95s
- status: ok
- input_tokens: 4
- output_tokens: 2437
- cache_read_tokens: 41105
- cache_write_tokens: 13677
- cost_usd: 0.004365
