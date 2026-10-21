# Variant v002 Insight

## Helped

(No measured dimension improved. Hit-rate is the only dimension in the eval, and it moved the wrong way.)

## Hurt

- hit-rate: `main_insert_behind_hand` now links every new main entry (ghost re-admissions at freq 0 and small graduates, which keep their freq > 1 credit) as the hand's `next` neighbour, so each one is guaranteed a full lap before it is examined. In the parent, head insertion let SIEVE quickly demote newcomers that arrived while the hand was near the head. Removing that removes a cheap filter: re-admitted keys that do not reuse quickly now hold their slot for a whole sweep. Graduates get the same full lap on top of their credit, so they are protected even longer. Each lap's eviction pressure therefore falls on older residents that had already proved reuse and had their freq drained by earlier passes. Because `matches_ring_reference_on_pseudo_random_streams` still checks the full state (small, main order, hand, ghost ring) against a reference that applies the same `hand_pos + 1` rule, and the new mid-list-hand test confirms the full-lap guarantee, the regression comes from the placement policy itself and not from a linking bug.

## Takeaway

On workload-a at size 2048, a guaranteed full lap for every main newcomer does worse than the parent's head insertion. Under head insertion, how long a newcomer survived depended on where the hand happened to be, and that partial quick demotion was doing useful filtering. So the parent's gain over FIFO reinsertion came from survivors staying in place, not from giving fresh entries maximal residency, and the hypothesis's "full sweep to prove reuse" mechanism is refuted as stated. Keep head insertion. Given this regression, a better next probe would cut newcomer protection further, for example by inserting ghost re-admissions nearer the hand or treating them differently from graduates.
