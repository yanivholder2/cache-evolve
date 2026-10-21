## Recurring wins

- Static sizing of admission is what has paid off: a much smaller small queue (~5%) plus a shorter ghost. Together they give the main gain over baseline. Later cycles showed this sizing-only setup matches more complex bundles, so it is the preferred simple base.
- Ghost length acts as an admission filter: longer hurt, shorter helped. Small-queue shrinking gave the clearest gain of any single lever, but with diminishing returns.
- Adding information without adding protection or loosening rules (e.g. ghosting main evictees) is safe but near-neutral.
- Hit-path promotion (small to main when freq reaches 2) is the first new lever in many cycles to show a gain that is real in behaviour terms. Its gain is still confounded with bundled sizing.

## Recurring failures

- Loosening admission (looser promotion gate, longer ghost, pre-protected or boosted ghost re-admits, grace passes) lost every time. It lets weakly reused keys displace useful residents.
- Ageing and decay changes (faster freq halving, lower freq cap, in-place ageing) lost every time. This workload rewards keeping reuse history. A higher freq cap gave at most a noise-level gain.
- Rewrites of main-eviction mechanics (lookahead, reinsertion caps) lost. The unbounded second-chance reinsertion scan does real protective work.
- Filtering ghost hits or ghost content (freq>0 fast path, reuse-only ghosting) lost. Ghost hits are strong evidence and should stay permissive.
- Eviction-time promotion-gate strictness tied twice. It is either non-binding or a no-op, so it is not worth more cycles.
- Adaptive or feedback-driven sizing lost to the fixed ratio. Per-key ghost metadata and main insertion-position tweaks gave nothing.
- Most cycles' best variant was a tie or marginal gain, and many gains may be within noise.

## Open questions

- Is the hit-path promotion gain real once the parent is the sizing-only base? Which threshold and which main insertion position are best?
- What does hit-path promotion cost in `get` per-op time? Cost dimensions (metadata-size, per-op-time) have never been measured in any cycle.
- Do any gains hold on a second trace or cache size? All evidence is one trace (LIRS) at one cache size.
- Do the no-op-suspected variants (promotion gate, insertion position) actually change behaviour? Authors could not read overlay source.
- Is S3-FIFO at a local optimum for this trace, so that only structural hit-path levers remain?

## Per-policy notes

- S3-FIFO (the only policy so far): the base is baseline plus a small queue of ~5% and a shorter ghost, with simple second-chance main eviction unchanged.
- Small-to-main promotion at eviction time is insensitive to freq threshold; hit-path promotion is the untested-in-isolation lever.
- Ghost: shorter is better, content and selection filters do not help, and extra per-key state adds nothing.
- Main eviction: keep the parent's reinsertion semantics, and avoid ageing.
- Next cycles should use the sizing-only base as the explicit parent, change one hit-path lever at a time, validate on a second trace, and restore cost measurements.
