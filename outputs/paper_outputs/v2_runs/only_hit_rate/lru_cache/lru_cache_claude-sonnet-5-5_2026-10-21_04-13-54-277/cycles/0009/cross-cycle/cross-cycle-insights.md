## Recurring wins

- Segmented LRU (probation + protected) is the one structural change that clearly paid off. Early cycles gained from it, and later gains came from tuning its protected share upward until the curve flattened.
- Simple designs stay competitive. The doorkeeper-free SLRU + sketch gate matched or beat more elaborate variants.
- Counting rejected misses in the sketch helps: recurring keys build frequency across retries and earn admission. Removing it hurt.
- Frequency that persists across segments gives frequency-aware decisions a real signal, but the gain is small.
- Winners in recent cycles are mostly marginal hit-rate edges, so the selection signal is thin.

## Recurring failures

- Cheaper-per-op and smaller-metadata ideas (compact layout, fast paths, relink-skip, 4-bit sketch) cannot be judged. Metadata-size and per-op-time have been missing from every fitness vector in every cycle.
- Tuning single constants past their optimum is inert or slightly negative. This covers protected share, aging window (bracketed from both sides), admission margin, and doorkeeper reset.
- Probation-side tweaks (demotion placement, probation-only eviction scans, victim scans) are nearly inert because demotions are rare at a high protected share. One variant was dead by construction because frequency was never persisted.
- Ghost lists and adaptive sizing either hurt (direct-to-protected) or match the parent. The trace shows little phase variance to exploit.
- Deferred admission (small window) and extra admission machinery add complexity without return, since entry-time gating is nearly sufficient.
- Every cycle's results rest on one scan-heavy trace at one cache size, so many "plateau" conclusions may be trace-specific.

## Open questions

- Is the missing metadata-size and per-op-time measurement a harness fault? It has blocked cost judgment for nine cycles and should be fixed before more hypotheses.
- Do any gains hold on other traces and cache sizes, especially ones with long-distance reuse or less scan pressure?
- Would a different axis help, such as an eviction signal based on size or cost, or frequency that spans protected entries?
- Does CLOCK's cheaper hit path offset its miss-time sweep and sketch updates in a head-to-head cost-aware comparison with the segmented incumbent?
- Is the compact sketch or the admission-margin tweak worth keeping once cost is measured, or are they noise?

## Per-policy notes

- Strict LRU: Micro-optimizations that preserve semantics cannot win on hit-rate and were unmeasurable on cost.
- Segmented LRU: Protected share rose to a plateau around the high-90s percent range. It is saturated on this trace. Smaller protected shares and unconditional ghost-to-protected admission lost.
- Segmented LRU + sketch gate: The doorkeeper-free version is the simplicity baseline. Aging window, doorkeeper, margin and counter precision are all within noise.
- CLOCK variants: These repeatedly report a hit-rate edge over strict LRU but have not beaten the segmented line, and frequency bits add only incremental gains. Their claimed size and speed advantages remain unverified.
- ARC-style adaptive sizing: Noise-level on this workload.
