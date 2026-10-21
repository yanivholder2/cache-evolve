# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:24:12+00:00

## timeline

### 04:24:22 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Structural separation of one-touch from re-referenced entries (probation/protected split) is the one idea that has paid off repeatedly. It gave the largest single gain (cycle 2).\n- Shrinking the probation share keeps helping, but with diminishing returns. Independent routes in cycle 4 both ended at a small probation segment. The turning point has not clearly been found.\n- A lean split on baseline geometry appears to capture most of the benefit. Extra bookkeeping added in earlier cycles looks unnecessary.\n- Wider buckets gave a small, early hit-rate gain. It is the base geometry for later work, but its cost is unmeasured.\n\n## Recurring failures\n\n- Weakening or delaying hit-path promotion (transposition, midpoint, half-promotion, two-hit rule) lowers hit rate. First-hit move-to-back is hard to beat, especially with short or small-probation segments.\n- Ghost-history ideas (ghost rings, ghost-hit promotion, adaptive targets) added nothing over the plain split. A ghost hit is weak reuse evidence here.\n- Small-window frequency victim selection at the LRU end changed nothing measurable.\n- Placement outside the hashed bucket (two-choice) fails the slot-legality invariant. This holds for any multi-bucket scheme unless the gate is relaxed.\n- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. All wins are hit-rate only, and every added piece of state is unpriced. Comparisons against plain LRU instead of the current parent also overstate increments.\n\n## Open questions\n\n- Where is the minimum probation size? Do sizes below the current point still help, or does scan resistance break? Check one size above as a control.\n- Does the lean split match the full accumulated stack, so older state can be removed?\n- Do hit-rate gains survive once per-op-time and metadata-size are measured? Fixing the evaluation is overdue.\n- Would a larger probation segment make delayed promotion viable, or is first-hit promotion simply better?\n- Would ghost history work with a stricter second signal, or should it be dropped?\n\n## Per-policy notes\n\n- Plain move-to-back LRU buckets: the reference. Promotion tweaks lose to it. Geometry changes give small gains.\n- Probation/protected split (2Q-like): the winning family. Small fixed probation with first-hit promotion is the best known configuration.\n- Frequency-aware victim choice: no effect at small windows. A wider window or a real probation-to-protected frequency test is untried.\n- Ghost/adaptive variants: no gain over the fixed split. Avoid unless promotion is made stricter.\n- Two-choice or multi-bucket placement: blocked by the invariant.\n"
}
```

### 04:24:25 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle output directory. It uses the four fixed sections in order and is built only from the ledger digest.

- **Recurring wins:** The probation/protected split is the one idea that keeps paying off, and shrinking the probation share keeps helping with diminishing returns. A lean split on the baseline geometry seems to capture most of the benefit.
- **Recurring failures:** Weakening or delaying promotion, ghost-history ideas, small-window frequency victim choice, and placement outside the hashed bucket all failed to pay off. Per-op-time and metadata-size have been missing from the fitness vector in every cycle, so all wins are hit-rate only.
- **Open questions:** Where the probation-size minimum is, whether the lean split matches the full stack, and whether the gains survive once speed and size are measured. Also whether delayed promotion works with a larger probation segment, and whether ghost history works with a stricter second signal.
- **Per-policy notes:** Grouped by plain LRU buckets, the probation/protected split, frequency-aware victim choice, ghost/adaptive variants, and multi-bucket placement.

I kept numbers out of it and described everything qualitatively, as the constraints require.


---

- finished: 2026-10-21T04:24:26+00:00
- duration: 13.69s
- status: ok
- input_tokens: 4
- output_tokens: 1472
- cache_read_tokens: 36425
- cache_write_tokens: 8022
- cost_usd: 0.050873
