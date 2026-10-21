# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:30:32+00:00

## timeline

### 09:30:49 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Admission filtering matters more than how recency is tracked.** S3-FIFO-style admission brought the big hit-rate gain in cycle 1. Every winner since then has been a descendant of that S3-FIFO.\n- **Shrinking the ghost has paid off.** The window was halved twice, first from near capacity to half and then to a quarter. Both steps raised hit rate, though the second gain was smaller. Ghost size is the most effective tuning knob found so far.\n- **The fixed probation split of about 10% has held up.** Moving away from it, whether adaptively or by enlarging it, never helped.\n- **The best results came from changing one knob per variant from a strong parent.** Those cycles gave clear answers. Cycles that changed policy and representation together gave confounded ones.\n\n## Recurring failures\n\n- **Only hit rate has been measured, for four cycles running.** Every memory or speed claim is still unverified. That includes the smaller nodes, the fingerprint ring, the smaller ghost's savings, the cost of the sketch, and LIRS's overhead. This is the largest blind spot in the search.\n- **Loosening admission has always cost hit rate.** Examples are a longer ghost window, ARC-style adaptive sizing, and fast paths from the ghost into `main`. Any change that lets keys into `main` without evidence of reuse loses.\n- **Admission that is too strict also fails.** Two-hit promotion bars and routing ghost hits through `small` either did nothing or regressed. With a small probation queue, they evict keys whose reuse gap is only slightly longer than that queue.\n- **Admission gates applied only to the ghost path leak.** Rejected keys get back into `main` through the other path, so stricter promotion bars and frequency gates applied only to ghost hits did little. Gates have to cover every route into `main` and compare against the real eviction victim.\n- **The representation-only experiments keep being put off.** The fingerprint ring at a fixed ghost size and the graft of the hasher plus narrow indices have been recommended since cycle 2 and never run.\n\n## Open questions\n\n- **Where does hit rate turn over as the ghost shrinks?** It could be at an eighth of capacity or lower. Does the best size hold at other cache sizes? So far only one has been tested.\n- **Can the harness be made to report `metadata-size` and `per-op-time`?** Until it can, the memory and speed hypotheses cannot be judged.\n- **Do the representation changes stack on the current S3-FIFO parent without affecting hit rate?** These are the Fx hasher, `u32` links and the fingerprint ring. What speed or memory gain do they give?\n- **Would TinyLFU-style admission against the real victim, applied to all insertions, pay back the cost of its sketch?**\n- **Which part of cycle 4's routing variant caused its regression?** It changed both the routing of ghost hits into `small` and the two-hit bar, and the two were never tested separately.\n\n## Per-policy notes\n\n- **S3-FIFO:** the lineage that keeps winning. The levers that matter, in order, are ghost size (smaller has been better so far), then keeping the probation split at about 10%. The sequence-number fix for ghost bookkeeping, which stops stale entries erasing newer ones, is a correctness change and should be carried forward. Ghost hits are a useful reuse signal when the window is short but misleading when it is long.\n- **CLOCK:** roughly matches LRU's hit rate, so exact LRU ordering is not needed for hit rate. Worth revisiting only if a memory or latency win is measured, and the risk of a long victim sweep on `put` has not been checked.\n- **LIRS:** beats LRU but trails S3-FIFO, and is heavier because of a larger node and a linear walk to enforce its bound. A larger HIR queue made it slightly worse. Set it aside until its costs can be measured and the walk is made O(1).\n- **ARC-style adaptivity:** drifted the split away from its best value and lost hit rate. Drop it.\n- **Strict LRU with representation tweaks (control):** leaves hit rate unchanged, as designed. These tweaks remain candidates for grafting onto other policies but have not been measured.\n"
}
```

### 09:30:56 thinking



### 09:30:56 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle directory, built only from the four-cycle ledger and using the four required sections in order.

The main points across the four cycles:

- **Wins:** S3-FIFO's admission filtering has been the winning line since cycle 1. Halving the ghost window twice raised hit rate both times, though the second gain was smaller. The fixed probation split of about 10% has held up. Variants that changed one knob from a strong parent gave the clearest answers.
- **Failures:** Only hit rate has been measured in all four cycles, so no memory or speed claim has been checked. Loosening admission always cost hit rate. Making it too strict (two-hit bars, routing ghost hits through `small`) also failed. Gates placed only on the ghost path let rejected keys back into `main` another way. The representation-only experiments recommended since cycle 2 have never been run.
- **Open questions:** where hit rate peaks as the ghost shrinks below a quarter of capacity, and whether that holds at other cache sizes. Also open: getting `metadata-size` and `per-op-time` measured, whether the hasher, narrow-index and fingerprint-ring changes stack on S3-FIFO, whether TinyLFU checked against the real eviction victim pays for its sketch, and which change caused cycle 4's routing regression.
- **Per policy:** keep S3-FIFO as the main line and carry forward its ghost bookkeeping fix. Revisit CLOCK only if it shows a measured memory or latency win. Set LIRS aside until its costs can be measured. Drop ARC-style adaptive sizing. Keep the strict-LRU control's representation changes as candidates to graft onto other policies.


---

- finished: 2026-10-21T09:30:56+00:00
- duration: 24.36s
- status: ok
- input_tokens: 4
- output_tokens: 2144
- cache_read_tokens: 36853
- cache_write_tokens: 9726
- cost_usd: 0.128075
