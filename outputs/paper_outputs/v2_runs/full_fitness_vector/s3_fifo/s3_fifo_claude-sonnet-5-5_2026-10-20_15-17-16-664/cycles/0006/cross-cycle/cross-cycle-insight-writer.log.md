# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T15:46:39+00:00

## timeline

### 15:46:53 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_15-17-16-664/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Behavior-preserving rewrites that remove a mechanism-identified cost keep winning: hit rate stays identical and the targeted latency drops. Examples: scan consolidation, hash-index membership, O(1) ring-buffer ghost, single location-tagged index.\n- The `put` tail has been peeled in layers. Linear scans came first, then ghost maintenance, then multiple hash-map probes per `put`. Each fix moved `put` only when it hit the real cost.\n- Only changes with a clear mechanism are credited. Latency shifts on paths a change never touched are treated as noise.\n- A small queue of about 5% has given modest hit-rate gains twice, from clean baselines, and the 10% change before it also helped a little. The direction is consistent but small, and every result comes from one trace.\n\n## Recurring failures\n\n- Anything that loosens main admission or gives main-evicted keys special treatment has lowered hit rate. This includes recycling into the shared ghost, a freq head start on re-entry, a larger ghost bound, a looser promotion threshold, and a larger small queue. The small queue's one-hit-wonder filter matters more than quick promotion.\n- Latency-only \"wins\" with no mechanism keep being selected: constant tweaks, the rotation cap, and the head-start ghost. Latency is not a trustworthy selection signal on its own.\n- Micro-optimizations are exhausted: frequency store, small-to-main move, deque pre-sizing, a CLOCK-style rewrite of main eviction, and a cheaper hasher on its own. They land within noise and do not touch the `put` tail.\n- Per-entry `metadata-size` hides real footprint growth. The index, the ring and the tagged index each grew whole-cache memory.\n\n## Open questions\n\n- Do the gains hold across other traces, cache sizes and repeated runs? All evidence is workload-a at one size.\n- Can the tagged index's footprint and `get` p99 be reduced (ghost keys kept out of the main index, better table sizing)? Does the cheap hasher stack on it?\n- What is left in the `put` tail after the tagged index?\n- Is there a hit-rate idea that does not loosen main admission? Hit-rate work has barely moved since the small-ratio tweaks. An O(1) tagged variant of the ghost for main-evicted keys is the only untried form.\n- Can the tests deleted during the scan refactor be restored without losing its gains?\n- Is the 5% small ratio real, or noise?\n\n## Per-policy notes\n\n- S3-FIFO, small queue: 10% beat the baseline. 15% regressed, and 5% gave small gains. The curve is flattening, so stop sweeping it unless other traces disagree. The baseline promotion threshold looks like a local optimum.\n- S3-FIFO, ghost: A shared ghost that main evictions also enter dilutes the filter. A separate ghost gave an unreliable small gain, and its variants later regressed. The ghost bound looks near its sweet spot. The ring-buffer ghost made maintenance O(1).\n- S3-FIFO, main eviction: Rewrites such as the CLOCK-style pop/push and the rotation cap showed nothing to gain. The frequency counter already saturates low.\n- Index and latency base: The current base is the tagged index, with the ring ghost and scan consolidation. Build further `put` and `get` work on it. Judge on hit rate and on mechanism-backed latency, with repeated runs.\n"
}
```

### 15:46:59 chat

I wrote `cross-cycle-insights.md` in the output dir with the four fixed sections, built from the six cycle digests alone. I didn't read the previous version or any code.

- **Recurring wins:**
  - Rewrites that keep hit rate identical and remove a specific cost keep winning on latency. That covers the scan consolidation, the hash index, the ring-buffer ghost and the tagged index.
  - The `put` tail has been peeled in layers.
  - The 5% small queue has given modest hit-rate gains, but only on one trace.
- **Recurring failures:**
  - Anything that loosens main admission has lowered hit rate.
  - Latency-only wins with no mechanism keep getting selected.
  - Micro-optimizations land within noise.
  - Per-entry `metadata-size` hides real memory growth.
- **Open questions:**
  - Do the gains hold on other traces, cache sizes and repeated runs?
  - Can the tagged index's footprint and `get` p99 come down?
  - Is there a hit-rate idea that doesn't loosen main admission?
  - Can the tests deleted in the scan refactor be restored?
- **Per-policy notes:** all four are S3-FIFO. They cover the small-queue ratio, the ghost, main eviction, and the index base to build on.

I kept it qualitative and didn't invent any numbers.


---

- finished: 2026-10-20T15:46:59+00:00
- duration: 20.00s
- status: ok
- input_tokens: 4
- output_tokens: 1748
- cache_read_tokens: 38420
- cache_write_tokens: 10260
- cost_usd: 0.062985
