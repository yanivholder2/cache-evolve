# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:16:37+00:00

## timeline

### 09:16:50 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Small, policy-neutral or narrowly targeted changes are the only ones that have won (fast single-hash/flat sketch in 0001, frequency-sampled victim selection in 0002). Both gains were marginal on hit-rate.\n- Keeping long-run frequency history has helped on workload-a in both cycles. Undecayed counts beat freshness.\n- Winners stack: v003 hashing (0001) carried into v001 sampling (0002). Building on the latest winner has not regressed.\n\n## Recurring failures\n\n- Sketch aging/halving lost hit-rate in both cycles. It lost with conservative update at a short period, and it lost alone at a longer period. Halving creates ties, and strict-`>` keep-victim makes newcomers lose those ties.\n- Window restructuring has not paid off. A fixed 1% segmented layout was neutral, and a naive hill-climbing adaptive window drifted toward a large LRU-like region, which the LIRS-style trace punishes.\n- Evaluation has never reported per-op-time or metadata-size, two cycles running. Every latency or space claim (fast hashing, extra miss-put estimates, per-get climber bookkeeping) is unverified.\n- Changing two levers at once (0001 v002) blurred attribution. Isolating aging in 0002 showed that conservative update and/or the short period caused most of the damage.\n\n## Open questions\n\n- Tie handling in the admission contest has been named as the bottleneck in both cycles but has never been tested. Candidates: `>=`, random admit on ties, a doorkeeper. Test each alone and on top of victim sampling.\n- Is victim sampling capped by cold or stale-high head entries? A wider sample, or one placed elsewhere, could show whether it scales.\n- Can aging be rescued by a very long period combined with a tie-aware contest, or is freshness simply worthless on workload-a?\n- Would a damped, warmed-up climber with small steps and a contest-on-shrink do better, or is a small fixed window simply right?\n- Does any of this hold beyond workload-a? Every conclusion so far rests on one trace.\n\n## Per-policy notes\n\n- W-TinyLFU (window + SLRU): window size or shape has been neutral or harmful. A small fixed window appears near-right for this trace.\n- TinyLFU admission (CM sketch + strict-`>` contest): this is where the hit-rate leverage is. Ties decide most contests, and the default tie rule favors the victim.\n- Sketch maintenance (aging, conservative update): consistently harmful on workload-a so far. Deprioritize this unless it is paired with tie-aware admission.\n- Hashing/sketch layout: the single fast hash with a flat sketch is the current base. Its claimed speed benefit is still unmeasured, and its small hit-rate shift probably comes from collisions.\n- Victim selection: frequency-sampled victims among the oldest LRU entries gave a small, real gain. Sample width and placement are unexplored.\n"
}
```

### 09:16:54 chat

I've written `cross-cycle-insights.md` in the cycle 0002 cross-cycle folder, using the four required sections in order. It's built only from the ledger for cycles 0001 and 0002, and it contains no numbers.

- **What keeps winning:** Small, narrowly targeted changes are the only ones that have won: the fast single hash in 0001 and the frequency-sampled victim choice in 0002. Both gave only marginal hit-rate gains. Keeping long-run frequency counts has beaten freshness on workload-a in both cycles, and each winner has built cleanly on the last.
- **What keeps failing:** Sketch aging lost hit-rate in both cycles, as did changing the window structure. Speed and memory use have never been measured, so every claimed latency or space benefit is still unverified.
- **Main open question:** How the admission contest handles ties has been named as the bottleneck in both cycles but never tested directly. All conclusions so far come from workload-a alone.
- **Per-policy notes:** These are grouped by window/segment layout, the admission contest, sketch maintenance, hashing, and victim selection.


---

- finished: 2026-10-21T09:16:55+00:00
- duration: 17.27s
- status: ok
- input_tokens: 4
- output_tokens: 1504
- cache_read_tokens: 34536
- cache_write_tokens: 7004
- cost_usd: 0.093035
