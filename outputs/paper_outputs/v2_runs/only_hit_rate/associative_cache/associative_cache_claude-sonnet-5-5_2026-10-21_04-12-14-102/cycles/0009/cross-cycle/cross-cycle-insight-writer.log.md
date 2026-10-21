# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:39:49+00:00

## timeline

### 04:39:59 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-12-14-102/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- A structural probation/protected split (admission control separating one-touch from re-referenced entries) is the one big policy win; it beat plain LRU in every cycle that tested it.\n- A lean positional split with no extra per-entry state (no ghost, counters, adaptive targets) captures nearly all the gain; it was rediscovered independently several times.\n- Small probation (about slots/16, roughly 4 slots at baseline width) won; the trend turned when going smaller.\n- Wider associativity stacks on the split, with diminishing increments (first doubling clear, later ones marginal).\n- Speed-motivated, policy-preserving refactors \"win\" only as exact hit-rate ties; they are not real evidence.\n\n## Recurring failures\n\n- Weakening or delaying promotion (transposition, midpoint, half-promotion, two-hit rules) loses hit rate; first-hit promotion is hard to beat with short reuse distances.\n- Ghost history and ghost-hit promotion add state without gain; a ghost hit is weak reuse evidence here.\n- Small-window frequency victim choice and recency-stamp overrides of probation-first eviction were flat or negative.\n- Insertion point tweaks (victim-end/LIP, mid-probation) cost hits: new keys need residency to earn a second reference.\n- Multi-bucket placement (two-choice) fails the slot-legality invariant.\n- Demotion placement and further probation shrinking are flat or negative; hit-rate knobs in this split are essentially exhausted.\n- Many \"wins\" compare against the baseline rather than the incumbent, so they are not increments.\n\n## Open questions\n\n- Per-op-time and metadata-size have never been in the fitness vector (nine cycles), so no cost claim (wide scans, rotate refactor, skip-move, \"lean\" footprint) is verified. Fix this first.\n- Does the marginal gain from 4x-wide buckets survive timing cost?\n- Is slots/8 vs slots/16 probation on wide buckets actually better than the incumbent?\n- Are further gains only possible from a different policy family (e.g. frequency-aware admission) or visible only on other traces?\n- Are exact ties real neutrality or edits that never took effect (authors could not read overlay code)?\n\n## Per-policy notes\n\n- Plain move-to-back LRU: baseline; hard to beat via promotion tweaks alone.\n- Segmented LRU (probation/protected, first-hit promotion, small probation, boundary-area insertion): the incumbent family; at or near its hit-rate ceiling on the LIRS-style trace.\n- 2Q/ghost-style and adaptive variants: no benefit over the fixed lean split.\n- Frequency-aware victim selection: no measurable effect in small windows.\n- Wide-associativity segmented LRU: tiny, diminishing hit-rate gains; cost unpriced.\n"
}
```

### 04:40:02 chat

I wrote `cross-cycle-insights.md` to the output dir with the four fixed sections, in the required order. It is built only from the nine ledger digests.

The main points it records:
- **Wins:** the probation/protected split is the one large policy gain, and a lean version with no extra per-entry state captures nearly all of it. Wider buckets stack on top with diminishing returns.
- **Failures:** delayed or weakened promotion, ghost history, small-window frequency victim choice and insertion-point changes all lost hit rate or did nothing. Two-choice placement fails the invariant gate.
- **Open questions:** per-op-time and metadata-size have never been measured in nine cycles, so no cost claim has been checked. Whether the 4x-wide buckets, slots/8 probation or exact ties hold up is still unknown.
- **Per-policy notes:** plain LRU, segmented LRU, 2Q/ghost variants, frequency-aware victim choice and wide-associativity segmented LRU.

I kept it qualitative, with no invented numbers.


---

- finished: 2026-10-21T04:40:02+00:00
- duration: 13.73s
- status: ok
- input_tokens: 4
- output_tokens: 1437
- cache_read_tokens: 40187
- cache_write_tokens: 11822
- cost_usd: 0.066474
