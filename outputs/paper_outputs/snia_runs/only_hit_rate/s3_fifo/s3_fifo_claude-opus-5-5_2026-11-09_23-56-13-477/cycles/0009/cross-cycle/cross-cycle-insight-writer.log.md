# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-11-10T10:26:36+00:00

## timeline

### 10:26:55 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-11-09_23-56-13-477/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0009)\n\n## Recurring wins\n\n- Per-key evidence beats global signals. Every real policy gain (0004 ghost-time freq gate, 0005 ghost-age floor for freq-0 returnees, 0009 correlated-reference cap) reads something the key itself carries. No global rate or occupancy proxy has ever won.\n- Strict admission into a full main holds up. The baseline two-reuse rule plus freq reset (0001–0003) has never been beaten by loosening it. Gains come from filtering or deferring keys before they reach main, not from protecting them once inside.\n- Deferring is safer than rejecting. Rerouting weak keys back to small (0004), or holding freq 1 into the ghost until a later spaced return (0009), keeps their chance to come back instead of discarding them.\n- Ghost age gated at about one small-queue turnover is a confirmed local optimum: 2× lost in 0006 and ½× lost in 0007.\n- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) have stayed bit-identical in hit-rate every time, so they are always safe to port onto the current policy parent.\n\n## Recurring failures\n\n- Single constant tweaks (threshold, static small/main split) trade hm_0 against web_0 and never win on both traces (0001, 0002, 0008).\n- Global adaptive signals: main occupancy fills and stays full (0002). Ghost-return-rate switching fired on the wrong traces (0003). An unweighted ARC-style split drifts. None of them follow steady-state value.\n- Extra protection or credit in main or small (returnee credit, relaps, a saturated-freq head start, extra small laps) displaces useful residents. Main is always full, and quick freq-0 returnees especially should not be pushed toward main's tail (0003, 0005–0007).\n- Mechanisms that erase or dilute the per-key signal break the ghost gate. Freq-resetting relaps (0005), and extra ghost producers that share the FIFO or budget (0007), are examples.\n- Scaling the ghost horizon or budget does not help. Uniform doubling was roughly neutral (0008), age tiering removed a privilege hm_0 needs (0009), and a `main_ghost` scaled to `main_size` did nothing (0009).\n- web_0 at the large size has resisted every lever: ghost routing, age, budgets, the split, main-victim memory and burst filtering. At most it moved marginally.\n- The cost dimensions (per-op-time, metadata-size) have gone unmeasured for nine cycles running, despite repeated requests. Cost hypotheses cannot be judged, and ghost-enlarging variants carry metadata costs nobody has seen.\n\n## Open questions\n\n- Does a narrower correlated-reference window keep 0009-v003's hm_0 gains while sparing web_0's genuine quick reuses at the small size?\n- Is web_0 at the large size at its ceiling for this policy family? The last untried probes are the `main_hit` gate on main-victim ghosting and the freq assigned to main-ghost returnees. If both fail, stop targeting this cell.\n- What do the cost refactors actually save in per-op-time and metadata-size, and does alignment on 8-byte keys cancel the ghost-layout savings?\n- Can a per-key signal ever break the hm_0/web_0 trade-off at both sizes at once, or are the remaining gains limited to the small size?\n\n## Per-policy notes\n\n- S3-FIFO (the only family explored), current line: baseline two-reuse rule plus freq reset → O(1) index → freq-gated ghost path (0004) → age-gated freq-0 returnees at 1× turnover (0005-v002) → correlated-reference cap (0009-v003, the current base).\n- Trace behaviour: hm_0 rewards more probation and memory at tight capacity, and has burst-only keys that pollute main. web_0 punishes privileges for keys returning after a long gap, and treats quick reuse after insertion as real evidence.\n- Cache size: gains concentrate at the small size. Only windows that scale with `small_size` have reached the large size.\n- Exhausted axes (do not retry): static split, threshold constants, ghost-age window tuning, relap stacking, age ceilings and tiers, ghost and `main_ghost` budget scaling, stamp-clock fixes.\n"
}
```

### 10:27:09 thinking



### 10:27:09 chat

I've rebuilt `cross-cycle-insights.md` from scratch using only the ledger for cycles 0001–0009, under the four required sections.

- **Recurring wins:**
  - Every real gain came from evidence each key carries itself, never from global rate or occupancy signals.
  - Keeping strict admission into main and sending weak keys back to the small queue (instead of rejecting them) has held up across cycles.
  - The ghost-age window at about one small-queue turnover is a local optimum: both 2× and ½× lost.
  - The cost-only refactors have always left hit-rate exactly the same.
- **Recurring failures:**
  - Single-constant tweaks and global adaptive signals keep trading hm_0 against web_0.
  - Extra protection or credit for a population pushes useful entries out of a full main or small queue.
  - Changes that erase the per-key signal break the ghost gate.
  - Scaling the ghost budget or horizon doesn't help.
  - web_0 at the large cache size hasn't moved under any lever.
  - Per-op-time and metadata-size have never been measured, nine cycles running.
- **Open questions:**
  - Whether a narrower window for cycle 0009's correlated-reference cap keeps hm_0's gains without hurting web_0.
  - Whether web_0 at the large size is simply at its ceiling. Two probes are left; if both fail, stop targeting that cell.
  - What the cost refactors actually save.
  - Whether any per-key signal can break the hm_0/web_0 trade-off.
- **Per-policy notes:** S3-FIFO is the only policy explored. The notes trace the lineage up to the current base (cycle 0009's v003) and list how each trace behaves, why gains cluster at the small cache size, and which levers are used up.


---

- finished: 2026-11-10T10:27:10+00:00
- duration: 33.85s
- status: ok
- input_tokens: 4
- output_tokens: 2211
- cache_read_tokens: 42840
- cache_write_tokens: 14490
- cost_usd: 0.169724
