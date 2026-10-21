# Variant v003 Insight

## Helped

(No measured dimension improved. `hit-rate` is the only dimension in the fitness vector, and it fell below the parent's value. That was H03's own refutation threshold, so the hypothesis is refuted.)

## Hurt

- hit-rate: The 32-counter sketch is shared by every key that maps to the bucket, and its ageing window is too long for it to stay selective. Each miss adds 2 to the counter total, and a halving happens only after `8 * max_size` misses. So just before each halving, the average counter sits around `max_size / 2`, and it is still around `max_size / 4` right after. For any realistic bucket size, `min(sketch[a], sketch[b]) >= 2` is then true for most newcomers, including one-shot scan keys. Those keys get counter 1, survive the hand that rests on them, and displace residents that were worth keeping. In effect, the LIP one-miss filter that the parent's ghost ring relied on has been switched off.

## Takeaway

On this GCLOCK line, the admission signal has to be rare to be worth anything. The counter-1 reward helps only when it goes to a small, exact set of keys, such as those found in the 4-entry ghost ring. A per-bucket count-min sketch that is far smaller than the set of distinct keys missing in that bucket saturates from hash collisions, and its `OR` with the ghost check turns into "admit almost everything at counter 1". If a frequency-based admission signal is tried again, its error has to be controlled first. That means more counters per bucket, a window short enough that the expected counter value stays well below the threshold, or a threshold that scales with the window. As built here, the sketch did not measure "keeps coming back after eviction"; it measured how busy the bucket was.
