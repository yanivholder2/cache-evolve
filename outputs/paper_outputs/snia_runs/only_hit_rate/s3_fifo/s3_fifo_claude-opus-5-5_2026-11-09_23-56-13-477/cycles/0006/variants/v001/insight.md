# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 16384): Only the cell the freq-1 ghosting rule was meant to protect held its ground (it moved up very slightly, within noise). A relap that gets no hit now goes into the ghost with freq 1, so its next access takes the strong path straight into main, as it did in the parent. This fixed the weak-path regression that cycle 0005 v001's insight blamed for its flat result here. What it did not do is add anything on top of v002's age gate.

## Hurt

- hit-rate (web_0 @ 131072): This was the target cell, and it went down instead of reaching the relap gain seen in cycle 0005. In the parent, a once-reused small tail was already ghosted at freq 1 and returned straight to main on its next access. So the relap only saves one miss, and only when the reuse lands inside the extra lap. Every relap also holds a small slot for one more turnover, which pushes fresh keys out of small sooner.
- hit-rate (web_0 @ 16384): Here the relap's cost is not offset. Small is only a quarter of a smaller cache, so recycling freq-1 tails back to the head cuts how long first-time keys stay resident. More of them get ghosted at freq 0 and have to come back within the age window to reach main.
- hit-rate (hm_0 @ 131072): Slight loss, from the same source: small slots spent on relaps. At this size v002's wide ghost-age window already sends hm_0's strong returnees to main, so the extra lap has little to add.

## Takeaway

The relap and the parent's ghost path both reward the same signal: one reuse while in small. Once failed relaps are ghosted at freq 1, the relap's only edge is turning a ghost-return miss into a hit inside small, and it pays for that with small-queue capacity. Cycle 0005 v001's web_0 @ 131072 gain did not carry over to v002's base, because the age gate plus freq-1 ghosting already covers what the relap offered there. Stacking a second reward for one reuse brought no gain, so the large-size web_0 stall needs a mechanism that uses a different signal. One candidate is the ghost-age window width, which v002 left untested.
