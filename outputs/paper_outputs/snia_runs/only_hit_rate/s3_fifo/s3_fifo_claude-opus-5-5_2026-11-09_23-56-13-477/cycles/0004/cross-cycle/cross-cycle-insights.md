## Recurring wins

- Strict admission to main holds up. The baseline two-reuse graduation rule plus freq reset has survived four cycles. Every attempt to loosen it (lower threshold, occupancy gate, ghost-rate switch) either fell back to it or lost.
- Per-key evidence beats global signals. The first ghost-path change to beat its parent (cycle 4) gated on each key's own small-queue freq at ghost time. Global rate proxies never won.
- Filtering inside small is cheaper than filtering at main admission. Cycle 4's freq-gated ghost path and its freq-1 small-tail relap both pointed this way.
- Cost-only refactors (O(1) index/slab, fast hasher, compact `Loc`) keep hit-rate bit-identical and port cleanly onto policy winners. Oracle tests enforce this.
- Cycle winners have often been the variant that didn't regress, not one that clearly improved. Policy gains stay small.

## Recurring failures

- Tuning a single constant (threshold or static small/main split): hm_0 and web_0 move in opposite directions, and split effects are near noise and flip sign across trace and size.
- Main occupancy as an adaptive signal: main fills and stays full, so the gate is dead in steady state.
- Ghost-derived global signals (windowed ghost-return rate, unweighted ARC-style ghost-hit split): the ghost is several times larger than small and mostly holds weak single-reuse keys, so the signal is inflated and fires on the wrong traces.
- Extra protection inside a full main (second-chance credit for ghost returnees): it keeps dead keys for an extra lap and evicts entries that would have been hit.
- Per-op-time and metadata-size are still absent from the fitness vector after four cycles. Two cost variants are waiting on measurement, and hypotheses keep assuming these dimensions are reported when they aren't.

## Open questions

- Do the freq-gated ghost path and the freq-1 small-tail relap stack, or do they compete for small slots?
- Can freq-0 ghost returnees recover hm_0 without giving back web_0? Options: route them by main tail freq, or give them a stronger initial credit in small.
- Should the second reuse count only within a recency window? Widely spaced reuses look like weak evidence.
- Would directly comparing the misses a promotion avoids against the main hits it displaces beat every proxy tried so far?
- How large are the actual per-op-time and metadata-size deltas of the O(1) index and the hasher/layout changes? Is the extra ghost-scan cost real?
- Has the policy essentially plateaued on these traces, so that remaining effort should go to cost and metadata?

## Per-policy notes

- S3-FIFO (the only policy evolved so far): the current lineage is the O(1) index, the occupancy-gated two-reuse rule (in practice the baseline rule) and the freq-gated ghost→main path. The pending cost changes from cycle 4 are still to be ported onto it.
- Policy effects show up almost only at the small cache size. At the large size the ghost covers a long reuse window and admission is cheap, so rerouting between ghost, small and main barely changes evictions.
- Trace character is consistent: web_0 penalises any loosening of main admission (exactly-twice keys pollute main) and gets little value from ghost returns. hm_0 sometimes has genuine ghost-return reuse and gained from looser admission only at the small size or cold start.
