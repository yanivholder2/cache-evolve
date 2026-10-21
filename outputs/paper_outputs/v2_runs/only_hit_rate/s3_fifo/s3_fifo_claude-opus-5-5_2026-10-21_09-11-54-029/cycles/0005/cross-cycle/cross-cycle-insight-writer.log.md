# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0005/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:34:24+00:00

## timeline

### 09:34:39 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0005/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Workload-a consistently rewards protecting keys with proven reuse. Every win across five cycles either gave main more capacity or more retention, or widened the ghost's evidence without making entry easier.\n- Partition shrinking paid off twice (default→10% in cycle 0001, →5% in cycle 0002), with diminishing returns. The gain comes from a bigger main queue, not from a bigger ghost.\n- Main-queue retention through the frequency cap gave a real gain from 3 to 7 (cycle 0003), then flattened (7→15 in cycle 0004 was within noise).\n- Ghosting main-queue eviction victims (cycle 0005) gave a small gain by widening the reuse evidence while leaving re-admission at front-of-main with freq 0.\n- Behaviour-neutral structural ports (indexed slab, index and lazy ghost) were twice verified as exact against the linear reference (cycles 0003 and 0004). Policy and representation can be changed independently.\n- Single-lever variants gave clean attributions. The one bundled change (cycle 0001 v002) could not be attributed.\n\n## Recurring failures\n\n- Making ghost re-admission more generous lost every time: a freq-1 credit (cycle 0002) and a doubled or decoupled ghost (cycle 0002). One ghost hit is enough evidence of reuse, and extra credit takes main capacity away from keys with proven reuse.\n- Making ghost re-admission stricter also lost every time: tail insertion (cycle 0003) and small-queue probation with a banked credit (cycle 0004). The small queue's window is far shorter than the ghost's reuse horizon, so probation rejects the keys the ghost correctly identified. Front-of-main at freq 0 is settled.\n- Changing graduation from small to main was negative or flat. Earlier graduation with a frequency reset regressed (cycle 0001), and a 1-hit threshold alone was flat (cycle 0005), because the ghost already recovers keys hit once in small.\n- Raising the frequency cap above 7 and aging credit through decay or halving (cycles 0004 and 0005) were flat or negative. Linear decrement at cap 7 looks near-optimal, and faster aging hurts keys with moderate reuse.\n- Evaluation coverage is the most persistent process failure. Per-op-time and metadata-size have gone unreported for all five cycles even though every Continuation asked for them. Only one trace at one cache size has ever been tested.\n\n## Open questions\n\n- What are the per-op-time savings and metadata-size costs of the indexed structure (duplicated keys, stale ghost deque, compaction spikes)?\n- Does the cumulative extra work in the incumbent lineage carry a hidden per-op-time cost? That work comes from a larger ghost, longer sweeps under higher caps, and ghost inserts on main evictions.\n- Do the accumulated small wins (5% split, cap 7, ghosting main victims) generalize to other traces and cache sizes, or are they overfitted to workload-a?\n- Would a separate or split ghost budget for main victims grow the cycle 0005 gain, or would it only show how the shared budget dilutes small-queue rejects?\n- Is any hit-rate headroom left on workload-a at this size? Effects have been near noise since cycle 0004, which suggests the trace is close to saturated for this policy family.\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far): the current baseline is the indexed structure with a 5% small queue, main cap 7 with linear decrement, re-admission at front-of-main with freq 0, and main victims recorded in the shared ghost.\n- S3-FIFO levers settled as flat or negative: re-admission credit, tail insertion and probation; ghost enlargement alone; graduation threshold; cap above 7; credit decay; shrinking the partition below 5%.\n- S3-FIFO levers still open: a ghost budget for main victims, plus all cost-side and generalization measurements.\n"
}
```

### 09:34:43 chat

I've written `cross-cycle-insights.md` from the five-cycle ledger. The main takeaway is that hit-rate on workload-a looks close to its limit for this policy, while per-op-time and metadata-size have never been measured in any cycle.

- **Recurring wins:** What kept working was protecting keys with proven reuse. That means a bigger main queue, a frequency cap of 7, and adding main-queue evictions to the ghost queue. The two structure ports kept hit-rate exactly unchanged.
- **Recurring failures:** Every change to how keys come back from the ghost queue lost, whether it made re-entry easier or harder. Changing how keys move from the small queue to main was flat or worse, as were raising the cap above 7 and decaying credit. The variants also kept reporting only hit-rate, despite every cycle asking for the other two measures.
- **Open questions:** The incumbent lineage keeps adding extra work, and its time and memory cost is unknown. Nobody has checked whether the small wins hold on other traces or cache sizes. A separate ghost budget for main-queue evictions is the one policy lever left untried.
- **Per-policy notes:** S3-FIFO is the only policy tried so far. The notes list the current baseline, the levers that are settled, and the ones still open.


---

- finished: 2026-10-21T09:34:44+00:00
- duration: 20.15s
- status: ok
- input_tokens: 4
- output_tokens: 1897
- cache_read_tokens: 37263
- cache_write_tokens: 10067
- cost_usd: 0.125945
