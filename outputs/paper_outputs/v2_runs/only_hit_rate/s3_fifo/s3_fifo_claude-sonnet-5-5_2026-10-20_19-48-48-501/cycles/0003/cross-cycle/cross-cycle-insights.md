## Recurring wins

- Structural capacity changes (shrinking the small queue, giving main more room) won in consecutive cycles, though with diminishing returns.
- Stricter admission or filtering helps: a shorter ghost helped or was neutral, and a longer ghost hurt. Ghost length acts as an admission filter.
- Adding information without changing admission or protection rules (main-victim ghosting) was safe and slightly positive.

## Recurring failures

- Loosening admission (looser promotion gate, ghost-hit freq=1, longer ghost) lets weakly reused keys into main and pollutes it. This failed in cycles 1 and 2.
- Faster ageing or decay of main entries (freq halving, lower freq cap) lost every time it was tried. This workload rewards keeping reuse history.
- Single-condition rule tweaks give small or negative results compared with structural levers.
- Metadata-size and per-op-time have been missing from every cycle, so cost claims are unverified.

## Open questions

- Is the shorter ghost additive with the small-queue shrink, or does it overlap with it as stricter admission?
- Where does the ghost-length gain turn over, for example with long reuse distances?
- Does a very small probation window weaken scan filtering on other traces?
- Are the small gains real? All evidence comes from one trace (LIRS) at one cache size, and a second trace and cache size are needed.
- Can metadata and per-op-time reporting be restored?
- Untested levers: stricter promotion rule, stricter ghost-hit rule, main-eviction policy beyond ageing.

## Per-policy notes

- S3-FIFO: baseline strict filtering is well tuned for this workload. The current line is the baseline plus main-victim ghosting plus a smaller small queue. The small-queue ratio is near exhausted, so stop tuning it.
- S3-FIFO ghost: length is the main admission knob (longer is worse, shorter is better or neutral). Next, combine a shorter ghost with the current line.
- S3-FIFO main eviction: leave the freq decrement and cap as they are. Do not try more ageing variants.
