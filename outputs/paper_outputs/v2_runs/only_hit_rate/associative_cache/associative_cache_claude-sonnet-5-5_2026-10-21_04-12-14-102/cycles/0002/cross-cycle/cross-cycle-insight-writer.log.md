# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:18:19+00:00

## timeline

### 04:18:26 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural changes win; small tweaks do not. Wider buckets (geometry) gave a small gain. A probationary segment (admission control) gave a clearly larger one.\n- Separating one-touch entries from re-referenced ones is the most productive lever so far. Policy, not hash balance, appears to limit hit rate on this trace.\n\n## Recurring failures\n\n- Weakening hit-path promotion in short buckets (transposition, midpoint, half-promotion) lowered hit rate. Full move-to-back recency is hard to beat at small associativity.\n- Small-window frequency tie-breaks at the LRU end changed nothing measurable.\n- Multi-bucket or two-choice placement fails the slot-legality invariant, even on the baseline. Any key placed outside its hashed bucket is rejected as the gate is written.\n- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. Speed and size claims in all hypotheses remain untested, and hit-rate wins are unproven as net wins.\n\n## Open questions\n\n- Fix measurement first: can per-op-time and metadata-size get into the fitness vector? Do the wider-bucket and probation gains survive their added scan and bookkeeping costs?\n- What probation size is best (smaller or larger than the current share)?\n- Does a real probation-to-protected frequency test, or ghost-history information, add anything on top of the probationary segment?\n- Should the slot-legality invariant be deliberately relaxed to allow placement experiments, or should they be dropped?\n\n## Per-policy notes\n\n- Plain LRU (move-to-back) buckets: strong baseline. Promotion-weakening variants lose to it.\n- Probationary segment (2Q-like, separate region): best result so far. It builds on the wide-bucket geometry, and the tuning space is still open.\n- Frequency-aware victim choice: no effect at a small window. A wider window or a promotion test is untried.\n- Two-choice or multi-bucket placement: blocked by the invariant, with no performance evidence either way.\n"
}
```

### 04:18:29 chat

I wrote `cross-cycle-insights.md` to the cycle 0002 cross-cycle output directory. It has the four required sections in the fixed order and is built only from the two ledger cycles.

- **Recurring wins:** Structural changes paid off and small tweaks didn't. The probationary segment gave a clearly larger hit-rate gain than the wider buckets did.
- **Recurring failures:**
  - Weakening promotion in short buckets lowered hit rate.
  - Small-window frequency victim choice changed nothing.
  - Two-choice placement failed the slot-legality check.
  - Per-op-time and metadata-size have been missing in both cycles, so no speed or size claim has been tested.
- **Open questions:**
  - Get per-op-time and metadata-size measured.
  - Find the best probation size.
  - Test whether a frequency or ghost-history promotion test adds anything.
  - Decide whether to relax the placement invariant or drop placement experiments.
- **Per-policy notes:** These cover plain LRU, the probationary segment, frequency-aware victim choice and two-choice placement.

I used no numbers, only qualitative terms.


---

- finished: 2026-10-21T04:18:30+00:00
- duration: 11.28s
- status: ok
- input_tokens: 4
- output_tokens: 1199
- cache_read_tokens: 35039
- cache_write_tokens: 6398
- cost_usd: 0.041369
