# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-20T20:03:45+00:00

## timeline

### 20:03:54 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-37-44-834/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changes that preserve frequency information and loosen admission slightly are the only ones that ever helped (cycle 3 tie-admit duel). Gains were tiny.\n- Behavior-preserving cost changes (single-hash row derivation, index reuse, slab links) match the parent on hit-rate. They are safe candidates, but their size and speed benefit has never been measured.\n- Windowed main-region structure gave a marginal gain early (cycle 1) and has been the stable base since.\n\n## Recurring failures\n\n- Anything that degrades frequency estimates loses hit-rate. This covers narrower rows (any size, 2x or gentle), 4-bit counters without aging, correlated or blocked row indices, and doorkeepers. The sketch is near the width where collisions matter.\n- Decay/aging, window-size tuning (static or adaptive), and admission loosening past tie-admit gave nothing or lost on workload-a. The trace looks stable and frequency-driven.\n- Segment-internal tuning (protected share, demotion victim) has been flat: ties with the parent.\n- \"Winner\" labels are often near-ties or least-bad losers (cycles 2, 4, 5, 6), so they are weak evidence.\n- Evaluation gap: for six cycles fitness has reported hit-rate only, with no metadata-size or per-op-time. Cost-oriented ideas cannot be judged, and only their downside shows. There is also only one trace and cache size.\n\n## Open questions\n\n- Do single-hash, index reuse and slab links reduce per-op-time or metadata? They are free wins if they do.\n- Is workload-a saturated for hit-rate? Six cycles of tweaks produced only noise-level movement.\n- Does aging, adaptive windowing or a doorkeeper pay off on a trace with shifting popularity or a different cache size?\n- Can a compact or blocked sketch with decorrelated rows and aging keep hit-rate while gaining size or locality?\n- Is the small tie-admit gain real, or noise?\n\n## Per-policy notes\n\n- TinyLFU/W-TinyLFU family (the only policy so far): the base is a window plus segmented main with a count-min sketch and a tie-admit duel.\n- Sketch: u8 counters with conservative update cost no accuracy. 4-bit counters, narrower rows and blocked rows lose. Hash sharing is safe on hit-rate.\n- Admission: tie-admit is the sweet spot. Looser rules swap in colder entries.\n- Window and segments: window size, protected share and demotion choice are all near-optimal or irrelevant on workload-a.\n- Next step before any new hit-rate hypotheses: add size and speed metrics and a second trace, then verify the cost-win candidates.\n"
}
```

### 20:04:04 chat

I wrote `cross-cycle-insights.md` in the cycle 0006 `cross-cycle` output directory. It has the four fixed sections in order and is built only from the six-cycle ledger digest. I did not read the previous insights file.

- **Recurring wins:** Only changes that kept frequency information and loosened admission slightly ever helped, and the gains were tiny. Behavior-preserving cost changes (single-hash, index reuse, slab links) matched the parent on hit-rate, but their size and speed benefit is still unmeasured.
- **Recurring failures:**
  - **Frequency estimates:** Anything that weakens them loses hit-rate. That covers narrower rows, 4-bit counters, blocked rows and doorkeepers.
  - **Flat on workload-a:** Aging, window tuning and segment tuning gave nothing, and several "Winner" labels were near-ties.
  - **Evaluation gap:** Six cycles of hit-rate-only fitness and a single trace.
- **Open questions:** Whether the cost changes actually save size or time, and whether workload-a is saturated. Whether aging and adaptive windowing pay off on a trace with shifting popularity, and whether the small tie-admit gain is real.
- **Per-policy notes:** The ledger covers only the TinyLFU family (window plus segmented main, count-min sketch, tie-admit duel), so the notes cover its sketch, admission, window and segment findings. They end with the next step: add size and speed metrics and a second trace before any new hit-rate hypotheses.

I kept it qualitative and gave no numbers, since the digest has none.


---

- finished: 2026-10-20T20:04:04+00:00
- duration: 19.41s
- status: ok
- input_tokens: 4
- output_tokens: 1589
- cache_read_tokens: 37962
- cache_write_tokens: 9559
- cost_usd: 0.058497
