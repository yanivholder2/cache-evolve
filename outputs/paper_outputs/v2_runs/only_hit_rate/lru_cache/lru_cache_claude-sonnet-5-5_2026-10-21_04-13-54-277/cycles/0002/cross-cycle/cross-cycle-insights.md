## Recurring wins

- Scan resistance is what pays off. Both winners (segmented LRU in cycle 1, CLOCK second-chance in cycle 2) gained hit-rate by departing from strict LRU, and the invariant gate accepted both.
- Variants that stay strict-LRU-equivalent are neutral on hit-rate, so they cannot win while hit-rate is the only measured axis.
- A large, stable protected set has been the useful direction in the segmented design.

## Recurring failures

- Weakening the protected segment (a smaller share, or unconditional ghost-to-protected admission) lowered hit-rate.
- Cheaper-layout and faster-path claims (compact nodes, fast hot paths, CLOCK's cheap hits) have never been verified. Metadata-size and per-op-time have been missing from every fitness vector for two cycles, so those variants can't be judged.
- Every result comes from one scan-heavy trace at one cache size, and the gains and losses are small. That is weak evidence.

## Open questions

- Is the missing metadata-size and per-op-time measurement a harness gap or a configuration issue? Fix or confirm this first.
- Do the CLOCK and segmented gains hold on other traces and cache sizes?
- What do the segmented design's bookkeeping and CLOCK's bits cost in memory and latency?
- Can CLOCK be extended with frequency or ghost information? Can the compact-layout and fast-path ideas combine with the winners?
- Does a protected share above the current split help further? Would a gentler ghost policy (probation MRU insertion, or adaptive sizing as in ARC) help?

## Per-policy notes

- Strict LRU: micro-optimizations are neutral on hit-rate and unmeasured on cost. Don't propose more until size and speed are measured.
- Segmented LRU: it improved hit-rate. Refinements that shrink the protected segment or admit ghost hits directly to protected hurt. If revisited, try a larger protected share or a gentler ghost policy.
- CLOCK second-chance: it won a small hit-rate gain by inserting unreferenced and using a second-chance bit. Its size and speed advantages are claimed but unverified. It is the current base to build on.
