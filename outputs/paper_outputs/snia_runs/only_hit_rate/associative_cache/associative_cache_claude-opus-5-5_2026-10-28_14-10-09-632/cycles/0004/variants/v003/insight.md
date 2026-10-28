# Variant v003 Insight

## Helped

- hit-rate (`web_0` at 131072, small): A first re-hit now lands at `len/2 - 1` and no longer jumps to MRU. `web_0`'s once-only re-references therefore stop pushing every protected block down one rank. In 1024-slot buckets, where v002's insight blamed pollution of the protected region, this gave back a little of that loss.
- hit-rate (`hm_0` at 16384, essentially flat): The refutation condition in the hypothesis was not triggered. In 128-slot buckets, one-hit reuse on `hm_0` did not depend on full MRU protection. It did not gain from graded promotion either.

## Hurt

- hit-rate (`hm_0` at 131072, the largest move): In the code, a block below the boundary can reach the upper half only by being hit while it sits exactly in the single slot `len/2 - 1`. Every later probation hit moves the earlier occupant one slot down, back below the boundary, and its next hit only returns it to that slot. So `hm_0`'s long-reuse blocks, whose second hit arrives after many other promotions, almost never get into the protected half. That half is refreshed mostly by the 1/32 BIP samples and gets stale.
- hit-rate (`web_0` at 16384): In 128-slot buckets, the nearly frozen upper half takes up about half of each bucket. The capacity that actually adapts is roughly the lower half, so the small-bucket config lost more to wasted protected space than it gained from less pollution. This is the opposite of the hypothesis's prediction that `web_0` would improve at both sizes.
- (`per-op-time` was not measured, even though the hypothesis asked for it. The mid-vector `remove`/`insert` costs no more than the MRU move it replaces.)

## Takeaway

"Two hits to reach MRU" did not work as designed here. All lower-half hits share one landing slot, so "hit again while in the protected region" became "hit again before any other probation hit". In effect the protected half was cut off from the reuse stream. A graded-promotion retry needs a protected landing region that holds position: for example, insert at the protected bottom and let protected items age down normally, or track a per-item hit bit. Shifting a single boundary slot is not enough. Separately, the implementation sends the boundary slot itself (`position == len/2 - 1`) to MRU, while the hypothesis specified `position < len/2`. Under the literal spec that slot would never promote, so even this narrow path into the protected half came from the implementation choice.
