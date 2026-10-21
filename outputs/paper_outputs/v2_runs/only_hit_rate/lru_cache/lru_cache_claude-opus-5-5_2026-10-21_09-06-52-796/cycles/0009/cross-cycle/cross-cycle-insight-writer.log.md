# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-21T14:43:21+00:00

## timeline

### 14:43:40 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (cycles 1–9)\n\n## Recurring wins\n\n- **Strict admission filtering is the main hit-rate lever.** S3-FIFO-style probation beat LRU, CLOCK and LIRS in cycle 1, and every later winner kept it. When admission was loosened (cycles 2 and 3), hit rate dropped.\n- **Tightening the admission window paid off many times, by less each time.** The ghost was halved from about capacity down to an eighth (cycles 3–5), and then `small` was shrunk while `small + ghost` stayed the same size (cycles 6–8). Both followed the same curve of shrinking gains and are now flat.\n- **Ghost and `small` work as one combined admission window.** A ghost is only worth having when it is larger than `small`. Moving resident probation slots into ghost keys is cheap and frees slots for `main`.\n- **Changes that only touch representation can be applied safely.** The fingerprint-ring ghost reproduced the parent's admission decisions exactly in cycles 5 and 9. It is a safe base for later variants.\n- **Changing one knob per variant gives clean answers.** Variants that changed two things at once (for example the cap plus carryover) left it unclear which change caused the result.\n\n## Recurring failures\n\n- **Paths into `main` that skip proof of reuse while resident.** Examples are a longer ghost window, ARC-style adaptive `small`, and a fast path from the main-ghost. Each one admitted false positives.\n- **Admission that is too strict.** Two-hit promotion bars and forcing ghost hits back through `small` drop keys whose reuse gap is only slightly longer than the tiny probation queue.\n- **Gates applied only to the ghost path.** Frequency gates on the ghost path leak through `small`. Even full TinyLFU admission against the real victim gained only a hair, because the existing filter already does that job.\n- **Giving ghost arrivals a starting credit.** This lost in cycle 7. Variant insights keep proposing it again, but treat it as rejected.\n- **Changes to how `main` retains keys.** Raising the credit cap, `small`-to-`main` carryover, SIEVE scan order, and a `main`-fed ghost all gave marginal or negative results. `main`'s full-rotation CLOCK grace period is useful protection for zero-credit arrivals, not waste.\n- **Gaps in evaluation that persist.** `metadata-size` and `per-op-time` have been unscored for all 9 cycles. All results come from one trace at one size (workload-a at 2048). A second size has been requested since cycle 4. Hypotheses that claim memory or speed wins remain unverifiable.\n\n## Open questions\n\n- Does the fingerprint ring actually lower `metadata-size`? Does its extra hashing raise `per-op-time`? How do the S3-FIFO variants compare with the baseline on both?\n- Do the settled split (`small` between about a fortieth and an eightieth, with the combined window fixed) and the small credit gains hold at a second cache size or on a second trace?\n- Are the hasher and narrow-index changes (cycle 1, v003) independent of policy, and can they stack onto the current S3-FIFO lineage for speed? This has still not been tested.\n- If hit rate is saturated on workload-a at 2048, is there any headroom left outside admission and retention? Or should the next cycles focus entirely on cost dimensions?\n\n## Per-policy notes\n\n- **S3-FIFO (main lineage):** This lineage won every cycle after the first. Current shape: a tiny `small`, a ghost larger than `small` with fixed combined size, CLOCK-style `main` with zero-credit entry, a cap of 7 plus carryover (marginal), and an optional fingerprint-ring ghost. Hit-rate tuning on workload-a is saturated.\n- **LRU / representation control:** Hit rate was unchanged, as expected. Its hasher and `u32`-link ideas were never measured or grafted.\n- **CLOCK:** Hit rate roughly matched LRU, so exact LRU ordering is not needed. Its claimed cost wins were never measured, and the line was dropped.\n- **LIRS:** It beat LRU but trailed S3-FIFO. Enlarging the HIR fraction hurt. It also has a heavier node and a linear bound-walk. It is set aside until costs can be measured.\n- **ARC-style adaptive sizing:** Drifting away from a fixed small probation split lost hit rate. Dropped.\n- **SIEVE (as `main`'s scan):** Demoting zero-credit arrivals faster hurt workload-a. Dropped.\n- **TinyLFU admission:** Correct in form (all routes, real victim) but near its ceiling. It has an unmeasured per-access cost. Revisit only if cost turns out to be close to zero.\n"
}
```

### 14:43:46 chat

I wrote `cross-cycle-insights.md` in the cycle 0009 cross-cycle output folder. It uses the four required sections, rebuilt from the ledger alone.

The overall picture: on the one trace and cache size tested (workload-a at 2048), hit-rate tuning for the S3-FIFO line has run out of room. The real limit is now what the harness measures, not the eviction policy.

- **What kept working:** strict admission filtering. Shrinking the probation window (the ghost first, then `small`, with their combined size held fixed) helped repeatedly, but by less each cycle, and has now flattened out. The fingerprint-ring ghost was confirmed twice to leave hit rate unchanged, so it is a safe base for later variants.
- **What kept failing:**
  - Admission that is too loose, and admission that is too strict.
  - Frequency gates applied only to the ghost path.
  - Giving ghost arrivals a starting credit. This lost in cycle 7, but variant write-ups keep suggesting it again.
  - Changes to how `main` keeps entries: a higher credit cap, carrying credits over from `small`, SIEVE scan order, and a ghost fed by `main` evictions.
- **Biggest gaps:**
  - Memory size and per-operation time have never been scored in any of the nine cycles.
  - No result has been run at a second cache size, although one has been requested since cycle 4.
  - The hasher and narrow-index changes from cycle 1 were never grafted onto the S3-FIFO line or measured.
- **Per-policy notes:** these cover S3-FIFO, LRU, CLOCK, LIRS, ARC-style sizing, SIEVE and TinyLFU.


---

- finished: 2026-10-21T14:43:46+00:00
- duration: 25.17s
- status: ok
- input_tokens: 4
- output_tokens: 2330
- cache_read_tokens: 41564
- cache_write_tokens: 15914
- cost_usd: 0.182241
