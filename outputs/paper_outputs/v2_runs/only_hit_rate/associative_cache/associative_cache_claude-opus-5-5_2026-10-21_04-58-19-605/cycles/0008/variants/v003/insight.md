# Variant v003 Insight

## Helped

(No measured dimension improved. The fitness vector contains only `hit-rate`. The side benefit H03 claimed, a worst-case `clock_find_victim` sweep bound of `2 * len + 1` steps instead of `3 * len + 1`, would show up in per-op-time. That dimension is not in this evaluation, so the shorter sweep is real by construction but unmeasured here.)

## Hurt

- hit-rate: it fell slightly below the parent instead of rising above it. Lowering `CLOCK_MAX` from 3 to 2 removes the third protection level, so a key that is hit repeatedly now survives at most two sweep passes after its last hit, not three. On workload-a at 2048, some of the keys that lost that last pass were genuinely reused and got evicted just before they came back. The drop is small. It stays inside the band between H03's target and its refutation threshold, so the "reused keys need the third level" explanation is suggested but not decisively confirmed.

## Takeaway

The source diff is exactly the one constant plus its doc comments. Every other mechanism is identical to cycle/0007/v001: ghost admission at 1, the LIP hand placement, and the 4-entry fingerprint ring. So the hit-rate change comes purely from the counter ceiling. The rest of the CLOCK line showed that extra protection for newcomers does not pay. This variant shows the opposite for keys that have proven themselves: the extra protection level they hold at ceiling 3 earns a little on workload-a. Making old hot keys lose protection faster across LIRS phase changes did not free up useful slots. Ceiling 2 is only worth keeping if a future per-op-time measurement shows the shorter sweep outweighs this small hit-rate loss. On hit-rate alone, ceiling 3 stays the lower bracket, and H01's opposite-direction probe decides whether it is also the optimum.
