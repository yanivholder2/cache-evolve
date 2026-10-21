# Variant v003 Insight

(Note: the overlay source could not be read in this session, so the code-level reasoning below comes from the hypothesis text and the fitness vectors, not from a line-by-line diff.)

## Helped

- hit-rate: Nothing moved, as intended. The hypothesis was deletion-only and left admission, insert position and eviction order alone, so the policy behaved identically to the parent.
- metadata-size: Unchanged, as intended. The hypothesis required the 40-byte Item layout to stay fixed, and the deletions touched no struct fields.

## Hurt

- per-op-time: Latency got worse on get p50 and p99 and on put p99, and put p50 was flat. The hypothesis described deletions of dead writes and reads plus reuse of the caller's hash. Those should have been neutral or faster, so the regression is probably not a real cost of the removed work. Likely causes are changed code layout or inlining after the edits, or run-to-run measurement noise at this nanosecond scale. The parent's own latency gain was also flagged as possibly noisy.

## Takeaway

Stripping supposedly dead per-put bookkeeping from the v001 incumbent did not speed up the cache. Latency was no better and was measurably worse on the get path, even though get was not the target of the edits. So the bookkeeping was not a meaningful cost here. Per-op-time at this scale seems driven by compiler and layout effects or noise more than by a few removed stores. The cycle 0004 gain probably came from the filter rather than from dead-code removal.
