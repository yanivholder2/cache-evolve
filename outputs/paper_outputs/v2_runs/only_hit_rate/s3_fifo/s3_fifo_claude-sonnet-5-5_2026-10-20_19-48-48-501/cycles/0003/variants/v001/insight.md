# Variant v001 Insight

## Helped

- hit-rate: Cutting the small queue from capacity/10 to max(1, capacity/20) gives main more room, so more promoted items survive on the LIRS trace. The improvement over the parent is small.

## Hurt

- None observed. Only hit-rate was measured. A probation window half as long could let more one-hit-wonders or scan items slip through, but this evaluation didn't test that.

## Takeaway

Shrinking the small queue still helps, but the gain from halving it again is much smaller than the earlier step from capacity/4 to capacity/10. That suggests the ratio lever is flattening out on this trace and cache size. Further shrinking is likely to give little, and it risks weakening the small queue's filtering. The next gains probably need to come from the promotion or eviction logic, not the ratio.
