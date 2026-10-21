# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:39:52+00:00

## timeline

### 09:40:08 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-21_09-11-54-029/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Rewarding proven reuse.** Changes that protect keys with demonstrated reuse in main have paid off: shrinking the small queue (cycles 1–2) and adding modest main retention (cap 3→7, cycle 3). Gains are small and shrink with each cycle.\n- **A bigger main beats a bigger ghost.** The partition-sweep gains came from main capacity, not from ghost growth (cycle 2 isolated this).\n- **Widening ghost evidence only slightly, without loosening re-admission.** Recording main victims in the shared, capped ghost (cycle 5) and the fingerprint ring's slightly wider window (cycle 6) both gave near-noise upticks.\n- **Policy and representation are separable.** Changes that keep behaviour fixed have been verified exact against reference implementations three times: the indexed slab/index/lazy ghost (cycles 3–4) and the fingerprint-ring ghost (cycle 6). Structural work can stack with policy work.\n- **One change per variant** made attribution clean from cycle 2 onward. The one bundled change (cycle 1's v002) could not be attributed.\n\n## Recurring failures\n\n- **Generous ghost re-admission.** It lost from every direction tested: freq credit (c2), tail insertion (c3), small-queue probation (c4). A bare ghost hit is enough evidence. Front-of-main at freq 0 is the settled entry point.\n- **Growing the ghost or adding ghost budget.** A decoupled or doubled ghost (c2) and a separate main-victim budget (c6) both lost. Every ghost hit bypasses the small filter and forces a main eviction, so the shared capped budget acts as churn protection.\n- **Tuning main credit beyond cap 7.** Cap 15 was flat (c4) and halving decay regressed (c5). Linear decrement at cap 7 is effectively optimal.\n- **Easier small→main graduation.** It regressed in c1 (bundled with a freq reset) and was flat in c5 (threshold alone). The ghost already recovers one-hit keys.\n- **Adaptive split driven by ghost-hit source (ARC-style).** It regressed (c6). The signal is biased toward small rejects and reacts after the re-admission has already happened.\n- **Leaving cost dimensions unmeasured.** Per-op-time and metadata-size have been requested every cycle and reported in none of the six. Every cost-motivated change (indexed port, ring ghost) and every cost risk (longer sweeps, extra ghost traffic, double hashing) is still unjudged.\n\n## Open questions\n\n- Does the indexed structure plus fingerprint ring actually cut per-op-time and metadata-size? Does the double hash on misses cancel out the savings? (Needs the evaluator to report these. This is the top priority.)\n- Do the accumulated choices (5% split, cap 7, ghosted main victims, ring ghost) hold on other traces and cache sizes, or are they fitted to workload-a at one size?\n- Was cap 15 or ghosting main victims a net loss once per-op-time is counted?\n- Only if a new trace shows headroom: a smaller combined ghost budget, a |B1|/|B2|-style adaptive signal, or a re-admission probation window based on access count.\n- An identity hasher on the ghost fingerprint map is a cheap cost-side follow-up.\n\n## Per-policy notes\n\n- **S3-FIFO (only policy explored).** Current lineage: 5% small, freq cap 7 with linear decrement, ghost hits re-admitted at front-of-main with freq 0, main victims recorded in the shared ghost, indexed structure with a fingerprint-ring ghost.\n- **S3-FIFO on workload-a at the current size** looks saturated. Partition, cap, decay, graduation, re-admission and ghost-sizing levers are all flat or negative. Further hit-rate tweaks on this trace are not worth proposing.\n- **S3-FIFO dynamics observed:** main evictions mostly take freq-0 ghost re-entries, so extra credit rarely changes which key is evicted. The small queue's window is much shorter than the ghost's reuse horizon, and few drained main victims return within the ghost's window.\n- **ARC-style adaptation layered on S3-FIFO** has been tried once and lost. The ghost-source signal is not a reliable controller on this trace.\n"
}
```

### 09:40:14 thinking



### 09:40:14 chat

I've written `cross-cycle-insights.md`, built only from the six-cycle ledger, with the four required sections in order.

The main takeaways for future cycles:

- **What works:** keeping keys that have shown repeated reuse in the main queue pays off. That means a smaller small queue (now 5%) and a frequency cap of 7 with plain decrement. Changes that keep behaviour identical, like the indexed port and the fingerprint-ring ghost, have been verified exact each time, so structural work can be stacked on top of policy work.
- **What keeps failing:** every attempt to let ghost hits back in more easily has lost (extra credit, tail insertion, small-queue probation). So have growing the ghost or giving it a separate budget, raising the cap past 7, decaying credit, easier graduation, and the ARC-style adaptive split.
- **Biggest gap:** per-op-time and metadata-size haven't been reported in any of the six cycles. None of the changes made for speed or memory, and none of the suspected costs, can be judged until the evaluator reports them.
- **Saturation:** S3-FIFO on workload-a at this cache size looks tapped out. Every remaining hit-rate lever is flat or negative, so the file points the next cycles at cost measurement and at testing other traces and cache sizes.

S3-FIFO is the only policy the ledger covers, so the per-policy notes are all about it plus the one ARC-style variant built on it.


---

- finished: 2026-10-21T09:40:14+00:00
- duration: 22.03s
- status: ok
- input_tokens: 4
- output_tokens: 2095
- cache_read_tokens: 38217
- cache_write_tokens: 11158
- cost_usd: 0.138823
