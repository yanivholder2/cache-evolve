## Recurring wins

- Shrinking the small/admission partition to enlarge the main resident queue is the dominant, repeatable lever — it paid in both cycles (10/90, then ~6/94), making "give main more capacity" the framework's most reliable win so far.
- Protecting the main queue's hot residents beats feeding it: changes that enlarge main outperform changes that loosen what gets into main.

## Recurring failures

- Loosening the promotion gate or otherwise admitting more keys into main keeps regressing — extra admissions displace hotter residents and cost more than they save.
- Levers that target the same retention behavior as the partition ratio (frequency-clamp / second-chance ceiling deepening) buy almost nothing once main is already large; hot keys clear eviction well before the cap binds. These "distinct on paper" knobs collapse into the partition knob.
- Throttling/gating the ghost re-admission path regresses: a ghost hit is itself reuse evidence, and demoting it to small-queue re-confirmation discards that signal for long-gap reaccesses.

## Open questions

- Where exactly does the partition ratio reverse? Probes are past the knee but still positive; one more shrink (~4/96) would locate the turn before freezing it.
- With main near its useful maximum, which levers are truly orthogonal to main retention — ghost-queue sizing/freshness, or the promotion-gate threshold itself — rather than yet another way to enlarge or protect main?
- If revisiting ghost history depth, does it help only when paired with a gate on the direct-to-main re-admission path, rather than widening history alone?

## Per-policy notes

- S3-FIFO: partition ratio (small/main) is the load-bearing knob; the baseline's stricter admission filtering is intentional, not an oversight. The direct-to-main ghost re-admission path should stay intact — it is the exception to the otherwise reliable "every direct-to-main path is contention-sensitive" rule. Retention stacking onto an already-large main is yielding diminishing returns; future gains likely lie in levers orthogonal to main retention.
