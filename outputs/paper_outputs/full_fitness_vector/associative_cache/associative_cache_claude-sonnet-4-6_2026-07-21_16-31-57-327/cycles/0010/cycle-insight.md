# Cycle 0010 Insight

## Cohort outcome

The cohort tested three distinct angles: first-touch teleportation to the warm-cold boundary (v001), conditional scan expansion above the primary cold window (v002), and a deliberate A/B diagnostic reverting scan direction to LIFO (v003). No variant improved over the parent; all three are selected as the least-bad cohort, with v002 neutral and v001/v003 regressing on individual dimensions.

## Patterns

- **Variable-length promotion breaks latency even when hit-rate intent is sound.** The first-touch teleport (v001) replaced a fixed 3-swap loop with a depth-dependent sweep of up to 8 iterations, doubling put p99 tail latency without a compensating hit-rate gain. The boundary-displacement of warm items at the promotion target offset the cold-zone benefit, leaving hit rate flat-to-negative. O(1)-bounded promotion is now a hard constraint: any promotion path whose swap count scales with cold-zone depth will incur this latency penalty.
- **Conditional scan expansion finds nothing useful when the base is already well-tuned.** The secondary 2-position scan above the cold window (v002) produced zero measurable effect on any dimension. The "all-warm primary window" scenario is rare enough — or the slots just above the window are also warm — that the fallback fires almost never. This closes the conditional-expansion axis for this configuration.
- **Scan direction is conclusively ruled out as the source of the Cycle 0006 put p99 regression.** The v003 diagnostic matched per-op-time values exactly between LRU-among-cold and LIFO-among-cold orderings. This eliminates the last plausible mechanical candidate: swap pressure was ruled out in Cycle 0007, scan direction ruled out here. The regression's origin is now either a legacy artifact of the compact-struct transition — which was permanently resolved before the current base — or a measurement-epoch artifact that is no longer reproducible.

## Open questions

- With swap pressure and scan direction both ruled out, is the Cycle 0006 put p99 regression still a live signal worth chasing, or should it be retired as an historical artifact that the compact-struct fix has already addressed?
- What insertion-point experiments remain that keep promotion strictly O(1)? The first-touch teleport failed on variable cost, but a bounded variant — e.g., capping first-touch promotion at 6 swaps instead of 3 — has not been tested.

## Continuation

The next cycle should retire the Cycle 0006 put p99 regression as a diagnostic target: the two most testable mechanical causes (swap count and scan direction) have been falsified, and continuing to expend variants on this thread has diminishing returns. The productive frontier is now structurally distinct insertion-point changes that stay O(1): experiments such as inserting new items above the conventional tail position, or applying a modest first-touch promotion bonus (e.g., 5–6 swaps vs 3) bounded tightly enough to avoid the latency cliff seen in v001. Conditional scan expansion and flag-expiry paths are closed.
