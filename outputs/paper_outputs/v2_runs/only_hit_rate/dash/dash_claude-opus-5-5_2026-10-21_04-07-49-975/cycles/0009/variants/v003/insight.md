# Variant v003 Insight

> Note: I could not inspect the overlay source in this session. Directory listing and search tools were unavailable, and the file paths I guessed under `attempts/01/overlay/src/dash/` did not exist. The mechanism below comes from H03's stated change: per segment, move one normal bucket's worth of slots into the stash, keep the total slot count the same, and leave all stash policy alone. I could not check how the layout constants, normal-bucket indexing (mask or modulo), or probing wraparound were actually changed.

## Helped

## Hurt

- hit-rate: The result is effectively flat with a negligible dip, not the gain H03 predicted. The added stash slots retained no more useful keys than the normal-tier slots they replaced. One possible reason: losing a normal bucket shrinks the Front-landed, frequency-coupled region and makes each hashed home bucket crowded, which sends more first-placement traffic into the stash. That traffic would use up the new stash capacity instead of extending the victim cache's evidence-filtered retention.

## Takeaway

At this cache size, one normal bucket's worth of capacity is worth about the same whether it sits in the normal tier or in the stash. So the claim that "a stash slot is worth more than a normal slot" is not supported, and the stash-to-normal ratio looks like a weak lever around the current split, not a main structural one. A flat result does not clearly call for the reverse shift that the rationale planned for a loss. Before more ratio steps are spent, check that the non-power-of-two normal-bucket indexing really spreads keys evenly. Uneven spreading could hide a small stash gain. This result comes from one LIRS trace at one cache size.
