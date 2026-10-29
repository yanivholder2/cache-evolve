# Cycle 0010 Insight

## Cohort outcome

All three variants branched from 0009 v003, the bounded stash-protection baseline. v001 was the identical-copy control, v002 stacked the halved `PROBE_WINDOW`, and v003 added a one-shot ghost credit at the stash gate aimed at web_0-large. All three are Winners. None of them produced a decisive gain. v001 confirmed the baseline. v002 and v003 each added a small, noise-level hm_0 hit-rate gain, and v003 failed its web_0-large target.

## Patterns

- The simulator is deterministic. The control matched its parent to the last digit in every cell, so any hit-rate delta on this lineage, however small, is real. That makes v002's and v003's small upward moves attributable to their changes. Latency drifted between sessions again, uniformly, which confirms the cycle 0009 rule.
- The control rule was followed, but nobody used the control. Both v002 and v003 compared latency to the cross-session parent and not to v001, even though v001 ran in the same session. Neither latency verdict is valid. The control only helps if the variant writer is handed its numbers.
- Levers on different paths do stack. Window plus protection kept the hm_0 gain intact and added a little. This is the first stacking test to pass, and it supports the cycle 0007 rule that levers fail to stack only when they sit on the same arm. The window still did nothing for get latency, so it is now inert on both hit-rate and latency.
- Every hm_0 gain so far comes from short-range reuse, and web_0-large is still untouched. Ghost re-admission helped hm_0 (keys returning within one ghost lifetime) and not web_0-large. That fails H03 by its own refutation criterion, so stash-gate admission joins stash exit, the probing arm and placement as levers that do not move this cell. The likely reading is that web_0-large is capacity-bound, although a 64-entry ghost ring may simply be too short-lived at 128K.
- Within-run evidence suggests v003 has a real put cost: put p99 moved up toward get p99, which fits its ghost scan on every stash insert. Whether that holds needs to be checked against v001.

## Open questions

- Compared against v001 in the same session, does v003's put p99 rise hold up, and is v002's put p99 jump just noise?
- Is web_0-large actually capacity-bound, or was the ghost too short-lived? A larger or longer-lived ghost (still off-Item) would settle that once, cheaply.
- Do v002 and v003 stack? They touch different paths (probing arm vs. stash insert), and both gained slightly on hm_0.

## Continuation

Keep 0009 v003, now confirmed by the control, as the baseline. Consider v002 as a free hit-rate add-on, but stop treating `PROBE_WINDOW` as a latency knob. The next cohort's harness or writers must score latency against the same-session control's numbers, not the parent's. Without that, the control rule has no effect. For web_0-large, run at most one last test with a longer-lived ghost. If that stays flat, declare the cell capacity-bound, stop targeting it, and spend hit-rate effort on hm_0 and the small caches.
