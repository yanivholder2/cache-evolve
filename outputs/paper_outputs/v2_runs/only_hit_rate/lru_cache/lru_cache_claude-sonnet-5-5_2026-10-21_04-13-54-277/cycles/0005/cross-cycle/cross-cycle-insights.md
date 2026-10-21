## Recurring wins

- Scan resistance is the lever that pays. Segmenting the LRU, then CLOCK with an unreferenced-insert bit, then an admission gate each produced a small hit-rate gain.
- Growing the protected share helped monotonically at first (smaller lost, larger won), then flattened out past about 90–95%.
- Each cycle's winner was a simple, single-mechanism change. Gains are always small.

## Recurring failures

- Strict-LRU micro-optimizations (compact layout, fast paths) can't win. They match the parent's hit-rate and their cost claims are never measured.
- Tweaks to the protected side (smaller share, second-chance on protected eviction, relink-skip) either lost hit-rate or matched the parent. The protected side is saturated.
- Probation-side tweaks (placement of demoted nodes, frequency-aware scans, probation-MRU ghost) are inert. Demotions are rare at a high protected share. One frequency variant was dead by construction because its counts never persisted.
- Unconditional ghost-to-protected admission hurt hit-rate. Gentler ghost variants only recovered to parity.
- Ideas judged on predicted size or speed can't be selected. Metadata-size and per-op-time have been missing from every fitness vector for five cycles.

## Open questions

- Is the missing metadata-size and per-op-time measurement a harness fault? It blocks every cost-motivated idea (CLOCK's advantage, sketch cost, relink-skip) and is the top blocker.
- Do the gains hold on other traces and cache sizes? All evidence comes from one scan-heavy trace at one size. Long-distance-reuse and low-scan traces are untested.
- Would adaptive segment sizing (ARC-style) beat any fixed share?
- Can CLOCK or frequency ideas combine with the segmented design? Would frequency that persists across segments help?
- Is the admission gate worth its cost? Cheaper forms include not counting rejected misses, or a tiny doorkeeper.

## Per-policy notes

- Segmented LRU: protected share is tuned out at about 95%. Protected-eviction and probation-placement refinements are exhausted on this trace.
- CLOCK (1-bit): small hit-rate gain from scan resistance without list manipulation. Its size and speed benefits are unverified.
- Frequency CLOCK (2-bit): only an incremental gain over 1-bit CLOCK.
- Admission gate (sketch): marginal gain on top of segmentation. Its hashing, counter and decay costs are unmeasured, and it can reject new hot keys.
- Ghost lists: harmful when they promote directly, neutral when gentle. They are worth revisiting only with long-reuse traces.
- Strict LRU: no further micro-optimizations until cost metrics are measured.
