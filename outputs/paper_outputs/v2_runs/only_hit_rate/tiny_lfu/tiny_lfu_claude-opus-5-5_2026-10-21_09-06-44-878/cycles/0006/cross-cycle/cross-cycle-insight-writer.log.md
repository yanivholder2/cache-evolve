# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T09:35:19+00:00

## timeline

### 09:35:35 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-21_09-06-44-878/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Estimate quality has been the only lever with a clearly large hit-rate gain. Widening the sketch to remove collision overcount (cycle 3) beat every contest, sampling and structural tweak. Making counts honest by deduplicating the get-miss-then-put double increment (cycle 4) added a smaller gain on top.\n- Lifetime (non-aging) frequency is the right signal for workload-a. Every lineage that kept it intact did at least as well as the lineages that added freshness.\n- Strict `>` with keep-victim-on-tie admission has held up. Every attempt to loosen it lost hit-rate.\n- Frequency-aware victim selection (a small sample of the oldest entries, picking the one with the minimum estimate) gave small but repeated gains. It now acts as a soft protected segment.\n- Changes that preserve semantics (fast single hash, hash-cached fast path, narrower conservative-update sketch on the non-SLRU parent) reliably match the parent's hit-rate exactly. They are safe to stack once their cost axes are scored.\n\n## Recurring failures\n\n- Aging or decay failed in every form tried: short-period with conservative update, long-period alone, and very-long-period on a clean sketch. Freshness keeps losing to lifetime counts on this trace. Early \"stale-high\" blocking came from collision noise, not from old popularity.\n- Admission windows lost every time: a 1% window on a noisy sketch, an adaptive hill-climbed window, and a 1% window in front of SLRU on clean estimates. Few keys come back within a short recency trial, and the slots taken from the main region cost more hits than the window recovers.\n- Loosening the contest failed: tie-admission swapped one cold key for another. Widening the victim sample failed too, because on a non-aging sketch the lowest estimate marks the youngest resident, not the coldest.\n- Once the sketch was clean, tuning its accuracy stopped helping: conservative update on a wide sketch did nothing, because width and conservative update fix the same error.\n- Tweaks inside SLRU (demotion policy, window, narrow sketch) only rearrange entries. The resident structure is already over-defended.\n- Systemic: for six cycles running, per-op-time and metadata-size have not been scored. Memory and latency variants can therefore only tie or lose, and their real benefit stays unverified. This is now the main blocker on progress.\n\n## Open questions\n\n- Will the evaluator score per-op-time and metadata-size? If it does, which stack dominates once costs are priced: SLRU, the narrow CU sketch, the hash-cached fast path, or all three combined? What do SLRU's per-hit list work and per-promotion estimates cost?\n- Inside SLRU, could a recency tie-break between equal estimates let the sketch drop below 8× width without flipping admission or victim decisions?\n- Is the flat hit-rate on workload-a (unchanged across cycles 5–6 over policy, structure and sketch levers) simply this trace's ceiling for TinyLFU-family policies? Would a different workload change any of the conclusions about aging or windows?\n\n## Per-policy notes\n\n- Count-min sketch: width matters most. In plain TinyLFU it is over-provisioned (half width plus u16 counters plus conservative update was neutral). In SLRU with probation-only sampling, 8× looks like the floor, because leftover collision error flips some decisions. Keep dedup of the get-then-put increment.\n- TinyLFU admission contest: keep strict `>` with keep-victim-on-tie. A first-time newcomer still ties a resident that was accessed once and loses, but cycle 6 showed these lost ties are not where the remaining misses are.\n- W-TinyLFU (window + main): consistently negative on workload-a, whether fixed or adaptive. A naive climber drifts toward an LRU-like window, which this LIRS-style trace punishes.\n- SLRU probation/protected: a marginal win that largely duplicates min-estimate victim sampling. Demotion by frequency held hit-rate unchanged but adds unpriced estimate cost.\n- Fast-path shortcuts (early stop or early reject at estimate ≤ 1) are valid only without aging. Any decay or reset proposal must remove or re-derive them.\n"
}
```

### 09:35:41 chat

I've regenerated `cross-cycle-insights.md` from the cycles 1–6 ledger, using the four required sections in order.

The main conclusions:

- **What paid off:** making the frequency estimates more accurate was the only lever with a large hit-rate gain. Widening the sketch to cut collision noise was the big win, and removing the double count from the get-miss-then-put sequence added a smaller one. Counts that never decay, plus the strict `>` admission rule that keeps the current entry on ties, both held up throughout.
- **What kept failing:**
  - Every form of aging tried.
  - All the admission windows, whether fixed or adaptive.
  - Admitting the newcomer on ties, and widening the victim sample.
  - Further sketch-accuracy tuning once the noise was gone.
  - Tweaks inside the SLRU layout (the version with probation and protected segments).
- **Biggest blocker:** per-op-time and metadata-size have gone unscored for six cycles in a row. Variants built to save memory or time can therefore only tie or lose, however well they work.
- **Open questions:**
  - Which combination wins once costs are actually scored.
  - Whether breaking ties between equal estimates by recency would let the sketch shrink below 8× width inside SLRU.
  - Whether the flat hit-rate on workload-a is simply this trace's ceiling for this family of policies.
- **Per-policy notes:** these cover the sketch width floor, the tie rule, the windowed layout (consistently negative), SLRU, and a reminder that the fast-path shortcuts are only valid while there is no aging.


---

- finished: 2026-10-21T09:35:41+00:00
- duration: 22.70s
- status: ok
- input_tokens: 4
- output_tokens: 2097
- cache_read_tokens: 37970
- cache_write_tokens: 10867
- cost_usd: 0.136486
