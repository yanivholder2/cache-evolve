# Variant v001 Insight

## Helped

- None. No dimension improved. I could not read the overlay source, so this comes from the hypothesis and the fitness vectors only.

## Hurt

- hit-rate: Hit-rate fell below the parent on the single trace and cache size measured. Replacing S3-FIFO's reinsert-on-tail eviction with a bounded K=4 lookahead looks like the cause. Skipped entries keep their position and only lose frequency, so they never get the second-chance move to the head. That makes the main queue behave less like FIFO-with-reinsertion. It also likely evicts entries that FIFO ordering would have kept, and it ages the frequency of the skipped entries on every eviction. Together these probably erode the frequency signal of items that are still useful.

## Takeaway

Main-queue eviction in S3-FIFO is not a free lever. Lazy promotion by reinsertion is doing real work, and trading it for a minimum-frequency lookahead with decay-in-place lost hit-rate on this trace. The "scan-resistance" this change was meant to add didn't help here. Later cycles should keep the parent's reinsertion semantics, and any eviction tweak should preserve the way frequency is aged.
