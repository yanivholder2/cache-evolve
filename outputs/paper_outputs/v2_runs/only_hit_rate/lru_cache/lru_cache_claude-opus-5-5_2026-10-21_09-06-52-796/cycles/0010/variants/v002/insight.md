# Variant v002 Insight

## Helped

(No measured dimension improved.)

## Hurt

- hit-rate: The only behavioural change from v001 is one line in `evict_one()`'s `main` branch: `node.freq -= 1` became `node.freq >>= 1`. The `small` branch, `MAX_FREQ = 7`, the saturating increments, the split and the ghost path are all unchanged. The extra code is a doc comment and a unit test. Halving does more than expire stale credit faster. It also caps how much credit a key that stays in use can hold. With decrement, a key hit about once per `main` rotation keeps or builds its credit toward 7, so it has a buffer for a later gap in reuse. With halving, the credit after reinsertion is `(f + hits) >> 1`, which settles near the per-rotation hit count. So a key with steady but sparse reuse sits one or two rotations from eviction at all times. Keys with credit 1 or 2 behave exactly as before. Only credit of 3 or more is cut, and that is where the loss shows up. Hit rate dipped slightly below v001. The dip is small enough to be noise on one trace at one size, but it is in the direction the hypothesis listed as "long, steady reuse needs linear aging".

## Takeaway

Making aging faster in `main` did not free useful slots on workload-a. The stale-credit cost that v001's insight flagged is either small or outweighed by the protection that linear decrement gives keys with long, steady reuse. Halving cuts that protection for every key, not just cold ones. Faster aging alone does not fix the problem. An aging rule that leaves keys hit since their last pass alone would be needed to target only the stale ones. Together with the tiny gain in cycle 9, this suggests the credit level and aging rate in `main` are not where the remaining hit rate is. v001's untested lead is still open: ghost-path admissions enter `main` with `freq = 0`.
