# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:14:04+00:00

## timeline

### 20:14:22 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that preserve frequency information and leave admission semantics alone are neutral at worst (index reuse, single-hash indexing, slab links, flat u8 packing). They are candidate free cost wins, but none has been verified on size or speed.\n- Loosening the window-to-probation duel to tie-admit (cycle 3) is the only admission change that ever clearly helped. Everything after it was flat or negative.\n- Sketch width moves hit-rate monotonically with sharply diminishing returns. Wider rows (4x, and width 8) gained slightly, and narrower rows lost.\n- Count semantics and the admit threshold are coupled. Retuned together they restore parity (cycle 8), but not more.\n\n## Recurring failures\n\n- Anything that degrades frequency estimates lost hit-rate: narrower rows, 4-bit saturating counters, correlated or blocked rows, and a doorkeeper.\n- Decay and aging (halving, even very infrequent) never paid off. The workload is stable and the sketch is not decay-limited.\n- Admission-rule tuning (looser than tie-admit, recency tie-breaks, standalone single-count) is flat or slightly negative.\n- Segment-internal tuning (protected size, demotion choice, probation victim sampling) is flat. Window-size tuning and hill-climbing are also flat.\n- Sketch depth (an extra row) is flat.\n- \"Winners\" are often near-ties or slight losers, chosen on hit-rate alone among noise-level differences.\n- Evaluation is the blocker. In every cycle so far the fitness vectors report hit-rate only. Metadata-size and per-op-time are unmeasured, and only one trace and cache size is used.\n- Hypothesis text is sometimes muddled about scaling direction or parent. Variant authors usually cannot read the overlay source, so the proposals are generic.\n\n## Open questions\n\n- Do the cost-oriented changes (single-hash, index reuse, slab links, flat u8, narrower or wider sketch) actually reduce size or per-op-time? This cannot be answered until those dimensions are measured.\n- Is workload-a saturated for hit-rate? A second trace with shifting popularity, or a different cache size, is needed. It would also give aging, the doorkeeper, and adaptive windows a fair test.\n- Is the width trade-off worth its size cost? Compare intermediate widths against baseline on size.\n- Would 4-bit counters recover with decorrelated hashes? Would blocked rows do the same? Both need cost measurement.\n- Does a per-entry flag, such as the touched-in-window bit, cost more memory than its tiny gain?\n- Eviction or segment structure, rather than sketch and admission, may be the only untouched lever.\n\n## Per-policy notes\n\n- TinyLFU-style windowed design (window + segmented main + count-min sketch with sketch-gated admission): the tie-admit duel and the single-count with lower threshold are the current base. Hit-rate on workload-a is near its ceiling.\n- Sketch: u8 counters with conservative update are safe. 4-bit counters, blocking, and width cuts hurt. Wider helps slightly. Single-hash derivation is behavior-safe.\n- Admission: sketch estimate dominates, and in-window recency adds nothing beyond it.\n- Segments: protected/probation sizing and victim choice are irrelevant on this trace.\n- Before the next hypotheses, fix the fitness output (size, time, second trace). Then verify the cheap behavior-preserving changes on cost, and state the parent and scaling direction in every width or packing hypothesis.\n"
}
```

### 20:14:26 chat

I wrote `cross-cycle-insights.md` in the cycle's `cross-cycle` output directory, with the four fixed sections in order (Recurring wins, Recurring failures, Open questions, Per-policy notes). I didn't read the previous insights file.

The main conclusions the file records:

- **Evaluation is the blocker.** In all nine cycles the fitness vectors report hit-rate only, on one trace and one cache size. Metadata-size and per-op-time are unmeasured, so the cost-oriented changes can't be judged.
- **Hit-rate on workload-a looks near its ceiling.** Most sketch, admission, aging and segment changes landed flat or slightly negative, and the "Winner" is often a near-tie.
- **Changes that preserve frequency information are the safe ones.** Single-hash indexing, index reuse, slab links and flat u8 packing are candidate cost wins, but unverified.
- **Wider sketch rows help slightly, with diminishing returns.** Narrower rows and 4-bit counters lost hit-rate.


---

- finished: 2026-10-20T20:14:26+00:00
- duration: 21.56s
- status: ok
- input_tokens: 4
- output_tokens: 1693
- cache_read_tokens: 40337
- cache_write_tokens: 12227
- cost_usd: 0.070686
