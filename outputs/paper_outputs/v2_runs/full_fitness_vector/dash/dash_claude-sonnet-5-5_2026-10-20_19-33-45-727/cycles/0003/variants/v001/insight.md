# Variant v001 Insight

## Helped

- hit-rate: Requiring two stash hits before stash→target promotion filters out keys that are re-referenced only once, so fewer of them displace established target items. The gain over v002 is tiny and may be within trace noise. (Based on the hypothesis; the overlay source was not readable in this session.)

## Hurt

- per-op-time: Both get and put p50/p99 latencies were no better than v002's, and get latency was clearly worse. The likely cause is the extra masked read-modify-write of the packed counter on every stash hit, plus the counter reset on demotion. Measurement noise may also contribute.

## Takeaway

Widening the one-bit second-hit flag to a 2-bit counter kept the Item at 40 bytes, so the signal really is free in metadata. But the admission gain was marginal, while the per-hit counter bookkeeping added latency. A stricter promotion threshold on the stash path looks like a weak lever for this trace. Any future admission-signal change should show a clearly larger hit-rate gain than this one before it is worth the hot-path cost.
