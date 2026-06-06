# Cycle 0003 Insight

## Cohort outcome

All three variants branched off cycle 0002's narrow-split winner and
probed the *knees* of the levers established so far: pushing the split
one step narrower, stacking the freq-ceiling raise on the narrow split,
and shrinking (rather than widening) the ghost queue. The two
retention/capacity moves lifted hit-rate again but by diminishing
margins; the ghost-shrink traded a small hit-rate loss for a large
put-latency win — and all three landed as Winners, signalling latency is
now scored alongside hit-rate.

## Patterns

- The split sweep is still monotone but clearly flattening: another
  capacity shift into main paid off, yet less than the prior step. The
  under-screening boundary was not crossed — it sits just past the
  current narrowest split, so the knee is near but not found.
- The freq-ceiling and split levers are confirmed orthogonal: deeper
  durability for proven keys still helps when combined with the narrow
  split, because how-long-a-key-survives is independent of how-many-fit.
  But the combined gain is marginal — extra second chances are starting
  to protect keys workload-a no longer reaccesses.
- Ghost length is a **latency/hit-rate tradeoff**, not a free hit-rate
  lever in either direction. Cycle 0002 showed widening it hurts; this
  cycle shows shrinking it also hurts hit-rate (the freshest quarter
  drops legitimate near reuse) while sharply cutting the linear
  membership-scan cost in put. The reuse distance this trace rewards is
  narrow but extends past the most-recent 25% of evictions.
- Metadata discipline held: every winning change retuned an existing
  field or constant rather than adding struct state, keeping metadata-size
  flat.

## Open questions

- Where exactly is the split knee — does the very next narrowing finally
  under-screen one-hit-wonders and regress?
- Is there a ghost length between ~25% and 100% of main that keeps most
  of the latency win while recovering the near-reuse hits lost at 25%?
- With latency now rewarded, are hit-rate and per-op-time pulling in
  opposite directions on the same knobs, forcing explicit tradeoffs?

## Continuation

The hit-rate levers (split, freq-ceiling) are deep into diminishing
returns — take at most one more confirming step on each and expect the
split knee imminently; stop assuming monotonicity holds. The newly
interesting axis is the **latency/hit-rate frontier** the ghost-shrink
exposed: sweep intermediate ghost lengths to find the point that banks
the put-latency win without shedding near-reuse hits. Keep avoiding
ghost *widening* and any change that adds struct state. Treat per-op-time
as a real, scored dimension now — but only when a change actually alters
a scan length (ghost/resident traversal), not as run-to-run jitter.
