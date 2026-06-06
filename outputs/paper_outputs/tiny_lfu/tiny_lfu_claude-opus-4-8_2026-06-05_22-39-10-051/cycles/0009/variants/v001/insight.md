# Variant v001 Insight

## Helped

- (none observed): the adaptive hill-climbing controller for `protected_cap` did not lift hit-rate above the static-sizing plateau it was designed to beat.

## Hurt

- hit-rate: introducing the windowed controller (`maybe_adapt` / `resize_protected`) lost a single hit versus the fixed `protected_cap = 0.8 * capacity` parent — the demotion churn from nudging the boundary slightly outweighed any phase-tracking gain.

## Takeaway

Making the protected/probationary split adaptive confirmed the prior cycle's read that the boundary is not the bottleneck on workload-a: hill-climbing `protected_cap` window-by-window oscillates around an optimum that the static sweeps had already saturated, so the controller's extra demotions only churn proven-hot nodes back to probation without finding better ground. The flat-to-marginally-worse outcome says the segment size is the wrong knob to turn — remaining hit-rate must come from admission/eviction mechanics, not from where the line between tiers sits.
