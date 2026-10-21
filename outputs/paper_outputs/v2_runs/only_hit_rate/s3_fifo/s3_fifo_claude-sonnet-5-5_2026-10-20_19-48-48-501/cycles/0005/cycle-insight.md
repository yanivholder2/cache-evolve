# Cycle 0005 Insight

## Cohort outcome

Three structurally different main-queue and ghost-path tweaks were tried: a one-time reinsertion grace for ghost-admitted entries (v001), protected-end insertion of high-freq small-queue victims into main (v002), and a per-key freq flag in the ghost, with re-admission at freq=2 (v003). None improved hit-rate. v002 is the Winner only because it tied its parent, while v001 and v003 were marginally worse.

## Patterns

- Nothing moved hit-rate. This is the second cycle running in which the best variant merely tied the parent. The S3-FIFO structure looks close to a local optimum on this trace.
- Both variants that gave ghost-admitted keys extra credit (a grace pass, a boosted freq) lost slightly. Ghost-hit information is already used well by plain admission, and extra protection keeps entries that stop being reused.
- The main insertion position (v002) looks like a weak lever. Either few victims reach the threshold, or main's reinsertion logic washes out the position difference.
- Variant authors still could not read the overlay source, so we cannot tell whether v002's change was a no-op. The tie is ambiguous.
- Metadata-size and per-op-time were again missing, and v003 added ghost state with no signal. Evidence is still one trace at one cache size.

## Open questions

- Did v002 alter behaviour at all, or is it equivalent to its parent?
- Is any remaining lever on this trace worth the effort, or are we measuring noise?
- Do these null results hold on a second trace or cache size?
- Can metadata-size and per-op-time be restored so cost can be compared?

## Continuation

Treat v002 as a neutral base; prefer the simplest parent if it is cleaner. Drop ghost-hit bonuses, per-key ghost metadata, and main insertion-position tweaks. The next cycle should validate on a second trace and cache size, and restore the metadata and per-op-time measurements. Only try new levers (such as main-eviction scanning or reinsertion rules) if the diff is checked to change behaviour.
