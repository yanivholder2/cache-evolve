# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:42:57+00:00

## timeline

### 04:43:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0010/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- A structural probation/protected split beats plain LRU. The gain comes from having the split, not from tuning it.\n- A lean positional split (no ghost, counter or adaptive state) captures nearly all the benefit. It has been rebuilt independently several times with the same result.\n- A small probation segment (about slots/16) is at or near the optimum. Tuning the size gave small gains and then turned negative.\n- Wider associativity stacks on the split, but the increments shrink with each doubling (clear, then marginal, then tiny). Misses now look capacity- or recency-driven, not conflict-driven.\n- Speed-motivated refactors (rotate rewrite, skip-move) tie their parents on hit rate. They count as Winners by default, and their goal is unjudged.\n\n## Recurring failures\n\n- Weakening or delaying promotion (transposition, midpoint, two-hit rule, ghost-hit promotion) loses hit rate. New keys need first-hit promotion and enough residency to earn it.\n- Insertion-position changes (victim-end/LIP, mid-probation) lose hits at both extremes.\n- Eviction overrides lose hits: small-window frequency victim choice (flat), recency-stamp compare, and one-bit reuse flag. The segment boundary already encodes the reuse evidence.\n- Ghost history and adaptive targets add state without gain.\n- Larger probation fractions (slots/8) take protected capacity and gain nothing. Shrinking below slots/16 also hurts.\n- Placement outside the hashed bucket (two-choice) fails the slot-legality invariant.\n- Demotion placement is exactly neutral.\n- Per-op-time and metadata-size have been unmeasured in every cycle. No cost-side claim has ever been confirmed or refuted.\n- Several variants compare against the baseline rather than the incumbent, so their gains can't be read as increments.\n- Authors can't read the overlay code. An exact tie may mean the edit had no effect.\n\n## Open questions\n\n- Do per-op-time and metadata-size fit in the fitness vector? Until they do, nothing about wide buckets (up to 8x), skip-move or the rotate refactor can be priced.\n- Does the marginal hit-rate gain from very wide buckets survive the cost of longer scans?\n- Is the ceiling for this design on this trace reached? Would a different policy family, such as frequency-aware admission or a separate admission filter, add anything?\n- Do the results hold on traces other than the LIRS-style one, given the tuning may be trace-specific?\n- Would a multi-bit or recency-aware reuse signal work where the one-bit flag failed?\n- Are exact hit-rate ties real neutrality, or are the edits ineffective?\n\n## Per-policy notes\n\n- Plain move-to-back LRU: the baseline. Weakened-promotion variants (2Q-style half-promotion, transposition) are worse than it at small associativity.\n- Segmented LRU (probation + protected, first-hit promotion, slots/16 probation, positional): the current base and the main source of gains. Tuning knobs are essentially exhausted.\n- Wide-bucket segmented LRU: the best hit rate so far, with shrinking returns per doubling. Its speed cost is unknown.\n- Skip-move / rotate variants: hit-rate neutral and unjudged on speed.\n- Frequency, ghost and reuse-bit variants: flat or negative on top of the split.\n- Next steps: price the existing designs first, then try a different policy family or other workloads. Avoid more split knobs.\n"
}
```

### 04:43:11 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four required sections in order. It is built only from the ledger digest of cycles 0001–0010.

The main points it records:
- **Recurring wins:** The probation/protected split is where nearly all the gain comes from. Wider buckets still help, but each doubling adds less.
- **Recurring failures:** Almost every other lever lost hit rate or did nothing. That covers weaker or delayed promotion, insertion-position changes, eviction overrides, ghost history and larger probation fractions. Per-op-time and metadata-size have never been measured.
- **Open questions:** Do the speed and size costs cancel the small gains from very wide buckets? Has this design hit its ceiling on the trace, and does it hold on other traces?
- **Per-policy notes:** Plain move-to-back LRU is the baseline. Segmented LRU is the current base, and its tuning knobs are used up. Wide-bucket and skip-move variants are unpriced.

I kept the file qualitative and invented no numbers.


---

- finished: 2026-10-21T04:43:12+00:00
- duration: 14.48s
- status: ok
- input_tokens: 4
- output_tokens: 1665
- cache_read_tokens: 40919
- cache_write_tokens: 12781
- cost_usd: 0.07274
