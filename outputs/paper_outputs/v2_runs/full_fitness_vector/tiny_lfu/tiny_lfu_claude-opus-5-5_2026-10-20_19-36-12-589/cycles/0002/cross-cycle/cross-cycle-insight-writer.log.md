# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T19:50:54+00:00

## timeline

### 19:51:11 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/tiny_lfu_2026-10-20_19-36-12-589/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Pure efficiency and layout changes that add no hot-path work are the only reliable wins so far. Cheap hashing (cycle 1) and compact `u32` link indices (cycle 2) both improved cost and left behaviour unchanged.\n- The lineage moves forward by layering efficiency gains: baseline, then fast hash, then compact links. Each base keeps the hit rate of the one before it.\n\n## Recurring failures\n\n- Hit rate has not moved in any cycle. Every attempt to raise it fell flat or regressed: window+SLRU structure, sketch aging, and frequency-aware victim sampling.\n- Adding structure or bookkeeping to the per-op path costs latency right away, and it has never paid back in hit rate. Examples are region tags, per-hit relinking, per-op aging branches, and extra hashing on misses.\n- Changing the frequency signal on its own has hurt. Aging with coarser counters made more admission ties, and the strict tie-goes-to-victim rule then rejected more good newcomers.\n- LRU-end victim sampling is ineffective. Estimates near the tail are nearly equal, so sampling there adds cost and gives no selectivity.\n\n## Open questions\n\n- Is the hit-rate ceiling coming from the admission rule (strict `>` comparison, ties going to the victim, no doorkeeper or window) rather than from the sketch or the list structure? Test changes to admission and tie handling, possibly together with aging.\n- Is workload-a already near its hit-rate ceiling for a TinyLFU-family design? If so, efficiency is the only axis left to improve.\n- Is the latency regression on the compact-links base real or benchmark noise? Re-measure it side by side with the fast-hash base on the same machine.\n- Is the unkeyed fast hasher safe under adversarial or collision-heavy key sets?\n- Would a larger admission window help, once admission or freshness is fixed?\n\n## Per-policy notes\n\n- TinyLFU (sketch + LRU, the base lineage): hashing was the dominant per-op cost. The put tail is most sensitive because it runs the admission contest.\n- W-TinyLFU (window + SLRU): tried once at about a 1% window. It cost memory and latency and gave almost no hit-rate gain while the sketch never ages. Not worth revisiting until the frequency/admission signal is shown to matter.\n- Sketch aging (Caffeine-style reset, 4-bit cap): first blocked by an infra failure, then hurt hit rate when evaluated alone. Only worth retrying alongside admission or tie-rule changes, with the reset bookkeeping kept off the hot path.\n- Sampled-victim eviction: no hit-rate effect and extra per-miss cost. Dropped.\n"
}
```

### 19:51:18 chat

I've written `cross-cycle-insights.md` to the cross-cycle output dir. It's rebuilt from the cycle 0001 and 0002 ledger only, using the four sections in the order you set.

- **Recurring wins:** Hit rate hasn't changed in either cycle. Only changes that make the code cheaper without changing behaviour have helped: faster hashing in cycle 1 and smaller list links in cycle 2.
- **Recurring failures:** Window plus SLRU, sketch aging, and picking the victim from a sample near the LRU end all failed to raise hit rate. Each also slowed per-op time by adding hot-path work. Aging made hit rate worse because coarser counts mean more admission ties, and ties go to the victim.
- **Open questions:**
  - Is the admission rule what caps hit rate? That's the strict "newcomer must beat the victim" comparison with no doorkeeper or window.
  - Is workload-a already near its hit-rate ceiling?
  - Is the latency regression on the compact-links version real? Re-measuring it on the same machine as the fast-hash version would settle it.
  - Is the unkeyed fast hasher safe against collision-heavy keys?
- **Per-policy notes:** Grouped by base TinyLFU, W-TinyLFU, sketch aging, and sampled-victim eviction, saying for each whether it's worth retrying and under what conditions.


---

- finished: 2026-10-20T19:51:19+00:00
- duration: 25.04s
- status: ok
- input_tokens: 4
- output_tokens: 1524
- cache_read_tokens: 34554
- cache_write_tokens: 6944
- cost_usd: 0.093264
