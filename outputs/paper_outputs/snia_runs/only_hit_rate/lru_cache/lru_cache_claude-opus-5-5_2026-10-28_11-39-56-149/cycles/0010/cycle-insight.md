# Cycle 0010 Insight

## Cohort outcome

All three variants built on cycle 0009's v002 and changed the frequency sketch. v001 raised the small-size `reset_at` to 64 × capacity. v002 raised the large-size `reset_at` to 32 × capacity. v003 switched to conservative update at every size, with no size gate. v003 is the Winner, but its gain is small and uneven: web_0 @ 16384 rose, and the other three cells slipped slightly.

## Patterns

- Sketch memory length has peaked. At small size, going from 32 × to 64 × produced equal and opposite moves at noise level. At large size, 32 × hurt both traces. The useful aging period seems bounded in absolute access count, not proportional to capacity. Small size is already near its best, and large size should stay at 16 × or shorter.
- Two variants independently named doorkeeper fill as the likely cause. The doorkeeper is cleared only on reset, so a longer period lets it saturate. One-shot keys then get a +1 from false positives. That links the c8 doorkeeper finding to the aging period: the two cannot be tuned separately.
- Admitting on ties is the hidden hinge. Conservative update did not fix inflated one-shot keys, because the doorkeeper already holds them at the floor. It lowered victim scores instead. That helped long-reuse web_0 at small size and loosened hm_0's scan filtering at both sizes. Any change that lowers victim estimates is really a change to admission strictness.
- For the first time in several cycles, the Winner is not size-gated. It is a trade (web_0 gains, hm_0 pays everywhere), not a clean single-axis gain. Its large-size cells moved for reasons unrelated to the stated rationale.
- Large size has still found no lever that helps both traces together. Longer memory joins the step, split, damping, split climber and victim selection on the list of failed levers.
- For the tenth cycle, only hit-rate was measured.

## Open questions

- Would a stricter tie rule recover hm_0's loss while keeping v003's web_0 gain? For example, require the candidate to strictly beat the victim when the victim's count is above 1, or apply strict-beat only to the doorkeeper-only candidate.
- Should conservative update be gated to small size only? Large-size web_0 got nothing from it and large-size hm_0 lost.
- Would clearing the doorkeeper on its own shorter schedule, separate from `reset_at`, let a long sketch memory stop leaking one-shot +1s?
- Is a shorter large-size `reset_at` (for example 8 ×) the direction that actually helps?

## Continuation

Use cycle 0010 v003 as the parent, but treat its gain as fragile. Next, try one of two single-lever changes: tighten the duel's tie rule (strict-beat for warm victims), or gate conservative update to small size. Separately, try clearing the doorkeeper on its own schedule, or a shorter large-size `reset_at`. Drop any further increase to `reset_at` at either size. `win_cap`, `metadata-size` and `per-op-time` are now ten cycles overdue.
