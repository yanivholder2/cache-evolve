# Variant v003 Insight

Note: the overlay source could not be listed or read in this session, so this insight rests on the hypothesis (H03), the parent's fitness vector and v003's fitness vector. It does not rest on a code diff.

## Helped

- hit-rate: The change was meant to leave counter semantics untouched, and hit-rate did not drop. Deriving the row indices from one 64-bit mix kept the rows decorrelated enough that estimate quality, and so admission decisions, stayed in line with baseline.

## Hurt

- per-op-time: v003 has no per-op-time entry, although this was the dimension the hypothesis targeted. The claimed speedup is unverified, and the missing measurement counts against the variant.
- metadata-size: v003 reports no metadata-size entry, so there is no evidence on whether the sketch layout or any cached hash state changed footprint.

## Takeaway

Replacing per-row hashing with a single 64-bit hash sliced into row indices appears to be safe for hit-rate. The evaluation did not measure the time and size dimensions, so it neither confirms nor refutes the hypothesis's per-op-time claim. Any conclusion about this change needs a run that records per-op-time and metadata-size.
