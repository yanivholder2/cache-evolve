# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-28T13:21:13+00:00

## timeline

### 13:21:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-28_12-54-17-355/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 0001–0004)\n\n## Recurring wins\n\n- Structural changes to the admission contest keep working. Changing who competes (the window), which resident is the victim (SLRU probation), or how big the window is (the adaptive climber) each won a cycle. Each one also added to the gains of the one before it (1% window → adaptive window → adaptive window + 80/20 SLRU).\n- SLRU's probation/protected split is the most reliable component. It helped web_0 in every cycle where it was tried, and it was the only change that moved web_0 at the large size once the climber had stalled there.\n- Strict `>` admission, where the victim wins ties, has held up. Relaxing it (recency tie-break, `>=` at a frequency floor) regressed on both traces.\n- Gains are consistently largest at the small cache size and shrink at the large size.\n\n## Recurring failures\n\n- Changes to timing or feedback-loop knobs have failed in three cycles running: the aging period, the climber's sample period (tied to capacity, fixed, or tied to turnover), and alternating-turn climbers. In each case the feedback signal was noisy, lagged behind lazy rebalancing, or was biased by integer truncation, warmup, or a first move that always went the wrong way.\n- Global periodic sketch halving fixed one problem in isolation (the cap-at-15 regression went away) but hurts once the adaptive window exists. It wears down the main region's frequency lead, and it hurts hm_0 at the small size the most.\n- Counter caps combined with strict `>` cause ties at the cap and reject hot newcomers. Dropped.\n- Metadata-size and per-op-time have gone unmeasured for four cycles, even though every continuation asked for them. Every winner adds cost (region tags, extra lists, demotions, sampling), and none of it has been quantified.\n- Since cycle 0004 the lineage looks close to a local optimum: the latest winner is basically flat against its parent, and the other variants regressed.\n\n## Open questions\n\n- Sketch double count: a miss followed by an insert counts the key twice, so a key seen once already has an estimate of 2. Does fixing this on its own change hit-rate, especially on hm_0? Does it make frequency floors usable? Does it explain the earlier problems with aging periods?\n- Can a counterfactual signal (ghost list or shadow cache) replace hill-climbing on noisy deltas, for the window size and for the protected fraction?\n- Is the 80/20 protected split already close to optimal, or can a climber with a float step and properly separated knobs learn a better split?\n- Aging aimed at specific entries (only demoted protected entries, or gated on the climber's state) instead of global halving.\n- What do the current lineage's components actually cost in metadata-size and per-op-time?\n\n## Per-policy notes\n\n- **TinyLFU baseline**: its weakness is the admission contest, which rejects every first-time key once the cache is full, not its eviction order.\n- **W-TinyLFU window**: the fixed 1% window was too small at small sizes. The adaptive climber helps at small sizes and is ineffective at large sizes. Every attempt to retune the climber's timing has failed.\n- **SLRU main region**: strong on web_0 (scan-like, one-hit traffic). On hm_0 it is flat or slightly worse, because the short probation stay cuts recency-driven entries short and stale demoted entries keep inflated counts.\n- **Sketch aging**: helps web_0 when there is no adaptive window. It is harmful on the current lineage and has been dropped there.\n- **hm_0**: the hard trace in every cycle, and hardest at the small size. Plain LRU already captures most of its reuse at the large size, and every change has dipped it somewhere.\n"
}
```

### 13:21:34 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle folder, built fresh from the four-cycle ledger with the four required sections in order.

- **Recurring wins:** changes to how the admission contest works keep winning, and each new winner added to the one before it (1% window, then adaptive window, then adaptive window plus 80/20 SLRU). SLRU is the most reliable piece. Keeping strict `>` admission, where the existing entry wins ties, has held up.
- **Recurring failures:** in three cycles in a row, changes to timing or feedback-loop settings failed, including aging periods and the climber's sample periods. Global sketch halving hurts once the adaptive window is in place. Metadata-size and per-op-time have still not been measured after four cycles. Cycle 0004's winner was about flat against its parent, so this line of changes looks close to a local optimum.
- **Open questions:** the bug where a single access is counted twice in the sketch, using a ghost list or shadow cache instead of hill-climbing, whether the 80/20 protected split is already near optimal, aging only selected entries instead of the whole sketch, and measuring costs.
- **Per-policy notes:** grouped by baseline, window, SLRU and aging, plus a note that hm_0 has been the hard trace in every cycle.


---

- finished: 2026-10-28T13:21:34+00:00
- duration: 21.28s
- status: ok
- input_tokens: 4
- output_tokens: 1826
- cache_read_tokens: 38164
- cache_write_tokens: 9620
- cost_usd: 0.121129
