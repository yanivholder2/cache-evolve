## Recurring wins

- Only one cycle so far, so nothing has held up across cycles yet. The candidate to watch is adaptive small/main sizing (ARC-style), which was the only lever evaluated in cycle 0001. It improved hit-rate on workload-a without making any dimension worse, and per-entry metadata stayed flat.
- Main signal from that cycle: queue capacity is a productive lever, and the fixed 25/75 split is not optimal for workload-a.

## Recurring failures

- Infrastructure, not ideas: candidates got rejected because the simulator build failed (Maven HTTP 429s). Those rejections tell us nothing about the hypotheses. If this happens again, treat it as an infrastructure issue and re-submit; don't take it as evidence against the lever.
- No lever has been shown to fail on its merits yet.

## Open questions

- Does adaptive sizing win because it adapts, or because it settles on a better fixed split? The two resize triggers are lopsided (grow-small fires most of the time in steady state), so the split may just converge to a point set by the clamps. Record where `small_size` settles, then test a fixed split at that point.
- Is the latency improvement real? Nothing in the change explains it. It may come from fewer misses, or it may be noise. Re-measure it before crediting sizing changes with latency gains.
- Still untested: looser small→main graduation, and ghosting main-queue victims. Both are movement-rule changes. They should stack with the sizing lever, so test them with the adaptive-sizing variant as the parent.

## Per-policy notes

- S3-FIFO: so far only the queue-capacity knob (small/main split) has been tried successfully. The movement-rule knobs (graduation threshold, ghost coverage of main) are untested. All results so far are on workload-a only, so nothing yet says whether they carry over to other workloads.
