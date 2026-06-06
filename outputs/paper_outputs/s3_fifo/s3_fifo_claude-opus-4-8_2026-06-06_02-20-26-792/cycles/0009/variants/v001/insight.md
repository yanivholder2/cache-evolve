# Variant v001 Insight

## Helped

- hit-rate: Replacing the fixed ±1 nudge with a momentum controller (double `step` up to 4 while `desired == last_dir`, reset to 1 on reversal) plus the wider clamp (`capacity/96 .. capacity/8`) let the split travel further and converge faster on workload-a's sustained directional imbalance, recovering hits the tight ±1 orbit around `capacity/24` could not reach.

## Hurt

## Takeaway

Letting the adaptive split accelerate and roam a wider band pushes hit-rate past the parent's single-step ceiling, confirming the cycle-0008 suspicion that the controller was under-reaching rather than mis-aimed. The gain is still small because workload-a's balance only drifts modestly, so most of the widened `capacity/8` headroom and the step-4 cap go unused — the momentum mostly buys faster convergence within a narrow effective range, not a fundamentally different operating point.
