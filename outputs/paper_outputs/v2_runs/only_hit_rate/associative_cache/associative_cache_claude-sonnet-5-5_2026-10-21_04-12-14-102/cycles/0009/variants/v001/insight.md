# Variant v001 Insight

## Helped

- (none): hit-rate is the only measured dimension and it did not improve over the parent. I could not list or read the overlay source in this session, so I rely on the hypothesis text for what changed.

## Hurt

- hit-rate: Inserting missed keys at the middle of the probation segment instead of the probation MRU end made new keys easier to evict. On this trace, many newly inserted keys are re-referenced shortly after insertion. The earlier eviction of those keys cost more hits than the faster removal of one-touch keys saved.

## Takeaway

Mid-probation insertion is slightly worse than top-of-probation insertion in this 4x-wide bucket with a slots/16 probation segment. The probation segment is already small, so moving the insertion point down only shortens the time new keys get to earn a second hit. Insertion position is not a useful knob here. Any scan-resistance gain would have to come from something other than where in probation new keys land.
