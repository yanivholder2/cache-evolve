# Variant v001 Insight

## Helped

- hit-rate: No movement. This is expected, because the hypothesis forbade changes to admission or eviction policy and the lookup rewrite should be behavior-preserving.
- metadata-size: No movement. Item layout was held fixed as required, so the bit-packed flag and the segment structs are the same size as in the parent.

## Hurt

- per-op-time: Both get and put latency got worse than the parent, including the p99 tails. The change was meant to cut get-hit cost by hashing once, using atomic or relaxed flag updates, and returning early from the primary bucket. Those edits apparently added overhead instead of removing it. Likely causes are atomic read-modify-write operations that cost more than the plain writes they replaced, and extra control flow on the hot path. Put also slowed even though the change was scoped to the lookup path, so put-side latency probably shares code with the edited paths or the effect is run-to-run noise. I could not read the overlay sources in this session, so these causes are inferred from the hypothesis text and the fitness change, not confirmed against the diff.

## Takeaway

On this small, already-fast incumbent, the supposedly redundant locking and re-hashing was not a bottleneck. Replacing it with atomics and early-return branches made the hot path slower, the same failure mode cycle 0003 saw with conditional guards. Further latency work should start from measurement of where get and put time goes, not from assumed lock or hash overhead. It should also check whether the put-side regression is real by re-running the measurement.
