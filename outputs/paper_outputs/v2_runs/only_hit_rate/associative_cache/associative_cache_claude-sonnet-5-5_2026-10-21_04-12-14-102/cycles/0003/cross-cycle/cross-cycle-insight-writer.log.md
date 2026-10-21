# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0003/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:21:12+00:00

## timeline

### 04:21:21 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0003/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural separation of one-touch from re-referenced entries (probationary/protected split) is the only change with a clear gain; it won in cycles 2 and 3.\n- Wider-bucket geometry gave a small gain (cycle 1) and has served as the base since.\n- Tuning the split size gives small but positive gains. Most of the benefit comes from having the split at all.\n- Each cycle's winner built directly on the previous winner, so incremental stacking works.\n\n## Recurring failures\n\n- Weakening hit-path promotion in plain LRU buckets (transposition, midpoint, half-promotion) lowered hit rate. Full move-to-back recency is hard to beat.\n- Small-window frequency-aware victim selection changed nothing measurable. A tie-break among a few old entries is too weak.\n- Multi-bucket placement (two-choice) fails the slot-legality invariant, even on the baseline.\n- Ghost-history ideas (ghost ring with direct promotion, adaptive targets) did not beat their parent. A ghost hit is weak reuse evidence, and promoting on it displaces entries that had earned protection.\n- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. Speed and size claims remain untested, and added state (counters, rings, markers) is unpriced.\n\n## Open questions\n\n- Fix the evaluation so per-op-time and metadata-size are reported. Until then, no trade-off can be judged and every gain is provisional.\n- Where does the smaller-probation trend turn? Is the optimum near the current size?\n- Does a fixed probation size match adaptive sizing, making extra state unnecessary?\n- Would a stricter promotion rule (ghost hit plus a second signal, or a real probation-to-protected frequency test) rescue history-based ideas?\n- Do the wider-bucket and split gains survive once scan and shift costs are measured?\n\n## Per-policy notes\n\n- Plain LRU (move-to-back): strong baseline at small associativity on this LIRS trace. Promotion tweaks lose to it.\n- Probation/protected split (2Q-like, within a bucket): the current best family. Smaller probation share was slightly better. Gains are now incremental.\n- Frequency-aware victim choice: no effect on its own. Retest only with a wider window or as a promotion test.\n- Ghost-history/adaptive variants: no gain over the fixed split. Deprioritize unless promotion is made stricter.\n- Two-choice or other placement schemes: blocked by the invariant unless it is deliberately relaxed.\n"
}
```

### 04:21:24 chat

I wrote `cross-cycle-insights.md` in the cycle 0003 cross-cycle output directory, using the four fixed sections in order. I built it only from the ledger digest in the prompt.

- **Recurring wins:** The probation/protected split is the one clear win, in cycles 2 and 3. Wider buckets and smaller probation sizes gave small gains on top of it.
- **Recurring failures:** Weakening hit promotion lost hit rate. Small-window frequency victim choice did nothing, and ghost-history ideas did not beat their parent. Placement outside the hashed bucket failed the invariant. Per-op-time and metadata-size have been unmeasured in all three cycles, so every gain is provisional.
- **Open questions:** The main ones are fixing the evaluation so speed and size are reported, and finding where the smaller-probation trend turns. The rest are whether a fixed split matches an adaptive one, and whether a stricter promotion rule could rescue ghost history.
- **Per-policy notes:** These cover plain LRU, the probation split, frequency victim choice, ghost/adaptive variants and placement schemes.

I did not read the previous insights file, and I used qualitative terms only, with no numbers.


---

- finished: 2026-10-21T04:21:25+00:00
- duration: 12.74s
- status: ok
- input_tokens: 4
- output_tokens: 1391
- cache_read_tokens: 35661
- cache_write_tokens: 7192
- cost_usd: 0.046589
