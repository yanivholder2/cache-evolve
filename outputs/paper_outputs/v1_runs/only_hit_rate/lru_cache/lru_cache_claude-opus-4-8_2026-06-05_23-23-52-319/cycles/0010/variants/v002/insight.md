# Variant v002 Insight

## Helped

- (none measured): the only evaluated dimension did not improve; immediate A1in promotion failed to beat the parent's ghost-gated promotion.

## Hurt

- hit-rate: Changing the `A1IN` arm of `get` to `unlink` + `push_tail(idx, AM)` on first reuse promotes a key into the protected Am list after a single in-window hit, which admits one-hit-wonders that the parent's delayed A1out-ghost gate would have filtered; these transient keys displace proven-reuse entries in Am and the hit-rate slipped below the parent.

## Takeaway

On workload-a, 2Q's delayed, ghost-gated promotion is not the cause of its gap to the incumbent — collapsing it to immediate first-hit promotion erodes hits rather than closing the gap, because the FIFO-then-A1out path is doing real filtering work by demanding a key prove reuse *across* an A1in lifetime before it earns an Am slot.
