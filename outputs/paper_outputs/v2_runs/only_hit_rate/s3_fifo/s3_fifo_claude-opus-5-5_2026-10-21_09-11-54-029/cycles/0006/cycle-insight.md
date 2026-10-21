# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's v003 (indexed, cap 7, main victims ghosted). v001 gave main victims their own ghost budget on top of the full small-reject budget. v002 kept the policy fixed and replaced the cloned-key ghost with a fixed-size fingerprint ring. v003 adapted the small/main split ARC-style, driven by which source each ghost hit came from. v002 won with a within-noise hit-rate uptick, probably from the ring's slightly wider effective window. v001 and v003 both regressed.

## Patterns

- Adding ghost capacity is not free evidence. Every ghost hit bypasses the small filter and forces a main eviction. The shared, capped budget was protecting main residents from churn, and was not diluting useful evidence. This is the same lesson the re-admission experiments taught, seen from the capacity side: widen evidence a little and it helps, open the bypass further and it hurts.
- Ghost-hit source is a biased control signal on this trace. Small rejects dominate the shared ghost and main victims rarely return, so an adaptive split most likely drifts back toward the larger small queues that the static sweep already rejected. The adaptation also acts too late, because the hit it reacts to has already been re-admitted. This drift is inferred; it was not logged.
- The Winner is the first cost-side change, and it held the policy exactly (verified against a ring-based reference). Its claimed benefits are in per-op-time and metadata-size, which were again not measured. That is now six cycles. Its hit-rate win is incidental. Its main new cost is hashing twice on every miss.
- Every policy lever on workload-a at this size is now flat or negative. The policy family looks saturated here.

## Open questions

- Does the fingerprint ring actually cut per-op-time and metadata-size compared with the cloned-key ghost, and does the double hash on misses cancel out the savings?
- Would a smaller combined ghost budget, or a source-weighted (|B1|/|B2|-style) adaptive signal, behave differently? Both are low priority given saturation.
- Do the accumulated choices (5% split, cap 7, ghosted main victims, ring ghost) hold on other traces and cache sizes?

## Continuation

Use v002 (fingerprint-ring ghost, policy unchanged) as the baseline. An identity hasher on the ghost map is a reasonable small follow-up. Drop split ghost budgets, adaptive splits driven by ghost source, and anything that enlarges the ghost or makes the main bypass more generous. Do not propose more hit-rate tweaks on workload-a. The next cycle should make per-op-time and metadata-size measurable and add at least one more trace or cache size. Without those, no remaining hypothesis can be judged.
