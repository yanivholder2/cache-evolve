# Cycle 0002 Insight

## Cohort outcome

All three variants branched from cycle 0001's v001 (10% split) and each changed one thing. v001 decoupled the ghost and doubled its size. v002 gave ghost-readmitted keys a freq-1 credit. v003 shrank the small queue to 5%. Only v003 won, and only by a small margin. Both ghost-handling variants lost a little hit-rate.

## Patterns

- Both ghost changes made re-admission more generous, either by keeping a longer history or by giving re-admitted keys a free second lap, and both hurt. On workload-a, one ghost hit is weak evidence of reuse. Main capacity spent on re-admitted keys comes out of keys that have already shown repeated reuse. This matches cycle 0001's finding that the policy rewards proven reuse rather than easy entry.
- The hit-rate gains from the partition sweep are coming from a bigger main queue, not from a bigger ghost queue. v001's isolated ghost test answers cycle 0001's question about where the 10% split's gain came from.
- The partition curve is still rising slightly at 5% but looks close to flat. At this point the ghost, not the small queue, is doing most of the filtering.
- Per-op-time and metadata-size were again not measured, even though cycle 0001's Continuation asked for them. The port onto the indexed structure was also not attempted. Every ghost-growing change, including the Winner, lengthens the linear ghost scan, so its unmeasured cost keeps going up.

## Open questions

- Is a stricter ghost re-admission rule net positive on workload-a, for example requiring repeat ghost hits or inserting at the main tail? And does that rule make a larger ghost useful?
- How much per-op-time does the Winner's larger ghost cost under linear scans?
- Do the small partition gains hold on other traces or other cache sizes?

## Continuation

Use v003's 5% split as the policy baseline. Don't pursue ghost re-admission credits or larger ghost queues on their own. Ghost changes are only worth trying if they make re-admission more selective. The top priority is the action carried over from cycle 0001 and still not done: port the current policy onto the indexed structure, and require every variant to report per-op-time and metadata-size as well as hit-rate. More partition shrinking beyond 5% is likely to give diminishing returns.
