# Variant v002 Insight

## Helped

- (none): hit-rate is identical to the parent, so protected-end insertion of freq>=2 small-queue victims into main gave no measurable gain.

## Hurt

- (none): hit-rate did not regress either; the change was neutral on the single measured trace.

## Takeaway

Changing where freq>=2 entries enter main, with promotion threshold, ghost length, small ratio and freq cap held fixed, left hit-rate unchanged on workload-a at this cache size. Either few small-queue victims reach freq>=2 before eviction, or main's reinsertion/second-chance logic quickly washes out the position difference. Insertion position in main looks like a weak lever here, and the next hypothesis should target admission or the small-queue sizing instead. I could not read the overlay sources in this session (only Read/Write were available), so the claim that the code path actually executes is unverified; check that the diff really changes behaviour before drawing stronger conclusions.
