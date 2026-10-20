# Cross-Cycle Insights

## Recurring wins

- **Resident capacity over admission breadth.** Transferring slots from the small queue to main (shrinking small from 25% to ~10%) produced the largest single gain in the experiment history. Each increment was positive, though with diminishing returns as small-queue share fell further.
- **Ghost precision at main_size/2.** Halving the ghost cap to main_size/2 improved hit rate regardless of which small-queue configuration accompanied it, confirming the benefit is a property of the workload's re-use distance distribution, not an interaction artifact.
- **Stacking complementary levers.** The best-known configuration combines enlarged main capacity (small ≈ 10%) and a tighter ghost window (main_size/2); neither change alone matches their combined effect — the two dimensions amplify each other.
- **Raising MAX_FREQ ceiling.** Increasing the second-chance frequency ceiling from 3 to 4 yielded a real (if modest) hit-rate gain, confirming that a meaningful minority of genuinely hot items exercises the extra retention pass.

## Recurring failures

- **Relaxing the freq > 0 promotion threshold.** Tried in isolation (cycle 0003, neutral) and in combination with ghost precision (cycle 0004, harmful). It never converts additional once-seen items into durable main-queue residents on this workload; dilution cost consistently equals or exceeds any promotion benefit.
- **Extending the ghost window beyond main_size/2.** Tested in cycle 0001 (longer ghost), performance fell. The workload's re-use distances do not extend far enough to benefit; extra history introduces incidental re-accesses that bypass the small-queue filter.
- **Shrinking the ghost window below main_size/2.** Tested in cycle 0003; hit rate declined. This confirmed main_size/2 as a hard floor — re-use distances extend into the 461–922-eviction range and any shorter window discards genuine signal.
- **Ghost re-admission credits (seeding at freq > 0).** Granting re-admitted items a head-start frequency caused net harm by displacing main-queue incumbents that had earned their slots through accumulated accesses. Ghost signal confirms re-use potential but not residency priority.

## Open questions

- **MAX_FREQ upper plateau.** The 3 → 4 gain was real but small; does MAX_FREQ = 5 or 6 continue the trend, or is the hot-set frequency distribution concentrated enough that the upper tail is already near-empty at 4?
- **Structural axes not yet explored.** Queue-sizing and admission-threshold knobs are largely exhausted. Candidates for the next design space: eviction ordering within main (e.g., frequency-weighted vs. pure FIFO within second-chance), hit-count handling on ghost re-admission at freq = 0, and adaptive queue sizing under workload phase changes.
- **freq > 0 combined with higher MAX_FREQ.** Admitted once-seen items into main are currently displaced quickly; a higher ceiling might give them time to prove worth. Empirical evidence so far is discouraging (combination hurt in cycle 0004), but the interaction with MAX_FREQ ≥ 5 is untested.

## Per-policy notes

- **S3-FIFO — small queue.** Functions as a quality gate: its *size* is the dominant lever, not its admission criteria. A narrow small queue (≈ 10% of capacity) is strictly better on this workload; further narrowing below 8% shows diminishing returns. Admission threshold changes (freq > 0) are inert because the bottleneck is capacity, not filtering sensitivity.
- **S3-FIFO — ghost queue.** Value comes from precision, not coverage. main_size/2 is the empirically confirmed optimum from both directions: larger introduces noise from distant non-recurring accesses, smaller discards genuine signal within the workload's re-use horizon.
- **S3-FIFO — main queue and second-chance (MAX_FREQ).** Main queue quality is fragile: flooding with lower-quality entries degrades hit rate even though second-chance eviction is running. MAX_FREQ = 3 is the confirmed lower bound (hot items genuinely use the third pass); MAX_FREQ = 4 adds measurable value; upward direction beyond 4 is the last open sizing axis.
