# Variant v001 Insight

## Helped

- hit-rate: Compared with v003, this variant makes exactly the two changes H01 asked for. `MAX_FREQ` goes from 3 to 7, and the promotion step in `evict_one()`'s `small` branch now runs `node.freq -= 1` where it used to run `node.freq = 0`. The split, the ghost path in `put()`, the `main` branch and `push_ghost()` are byte-identical to v003. So a key hit k times during probation now enters `main` with k − 1 credits and survives that many extra passes of lazy reinsertion. The higher cap lets both those credits and later `main` hits build up past 3. Hit rate rose over v003, but only by a sliver. That fits the parent insight's diagnosis: with `small_target` at 25 (size 2048), few keys collect two or more hits before leaving `small`, so few keys benefit from carryover.

## Hurt

(No measured dimension regressed. One cost is built into the change. `MAX_FREQ = 7` applies to `get`/`put` for every resident key, not just keys promoted from `small`. A key that was hot and then went cold can now cycle through `main` up to 7 times before eviction, where v003 allowed 3. That holds `main` slots away from newer reused keys, and it probably offsets part of what carryover gains.)

## Takeaway

On v003's `capacity / 80` split, carrying credit from `small` to `main` mostly overlaps with what the smaller `small` queue already does. The two levers do not add up: the gain over v003 is tiny and could be noise, given the single trace and single size. The variant also changes two things at once (carryover and the higher cap), so it cannot tell which one produced the sliver. More importantly, it leaves untouched the bottleneck v003 identified: keys admitted through the ghost queue still enter `main` with `freq = 0`. The next credit experiment should target that ghost path, not the `small` path.
