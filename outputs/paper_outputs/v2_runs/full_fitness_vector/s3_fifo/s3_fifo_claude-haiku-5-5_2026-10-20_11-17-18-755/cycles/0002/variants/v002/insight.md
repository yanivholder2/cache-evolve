# Variant v002 Insight

## Helped

- hit-rate: Choosing the lowest-frequency victim among the rotated main-queue entries, instead of always dropping the tail once the 8-step budget runs out, keeps recently referenced entries resident longer, which is the loss v003 introduced. The gain is small and comes from one trace at one capacity.
- per-op-time: The put tail improved relative to v003 because the scan budget and the bounded victim buffer are both kept, so a put still cannot rotate the whole main queue and the added selection work stays within an 8-slot pass.

## Hurt

- per-op-time: The put median rose slightly, because every rotated main-queue entry is now recorded into the buffer and the selection and re-insertion step runs on budget exhaustion, which adds constant work to the common put path.

## Takeaway

The change targets only the victim choice after the second-chance budget is spent, and the measured hit-rate recovery and put-tail improvement are consistent with that scope; the get-path movement is not attributable to this diff since `get()` was not modified. The attribution here rests on the hypothesis description, because the overlay source could not be inspected in this run, and the hit-rate delta is within what one trace could show as noise.
