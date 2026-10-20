# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:34:44+00:00

## timeline

### 11:34:50 thinking



### 11:34:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Replacing stored ghost keys with fingerprints moved put-path cost in the same direction in two separate cycles (0003, 0004), with hit rate unchanged. It is the only edit with a repeated, attributable benefit so far.\n- Policy-preserving data-structure changes (hash membership index, sequence-tagged ghost) left hit rate exactly unchanged, which makes them safe to evaluate on cost alone.\n- Edits confined to the put path have been the ones with a mechanism that matches the measured effect; get-path changes have not.\n\n## Recurring failures\n\n- Hit-rate losses from changing how much the ghost or small queue admits: lowering the small-queue promotion threshold (0001) and enlarging the ghost queue toward full capacity (0004) both moved singly-reused keys into main without a matching reuse gain.\n- Stacking a lowest-frequency victim choice on the enlarged small queue (0003) lost hit rate, and the rotation budget it depends on likely runs out less often once the small queue is larger.\n- Sequence-tagged ghost bookkeeping (0002) added per-eviction put cost without any policy gain.\n- Moving the frequency counter into a parallel side array (0004) grew the container enough that total footprint got worse, and it added an indexed read on every hit.\n- Frequency aging on main-queue rotation (0003) showed no effect on the one trace reported; it has never been shown to change an eviction.\n- Several Winner labels rest on hypothesis text and fitness deltas rather than verified mechanisms, and on single traces, so they are weaker than the labels imply.\n\n## Open questions\n\n- Does the fingerprint ghost's put-latency gain hold across other traces and capacities, and does its linear ghost-hit scan or added container metadata show up in tail latency at larger sizes?\n- Is the get-path latency shift real? It has appeared in variants with no get-path edit (0001–0003) and in one variant with a plausible code-level cause (0004). A baseline rerun with no code change has not been done.\n- Is there a ghost capacity between main-sized and full capacity that keeps some reuse benefit without the hit-rate loss?\n- Does the small-queue share increase hold on other traces and capacities, or is the gain specific to the workload tested?\n- Where is the dominant put miss-path cost, if the ghost `position` scan is not it?\n- How often does the rotation budget actually run out under the evaluated workloads, and does the lowest-frequency victim choice help when it does?\n\n## Per-policy notes\n\n- Small queue: a larger share gave a small hit-rate gain on one trace (0002); promotion-threshold changes (`freq > 0`) cost hit rate and gave no attributable latency benefit (0001). Keep the share question open; drop threshold tuning.\n- Main queue: the second-chance rotation budget is a bounding change that trades forced evictions of hot entries for put cost (0001). Post-budget victim choice helped only in the one case tested and lost hit rate once stacked on the larger small queue (0002, 0003). Frequency aging is untested on a trace where the rotation pass runs long (0003).\n- Ghost queue: the fingerprint representation is the clearest put-path win (0003, 0004), but it adds container overhead and linear ghost-hit cost. Sequence tagging added put cost without policy gain (0002). Enlarging toward full capacity lowered hit rate (0004). Hash membership is policy-neutral (0001).\n- Per-entry metadata: moving the frequency counter to a side array did not reduce total footprint and touched the get path (0004). No variant has yet shown a footprint gain that holds at the structure level.\n"
}
```

### 11:34:58 chat

I wrote `cross-cycle-insights.md` to the output directory, regenerated from the four-cycle ledger digest. It has the four required sections in order, and I did not read the previous version.

The main signals:

- **Recurring win:** the fingerprint ghost lowered put-path cost in two separate cycles (0003 and 0004) with hit rate unchanged. It is the only edit with a repeated, attributable benefit so far.
- **Recurring failures:** lowering the small-queue promotion threshold, enlarging the ghost queue toward full capacity, and stacking the victim choice on the larger small queue all lost hit rate. The side-array counter did not reduce total footprint and added a per-hit read. Frequency aging has not been shown to change any eviction.
- **Open questions:** whether the get-path latency shift is real (it appears in variants with no get-path edit, and no baseline rerun has been done), whether the fingerprint gain holds on other traces and capacities, and where the dominant put miss-path cost actually is.

I flagged in the file that several Winner labels rest on hypothesis text and single traces, so they are weaker than the labels suggest.


---

- finished: 2026-10-20T11:34:58+00:00
- duration: 14.27s
- status: ok
- input_tokens: 4
- output_tokens: 2711
- cache_read_tokens: 36742
- cache_write_tokens: 9623
- cost_usd: 0.003648
