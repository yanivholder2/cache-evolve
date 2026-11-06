# Cycle 0010 Insight

## Cohort outcome

All three variants built on cycle 0009's v002 (the contention-gated ghost bypass). Two took up c0009's open questions on separating the sizes. v001 relaxed the gate only when few tail-window slots were warm, judged by the window's counter histogram. v002 gave a per-key "refused once" strike, so a second return could displace a counter-1 minimum. v003 tried a web_0 lever outside the ghost path: it put sequential-run inserts at the stash's LRU end and suppressed their ghost entries. v002 was the Winner for the second cycle running only because it did no harm. Every cell moved at noise level, and large-size web_0 is still mostly unrecovered.

## Patterns

- The window's counter histogram cannot tell large-size web_0 from small-size hm_0. In both, the tail windows are mostly counter-1. So v001's warm-count guard fired in both places and recreated the c0009 v001 trade-off exactly: large web_0 up, small hm_0 down, mean flat. Any gate that reads only the target window will just move along this trade-off.
- Per-key repeat-refusal evidence is real but too rare to count (v002). The valuable large-size web_0 returners do not mostly go through refuse → stash → drop → ghost → return. Either they return only once within ghost reach, or they get lost in the stash before they fall back to the ghost.
- Sequential web_0 blocks are not one-shot pollution. They are among the ghost's beneficiaries. v003 removed both their stash residency and their ghost entry, and lost hits on web_0 at both sizes and on hm_0 too. This confirms that every web_0 gain in the run has come through ghost readmission, and that removing a key's ghost entry costs hits.
- The ghost/bypass gate family has now stalled for two cycles. Three different gate signals have been tried: a global threshold, window occupancy, and per-key strikes. None improved both sizes.
- For the third straight cycle, overlay checks failed for two of the three variants, so their mechanisms are inferred, not confirmed. No variant has branch-firing counts. Per-op-time and metadata-size are still unmeasured, now for ten cycles.

## Open questions

- What signal about the returner itself separates the two cases? Candidates are how recently its ghost entry was written (its ghost age) or whether it was ever hit in target. A window-only or repeat-only signal does not.
- Do large-size web_0 returners mostly die in the stash before reaching the ghost? If so, the lever may be stash retention for ghost-admitted keys, not the target gate.
- Does v003's scan detector help if only one lever is used: either LRU-end insertion with ghosting kept, or normal insertion with ghost suppression?

## Continuation

Keep cycle 0010 v002, or the equivalent 0009 v002, as the parent. Drop further window-histogram gate variants and the refused-strike credit. Drop sequential-run filters that remove ghost entries. If the gate is revisited, use a returner-side signal such as ghost-entry age. Before building on that signal, first add instrumentation: gate-branch firing counts, and how stash-refused returners end up. Another option is to treat the ghost/bypass line as saturated and try a different axis. Variant insights must confirm the overlay landed, and they must report per-op-time and metadata-size.
