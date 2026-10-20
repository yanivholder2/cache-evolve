# Variant v003 Insight

## Helped

## Hurt

- per-op-time: The put path got slower, not faster, which contradicts H03's premise that the bounded aging rounds were the put-tail cost; the new two-step victim search (a full max-scan of the bucket followed by an unconditional saturating bulk add across every slot) likely does more full-bucket passes on the common full-bucket put than the parent's early-exit round loop did, so the replacement added work rather than removing it.

## Takeaway

Hit-rate held exactly on workload-a, which confirms the bulk-add rewrite preserves the parent's victim and aging semantics. But the put tail rose, so the aging loop was not the dominant put-tail cost and the scan-then-bulk-add form is a net loss; the next step should profile the victim search itself to find where the put tail actually comes from before changing it again.
