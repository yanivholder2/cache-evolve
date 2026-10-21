# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:20:04+00:00

## timeline

### 09:20:15 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Shrinking the small (probation) queue to give the main queue more room has won twice in a row (baseline → 10% → 5%). The gains come from a larger main queue, not from a larger ghost queue. The second step gained less than the first, so the curve is close to flat.\n- Rewarding proven repeated reuse beats making entry easier. On workload-a, protecting main capacity for keys with demonstrated reuse is the most reliable way to gain hit-rate.\n- Single-change variants give clear attribution. Where one thing was changed (partition, ghost size), the result could be explained.\n\n## Recurring failures\n\n- Making admission or re-admission more lenient keeps losing hit-rate. This includes lowering the graduation threshold, giving ghost-readmitted keys a free frequency credit, and simply enlarging the ghost queue. One reuse signal is weak evidence on workload-a, and keys let in on it push out keys with proven reuse.\n- No cycle has measured per-op-time or metadata-size, even though both Continuations asked for them. Every ghost-growing winner adds to unmeasured linear-scan cost.\n- The port of the winning policy onto the behaviour-neutral indexed structure has been carried over twice and never done.\n- Changing two things at once (threshold plus frequency reset in cycle 0001) left a regression that can't be attributed.\n\n## Open questions\n\n- What per-op-time and metadata-size cost does the current winner carry under linear ghost scans? Does the indexed structure remove that cost?\n- Does making ghost re-admission stricter pay off? Examples: requiring repeat ghost hits, or inserting at the main tail. Would a stricter rule make a larger ghost worth having?\n- Does the partition gain hold on other traces and cache sizes, or is it specific to workload-a?\n- Does a lower graduation threshold alone, without the frequency reset, still regress?\n\n## Per-policy notes\n\n- S3-FIFO (the only policy explored so far, on workload-a only):\n  - Partition: smaller small-queue fractions help, with diminishing returns. The current baseline is 5%, and going below it is unlikely to pay off much.\n  - Ghost queue: it already acts as a safety net for keys that miss graduation, so changes to the probation window or promotion timing matter less than expected. Changes that make the ghost more generous hurt; only changes that make it more selective are still unexplored.\n  - Graduation: earlier promotion hurt. Only the threshold change on its own is still untested.\n  - Data structures: the indexed-slab variant kept hit-rate unchanged, but its latency and memory effects are still unmeasured.\n"
}
```

### 09:20:21 chat

I've written `cross-cycle-insights.md` from the ledger digest for cycles 0001 and 0002, using the four required sections in order. It covers one policy (S3-FIFO) on one workload (workload-a), so every pattern below rests on just two cycles.

- **Recurring wins:** Shrinking the small queue won both cycles (10%, then 5%), and the second gain was smaller than the first. The gain comes from a bigger main queue, not a bigger ghost queue. Keys that have shown repeated reuse are worth more than keys that were let in easily.
- **Recurring failures:** Every change that made entry or re-admission easier lost a little hit-rate. That includes the lower graduation threshold, the free frequency credit for ghost-readmitted keys, and the larger ghost queue. Two requests have also been carried over twice and never done: measuring per-op-time and metadata-size, and porting the winning policy onto the indexed structure. Changing two things at once in cycle 0001 left one regression with no clear cause.
- **Open questions:**
  - What the larger ghost queue costs in speed and memory while the linear scans are still in place.
  - Whether a stricter ghost re-admission rule helps.
  - Whether the partition gains hold on other traces and cache sizes.
  - Whether a lower graduation threshold still loses hit-rate without the frequency reset.
- **Per-policy notes:** These cover S3-FIFO's partition, ghost queue, graduation rule and data structures separately.

I didn't read the previous insights file, and I referred to all results in qualitative terms without numbers.

File is in the output dir:
- cross-cycle-insights.md


---

- finished: 2026-10-21T09:20:22+00:00
- duration: 17.76s
- status: ok
- input_tokens: 4
- output_tokens: 1558
- cache_read_tokens: 34443
- cache_write_tokens: 6785
- cost_usd: 0.092345
