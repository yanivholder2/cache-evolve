# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-28T12:36:18+00:00

## timeline

### 12:36:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- Filtering out blocks seen only once is the idea behind every winner in the lineage: SLRU (c1), then W-TinyLFU over SLRU (c2), then a stricter one-shot filter (c3). Scan-heavy hm_0 rewards it every time.\n- Soft, relative signals beat hard overrides. The tie-admitting duel against the current victim has won since c2. Hard gates, direct promotion and bypasses all lost (c1, c2, c4).\n- Enlarging the window was the only lever that moved small-size web_0 by a large amount. The hill-climbed window with absolute caps and a floor at the old size (c4) is the single biggest gain in the run.\n- Changing one lever per variant made results attributable (c4 onward). Bundled changes in c3 could not be diagnosed.\n- Gating parameters by cache size is reliable. Size-gated changes left the other side of the gate exactly unchanged (c7, c8). The large-only larger step (c7) and the large-only 75% protected split (c8) each gave small strict gains.\n- These parts carry weight and should be kept: the long sketch reset, the doorkeeper, dedup on miss-then-put, the wide sketch, a large protected segment (at least at small size), and the deterministic tie-admitting duel.\n\n## Recurring failures\n\n- Loosening admission. Hard TinyLFU reject was too strict (c1), but every loosening since then has lost, and small-size hm_0 lost first: in-window bypass (c4), reuse bonus (c5), random admission of warm keys (c7). Admission is not too strict.\n- Counting in-window reuse again. Window hits are already in the sketch, so any bonus double-counts them and tilts close duels toward recency.\n- Fixed fractions or parameters scaled with capacity tend to fail at one end of the size range: ghost list (c2), 1% front segment (c2), linearly scaled climber knobs (c3).\n- Climber tuning is used up. Ceiling, shorter or longer period, larger step cap, two-drop hysteresis and flip damping were all neutral or worse (c5–c8). Single-sample drops are real signals, so delaying reactions hurts.\n- Faster sketch aging (c4) and a more accurate doorkeeper (c8) both hurt small-size hm_0 and did not help web_0.\n- Ghost-to-protected promotion and LIRS (any HIR share) gave trace-dependent trades, not net gains.\n- Shrinking protected globally pushes hm_0's reuse set into the eviction path (c7).\n- Every cycle so far has reported hit-rate only. `metadata-size`, `per-op-time` and `win_cap` logs have never been read, so the cost claims are unverified for every policy.\n\n## Open questions\n\n- What limits small-size web_0? It has resisted climber, split, admission-loosening and doorkeeper changes. Untested small-only levers: the duel tie-break, the `reset_at` period, and the window/main split.\n- At large size, static gates now only move hit-rate between web_0 and hm_0. Can a runtime signal (response after a flip, how scan-heavy the window traffic is, an adaptive protected split) separate the two workloads?\n- Where does `win_cap` settle in each cell, and does it oscillate at small size?\n- What are the actual metadata and per-op costs of the sketch, doorkeeper and climber compared with plain SLRU or CLOCK?\n\n## Per-policy notes\n\n- **LRU (baseline):** beaten in nearly every cell by every segmented or filtered design.\n- **SLRU:** the core of every winner. Probation shields hm_0 from one-shot blocks. A longer probation helps large-size web_0 but costs hm_0 unless it is gated to large size only.\n- **CLOCK:** decays toward FIFO with a large ring on web_0. It was kept as a cost-axis candidate but never re-tested, because costs were never measured.\n- **TinyLFU / W-TinyLFU:** plain TinyLFU over LRU hurt recency traffic. W-TinyLFU with a tie-admitting duel over SLRU, an adaptive capped window, and a strict sketch (long reset, doorkeeper, dedup, wide) is the current lineage. Current parent: c8 v002 (75% large-size protected split, 80% at small size, large-only step).\n- **LIRS:** competitive with LRU but never beat W-TinyLFU. Tuning its HIR share swaps one trace for the other. Dropped.\n- **Ghost-list SLRU:** ghost hits are a weak signal on web_0. They may help only as input to the admission comparison, never as direct promotion.\n"
}
```

### 12:36:43 chat

I've written `cross-cycle-insights.md` in the cycle 0008 cross-cycle folder. It has the four required sections in order and was built only from the eight-cycle ledger, without reading any earlier version of the file.

The main points across cycles:

- **Recurring wins:**
  - **Approach that won:** every winner builds on filtering out blocks seen only once, using soft relative checks rather than hard rules. The current design is W-TinyLFU over SLRU with a duel that admits on ties.
  - **Biggest single gain:** the capped hill-climbed window from cycle 0004.
  - **Gating by size:** gating changes by cache size reliably leaves the other size unchanged. It produced the small large-size gains in cycles 0007 and 0008.
- **Recurring failures:**
  - **Admission:** every attempt to loosen admission has lost, and small-size hm_0 is hurt first.
  - **Window reuse:** extra credit for reuse inside the window counts the same evidence twice.
  - **Climber:** tuning its knobs is used up.
  - **Fixed fractions and LIRS:** settings tied to capacity fail at one end of the size range, and LIRS gave trades between traces, not net gains.
  - **Cost measurement:** in eight cycles, only hit-rate has been reported. `metadata-size`, `per-op-time` and the `win_cap` logs have never been read, so every cost claim is unverified.
- **Open questions:**
  - Small-size web_0 has resisted every lever tried. The untested small-size-only options are the duel tie-break, `reset_at`, and the window/main split.
  - At large size, fixed rules now only trade web_0 against hm_0, so a signal that adapts at runtime is the next idea to try.
- **Per-policy notes:** these cover LRU, SLRU, CLOCK, TinyLFU/W-TinyLFU, LIRS and the ghost-list variant. The current parent is cycle 0008's v002.


---

- finished: 2026-10-28T12:36:44+00:00
- duration: 25.81s
- status: ok
- input_tokens: 4
- output_tokens: 2362
- cache_read_tokens: 42071
- cache_write_tokens: 13842
- cost_usd: 0.166406
