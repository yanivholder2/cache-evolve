## Recurring wins

- Small, policy-neutral or narrowly targeted changes are the only ones that have won (fast single-hash/flat sketch in 0001, frequency-sampled victim selection in 0002). Both gains were marginal on hit-rate.
- Keeping long-run frequency history has helped on workload-a in both cycles. Undecayed counts beat freshness.
- Winners stack: v003 hashing (0001) carried into v001 sampling (0002). Building on the latest winner has not regressed.

## Recurring failures

- Sketch aging/halving lost hit-rate in both cycles. It lost with conservative update at a short period, and it lost alone at a longer period. Halving creates ties, and strict-`>` keep-victim makes newcomers lose those ties.
- Window restructuring has not paid off. A fixed 1% segmented layout was neutral, and a naive hill-climbing adaptive window drifted toward a large LRU-like region, which the LIRS-style trace punishes.
- Evaluation has never reported per-op-time or metadata-size, two cycles running. Every latency or space claim (fast hashing, extra miss-put estimates, per-get climber bookkeeping) is unverified.
- Changing two levers at once (0001 v002) blurred attribution. Isolating aging in 0002 showed that conservative update and/or the short period caused most of the damage.

## Open questions

- Tie handling in the admission contest has been named as the bottleneck in both cycles but has never been tested. Candidates: `>=`, random admit on ties, a doorkeeper. Test each alone and on top of victim sampling.
- Is victim sampling capped by cold or stale-high head entries? A wider sample, or one placed elsewhere, could show whether it scales.
- Can aging be rescued by a very long period combined with a tie-aware contest, or is freshness simply worthless on workload-a?
- Would a damped, warmed-up climber with small steps and a contest-on-shrink do better, or is a small fixed window simply right?
- Does any of this hold beyond workload-a? Every conclusion so far rests on one trace.

## Per-policy notes

- W-TinyLFU (window + SLRU): window size or shape has been neutral or harmful. A small fixed window appears near-right for this trace.
- TinyLFU admission (CM sketch + strict-`>` contest): this is where the hit-rate leverage is. Ties decide most contests, and the default tie rule favors the victim.
- Sketch maintenance (aging, conservative update): consistently harmful on workload-a so far. Deprioritize this unless it is paired with tie-aware admission.
- Hashing/sketch layout: the single fast hash with a flat sketch is the current base. Its claimed speed benefit is still unmeasured, and its small hit-rate shift probably comes from collisions.
- Victim selection: frequency-sampled victims among the oldest LRU entries gave a small, real gain. Sample width and placement are unexplored.
