# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:17:00+00:00

## timeline

### 20:17:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that preserve frequency information and loosen admission only slightly (cycle 3 tie-admit duel) are the only admission change that helped. Behavior-preserving changes (index reuse, slab links, single-hash) match the parent exactly.\n- Wider sketch rows help slightly and monotonically (cycles 7, 8, 10), with sharply diminishing returns.\n- Count semantics and the admit threshold must be tuned together. Retuning both restored parity (cycles 7-8).\n\n## Recurring failures\n\n- Anything that discards or degrades frequency information loses hit-rate: aging/halving, larger window, doorkeeper, narrower rows (any size), 4-bit saturating counters, blocked/correlated rows.\n- Admission-rule tweaks beyond the tie-admit duel (looser gating, tie-break flags, ghost bonus, extra rows) are flat at noise level.\n- Segment-internal tuning (protected size, demotion choice, probation victim sampling) is flat.\n- Adaptive window tuning has no value on a single stable trace.\n- Winners are often near-ties or least-bad losers, chosen on hit-rate alone.\n- For ten cycles, fitness has reported only hit-rate. Metadata-size and per-op-time have never been measured, so no cost idea has been judged. Hypotheses are also muddled (direction of width scaling, parent) and authors cannot read overlay source.\n- The one rejection (cycle 1 aging) was environmental (HTTP 429 on a dependency download), not a flaw in the idea.\n\n## Open questions\n\n- Do the cost changes (flat u8 packing, single-hash, index reuse, slab links, compact sketches) actually shrink metadata or speed up ops? This is the main unverified thread.\n- Is workload-a saturated for hit-rate? A second trace (shifting popularity or different cache size) is needed, and might revive aging, doorkeeper, or adaptive-window ideas.\n- Does an intermediate width (such as 8) keep most of the wide-sketch gain at lower size?\n- Could a blocked sketch with decorrelated rows keep hit-rate and gain locality?\n- Is eviction or segment structure the only remaining hit-rate lever?\n\n## Per-policy notes\n\n- TinyLFU / W-TinyLFU (window + segmented main + sketch-gated admission): the baseline is near its optimum on workload-a. The sketch is mildly collision-limited and not over-narrowable; hash-sharing is safe. Keep the tie-admit duel and single-count with a lower threshold as the base.\n- Next steps: fix the evaluation first (add metadata-size and per-op-time, add a second trace). Keep cost and behavior changes in separate variants. Any width hypothesis must state its parent and scaling direction.\n"
}
```

### 20:17:15 chat

I wrote `cross-cycle-insights.md` in the output dir. It has the four required sections, in order, and is built only from the ledger digest.

The main points it records:

- **Recurring wins:** Only slight loosening of admission that keeps frequency counts has helped, and wider sketch rows help a little with diminishing returns. Behavior-preserving changes match their parent exactly.
- **Recurring failures:** Anything that discards or degrades frequency information loses hit-rate. Admission tweaks and segment-internal tuning are flat. Fitness has reported only hit-rate for ten cycles, so no cost change has been judged.
- **Open questions:** Whether the cost changes actually save size or time, whether workload-a is saturated, and whether a second trace would revive aging or the doorkeeper.
- **Per-policy notes:** The W-TinyLFU baseline is near its optimum on workload-a. The notes also say to fix the evaluation first and to keep cost changes separate from behavior changes.


---

- finished: 2026-10-20T20:17:16+00:00
- duration: 16.10s
- status: ok
- input_tokens: 4
- output_tokens: 1401
- cache_read_tokens: 41020
- cache_write_tokens: 12640
- cost_usd: 0.069551
