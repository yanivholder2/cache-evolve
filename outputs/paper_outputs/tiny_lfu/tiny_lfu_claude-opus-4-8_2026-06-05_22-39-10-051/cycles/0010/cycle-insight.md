# Cycle 0010 Insight

## Cohort outcome

The cohort finally committed to the long-deferred efficiency pivot, probing
two free-trim directions — halving sketch width (v003) and halving sketch
depth (v002) — alongside the leftover hit-rate question of whether the two
cycle-0009 gate tightenings stack (v001). The 2x width cut took the Winner
slot by holding hit-rate dead flat while reclaiming memory; both the depth cut
and the stacked-gate combination regressed.

## Patterns

- The estimator was over-provisioned on width but not on depth. Halving width
  (64x to 32x cells) held hit-rate within noise — the trace's low churn and
  stable popularity let a 2x-smaller sketch absorb essentially all collisions
  — while halving depth (4 to 2 rows) cost a measurable sliver, because the
  dropped rows were still buying a last fraction of count-min over-estimate
  correction. The free-trim floor sits at depth-4 but at or below width-32x.
- The two gate tightenings are confirmed non-stacking. Layering the hysteresis
  margin on top of the already self-consistent pre-increment score
  over-corrects: the pre-increment scoring had already removed the self-bias
  the margin was meant to offset, so the extra full-count margin now rejects
  genuine reuse candidates rather than just suppressing near-tie noise. The
  margin is only safe relative to the post-increment score it was designed
  against — the cycle-0009 overlap suspicion is resolved as redundancy-plus-loss.
- Fitness blindness is now explicit: every efficiency win this cycle is
  invisible to the hit-rate-only fitness vector. The width and depth cuts are
  scored solely on whether they *avoid* a hit-rate dip, so a flat result is the
  best possible outcome and the actual memory/hash savings go unrewarded.

## Open questions

- Width holds flat at 32x but the redundancy edge is only bounded between 32x
  and the refuted 4x — is there a further free cut (e.g. 16x or 8x) before
  accuracy breaks, and is it worth chasing when fitness cannot see the payoff?
- With hit-rate exhausted and the only scorable efficiency move being "don't
  lose hits," is there any path to credit cost/memory directly, or will every
  future efficiency trim keep registering as a tie at best?

## Continuation

The efficiency pivot is live and partly mapped: width is over-provisioned and
can be cut to 32x for free, but depth is already at its redundancy floor at 4
— do not trim depth further. The next free-trim probe should push width
*below* 32x to find where accuracy actually breaks, not revisit depth. Drop
the gate-tightening thread entirely: the two tightenings are proven
non-stacking and strict-guard headroom is spent. Above all, flag that hit-rate
is fully saturated and the current fitness vector cannot score the memory wins
this pivot produces — future cycles should expect ties, not lift, and judge
efficiency variants on flatness rather than improvement.
