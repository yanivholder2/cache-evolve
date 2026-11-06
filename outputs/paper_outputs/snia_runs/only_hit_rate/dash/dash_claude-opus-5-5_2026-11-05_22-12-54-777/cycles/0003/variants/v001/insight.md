# Variant v001 Insight

## Helped

- hit-rate (hm_0, size 16384): An item promoted from stash to target now lands at the MRU end (`promote_pos = Back`), not at index 0. So it sits outside the 8-slot LRU-tail victim window until older residents age past it. Under heavy pressure on hm_0's small cache, that protection keeps blocks that were just reused long enough to hit again. This is the one cell the hypothesis predicted would gain. The gain is small. At size 131072 hm_0 barely moved, because the target bucket is rarely full enough for the landing position to matter.

## Hurt

- hit-rate (web_0, both sizes, and the summary): MRU landing shields every stash→target arrival from the frequency-aware tail scan, even items with a single reuse. That pushes the tail scan onto established target residents, which may have higher `lfu_counter`s. The item also lands next to the MRU end, where the parent's comments say the promotion candidate sits ("one more access promotes to the next tier"). So a second hit can move it straight into probing. The demotions that cascade out of probing then land at the LRU end, as before. On web_0, one-or-two-touch blocks churn the normal tier and displace blocks with more reuse. The loss is larger at size 131072, the cell where the parent's stable tier was most valuable.

## Takeaway

The diff only splits `insert_pos` so that `promote_to_target` uses `Back`. Probing landing, demotion and the trigger are unchanged. Even so, web_0 still regresses. So MRU landing on the stash→target hop is not neutral on top of v002's frequency filter. It weakens that filter by moving new arrivals out of the scan window, and it shortens the path to probing. The two LRU-end placements complement each other: Front landing is what makes a promoted item prove itself under the 8-slot tail scan. Future variants should keep stash→target at `Front`. If hm_0 at size 16384 needs more protection for new arrivals, a better approach is a counter-gated landing position (`Back` only when `lfu_counter` > 1) than a blanket move to the MRU end.
