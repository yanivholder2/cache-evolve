# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Raising `PROMOTE_THRESHOLD` from 2 to 3 on top of v003's MRU stash re-entry (`DEMOTE_TO_STASH_POS = Back`) did not move hit-rate at all. On the single LIRS workload-a trace at size 2048, the result matches the parent to every reported digit, so the cycle-2 admission and retention gains did not stack. They also did not conflict: MRU-protected ex-residents did not visibly crowd out fresh keys. A result this exact is suspicious. Either almost no fresh key on this trace sits between its second and third stash hit, because demoted ex-residents carry the `PROMOTE_THRESHOLD - 1` pre-credit and dominate promotions, or the threshold change never reached the evaluated build. The overlay source could not be inspected while writing this insight, so check that the overlay really sets `PROMOTE_THRESHOLD = 3` before treating this as a real "no effect" result.
