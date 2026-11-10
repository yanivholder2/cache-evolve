# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-11-10T09:54:02+00:00

## timeline

### 09:54:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Strict admission to main holds up. The baseline two-reuse graduation rule plus freq reset has survived four cycles. Every attempt to loosen it (lower threshold, occupancy gate, ghost-rate switch) either fell back to it or lost.\n- Per-key evidence beats global signals. The first ghost-path change to beat its parent (cycle 4) gated on each key's own small-queue freq at ghost time. Global rate proxies never won.\n- Filtering inside small is cheaper than filtering at main admission. Cycle 4's freq-gated ghost path and its freq-1 small-tail relap both pointed this way.\n- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) keep hit-rate bit-identical and port cleanly onto policy winners. Oracle tests enforce this.\n- Cycle winners have often been the variant that didn't regress, not one that clearly improved. Policy gains stay small.\n\n## Recurring failures\n\n- Tuning a single constant (threshold or static small/main split): hm_0 and web_0 move in opposite directions, and split effects are near noise and flip sign across trace and size.\n- Main occupancy as an adaptive signal: main fills and stays full, so the gate is dead in steady state.\n- Ghost-derived global signals (windowed ghost-return rate, unweighted ARC-style ghost-hit split): the ghost is several times larger than small and mostly holds weak single-reuse keys, so the signal is inflated and fires on the wrong traces.\n- Extra protection inside a full main (second-chance credit for ghost returnees): it keeps dead keys for an extra lap and evicts entries that would have been hit.\n- Per-op-time and metadata-size are still absent from the fitness vector after four cycles. Two cost variants are waiting on measurement, and hypotheses keep assuming these dimensions are reported when they aren't.\n\n## Open questions\n\n- Do the freq-gated ghost path and the freq-1 small-tail relap stack, or do they compete for small slots?\n- Can freq-0 ghost returnees recover hm_0 without giving back web_0? Options: route them by main tail freq, or give them a stronger initial credit in small.\n- Should the second reuse count only within a recency window? Widely spaced reuses look like weak evidence.\n- Would directly comparing the misses a promotion avoids against the main hits it displaces beat every proxy tried so far?\n- How large are the actual per-op-time and metadata-size deltas of the O(1) index and the hasher/layout changes? Is the extra ghost-scan cost real?\n- Has the policy essentially plateaued on these traces, so that remaining effort should go to cost and metadata?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy evolved so far): the current lineage is the O(1) index, the occupancy-gated two-reuse rule (in practice the baseline rule) and the freq-gated ghost→main path. The pending cost changes from cycle 4 are still to be ported onto it.\n- Policy effects show up almost only at the small cache size. At the large size the ghost covers a long reuse window and admission is cheap, so rerouting between ghost, small and main barely changes evictions.\n- Trace character is consistent: web_0 penalises any loosening of main admission (exactly-twice keys pollute main) and gets little value from ghost returns. hm_0 sometimes has genuine ghost-return reuse and gained from looser admission only at the small size or cold start.\n"
}
```

### 09:54:23 thinking



### 09:54:23 chat

I've written `cross-cycle-insights.md` from the ledger for cycles 0001–0004, using the four sections in the required order.

**Recurring wins**
- The strict two-reuse rule for promoting keys into main, plus the freq reset, has held for four cycles. Every attempt to loosen it either fell back to it or lost.
- Gating on each key's own freq when it went to ghost gave the first ghost-path gain (cycle 4). No global rate signal has ever won.
- Filtering inside the small queue costs less than filtering at the point a key enters main.
- The cost-only refactors kept hit-rate exactly the same.

**Recurring failures**
- Changing one constant at a time (threshold or static split) moves hm_0 and web_0 in opposite directions.
- Main occupancy is useless as a signal because main stays full.
- Ghost-rate signals are inflated by how large the ghost is, so they fire on the wrong traces.
- Giving keys extra protection inside a full main pushes out entries that would have been hit.
- Per-op-time and metadata-size have been missing from the fitness vector in all four cycles. Two cost variants are still waiting to be measured.

**Open questions**
- Whether the freq-gated ghost path and the small-tail relap add up or compete for the same small-queue slots.
- How to route freq-0 ghost returns so hm_0 gains without losing web_0.
- Whether a key's second reuse should only count if it comes soon after the first.
- Whether directly comparing what a promotion gains against what it displaces would beat the proxies tried so far.
- How big the cost deltas actually are.
- Whether the policy has plateaued on these traces.

**Per-policy notes:** S3-FIFO is the only policy so far. Its policy gains show up almost only at the small cache size. web_0 consistently punishes looser admission into main. hm_0 sometimes gets real value from keys returning from ghost.


---

- finished: 2026-11-10T09:54:24+00:00
- duration: 21.64s
- status: ok
- input_tokens: 4
- output_tokens: 1951
- cache_read_tokens: 37941
- cache_write_tokens: 9286
- cost_usd: 0.121422
