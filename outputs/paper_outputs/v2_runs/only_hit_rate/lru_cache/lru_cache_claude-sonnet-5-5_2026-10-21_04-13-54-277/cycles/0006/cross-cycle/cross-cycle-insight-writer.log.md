# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:36:19+00:00

## timeline

### 04:36:28 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Segmented LRU (probation + protected) is the one design line that has consistently produced winners; scan resistance comes from keeping one-hit keys in a small probation segment.\n- Raising the protected share paid off in successive cycles, with diminishing returns, then a plateau.\n- Counting rejected misses in an admission sketch helps: recurring keys build frequency across retries and earn admission.\n- CLOCK variants (one-bit second chance) give small hit-rate gains over strict LRU without list manipulation.\n\n## Recurring failures\n\n- Weakening the protected segment (smaller share, ghost-to-protected re-admission) loses hit-rate.\n- Probation-side tweaks (demotion placement, frequency-aware eviction scans) are inert at a high protected share: demotions are rare, and frequency state was not persisted, so scans fell back to LRU order.\n- Protected-eviction refinements (second-chance bit) and approximate-recency shortcuts (relink-skip) are inert or slightly negative.\n- Admission-gate tuning (sketch update rule, doorkeeper, thresholds) has plateaued; segmentation already filters most scans.\n- Frequency-aware CLOCK and CLOCK + sketch are only incremental; they have not beaten the segmented line.\n- Many variants matched their parent exactly, meaning the changed code path rarely ran on the evaluated trace.\n- Metadata-size and per-op-time have been absent from every fitness vector in every cycle. Selection has been on hit-rate alone, with marginal differences, and all cost claims are unverified.\n\n## Open questions\n\n- Is the missing cost measurement (metadata-size, per-op-time) a harness fault? It blocks judging every variant and should be fixed first.\n- Do the gains hold on other traces and cache sizes, especially those with long-distance reuse or little scan pressure? All evidence comes from one scan-heavy trace at one size.\n- Would adaptive (ARC-style) segment sizing beat any fixed share?\n- Would frequency persisting across segments or covering protected entries help?\n- How does CLOCK + sketch compare head to head with the segmented winner, on hit-rate and cost?\n- Are the sketch gate and doorkeeper worth their added state and time?\n\n## Per-policy notes\n\n- Segmented LRU: saturated on this trace at about a 95% protected share. Stop tuning the share, probation placement, and protected eviction.\n- Admission gate (TinyLFU-style sketch, doorkeeper): marginal gain, plateaued; the cost is unmeasured. Prefer cheaper variants only if cost can be measured.\n- CLOCK (one-bit, frequency, + sketch): small gains. Its size and speed advantages are the main claim but are unverified.\n- Strict LRU: micro-optimizations were neutral on hit-rate and cannot win without cost metrics.\n- Ghost list: harmful when it admits directly to protected; neutral when it re-admits at probation MRU. Revisit only with a long-reuse trace.\n"
}
```

### 04:36:31 chat

I wrote `cross-cycle-insights.md` in the output dir, with the four required sections in order.

The two themes the file carries forward:

- **Cost metrics:** Metadata-size and per-op-time have been missing from every fitness vector in all six cycles. Selection has run on hit-rate alone, and every cost claim is unverified. The file lists this first among the open questions.
- **Saturation:** The segmented LRU line looks saturated at about a 95% protected share. Probation tweaks, protected-eviction refinements and admission-gate tuning are all inert or marginal. The file flags that all evidence comes from one scan-heavy trace at one cache size.

I did not read the previous insights file, and the file contains no numbers beyond the 95% share the ledger itself cites.


---

- finished: 2026-10-21T04:36:32+00:00
- duration: 12.55s
- status: ok
- input_tokens: 4
- output_tokens: 1399
- cache_read_tokens: 38181
- cache_write_tokens: 9873
- cost_usd: 0.057897
