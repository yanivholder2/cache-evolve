# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005's v002 (the age-gated ghost path), and all three changed its ghost or small routing. v001 restacked the one-time relap, this time ghosting failed relaps at freq 1. v002 widened the freq-0 recency window to two small-queue turnovers. v003 added an age ceiling on freq ≥ 1 returnees. v003 won, but only because it moved the least: it rose slightly on hm_0 at the large size and dipped slightly in every other cell. None of the three cleanly beat the parent, so this cycle mapped where the policy's boundaries are rather than finding a new gain.

## Patterns

- The parent's ghost gate is close to a local optimum on all three axes probed. Stacking a second reward for one reuse (v001), loosening the recency floor (v002) and tightening admission for freq-1 returnees (v003) each gave back hit-rate on web_0, which is the trace the parent's gate was tuned against.
- The two signals do different jobs. Ghost age is useful as a floor on freq-0 returnees, and about one small-queue turnover is the right boundary: going to two turnovers lets in web_0's late one-off rereferences. Age adds little as a ceiling on freq-1 returnees, because a reuse in small is still good evidence late in the ghost window.
- Once failed relaps keep their freq-1 ghost signal, the relap duplicates what the ghost path already does, and it costs small-queue capacity. The cycle 0005 relap gain on web_0 at the large size came from the old freq-0 base and did not carry over.
- web_0 at the large size has now resisted every ghost and small routing change, whether it loosened or tightened admission. Whatever limits that cell, it isn't how ghost returnees are routed.
- Every small-queue re-admission (relap, re-routed returnee) pushes another resident out of small. At the small cache size this displacement keeps costing more than whatever main pollution it avoids.
- Per-op-time and metadata-size are still absent from the evaluation, for the sixth cycle running.

## Open questions

- What actually limits web_0 at the large size: main's eviction order, the small/main split at that size, or first-admission filtering of keys never seen before?
- Would a window narrower than one turnover sharpen the web_0 / hm_0 split, or is one turnover already at the boundary?
- Is hm_0's modest large-size gain under v003 real, or is it noise?

## Continuation

Keep cycle 0005's v002 policy, with the age window at 1×, as the policy base. Drop relap stacking, windows wider than one turnover, and age ceilings on freq ≥ 1 returnees; the ghost-path routing axis looks exhausted. If policy work continues, move off the ghost path and target main's eviction or first-admission handling for web_0 at the large size, and try at most one narrower-window probe. Otherwise, port the pending cost-only changes and push for per-op-time and metadata-size to be measured.
