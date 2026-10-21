# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants started from baseline and each changed one thing: the partition split (v001), the graduation rule (v002), and the data structures with policy left unchanged (v003). The Winner was v001, which shrank the small queue to the paper's 10%. v002 lowered the graduation threshold and lost a little hit-rate. v003 kept hit-rate exactly where it was.

## Patterns

- On workload-a, giving more room to the main queue and the ghost queue (which grows with main) helped. Making it easier to enter main did not. The policy rewards keys that have shown repeated reuse, not keys that were reused once.
- The ghost queue already acts as a safety net. A key that misses graduation can still get back into main later, so both a shorter probation window (v001) and an earlier promotion (v002) change less than expected.
- Each variant reported hit-rate only. Neither per-op-time nor metadata-size was measured anywhere in the cohort, so the stated purpose of v003 and the ghost-scan cost risk in v001 are both still untested.
- v002 changed two things together (the threshold and a frequency reset on graduation), so we can't tell which one caused the regression.

## Open questions

- Does the larger ghost queue in v001 hurt per-op-time while the linear scans are still in place?
- Does v003's indexed slab actually lower get/put latency, and what does it cost in metadata-size?
- Was v002's regression caused by the lower threshold, by the frequency reset, or by both together?

## Continuation

Use v001's 10% split as the new policy baseline, and port it onto v003's behaviour-neutral indexed structure so that hit-rate gains and per-op-time gains can stack. Every variant must report all three dimensions. Don't pursue earlier graduation as it was tried here. If you revisit it, test the threshold change alone, with no frequency reset. Partition sweeps around 10% and ghost-queue sizing that is decoupled from main are worth probing next.


### Cycle 0002

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's v003 (5% split) and each changed one thing. v001 inserted ghost-readmitted keys at the main tail (stricter re-admission). v002 is the long-requested behaviour-neutral port onto indexed structures. v003 raised the main frequency cap from 3 to 7. The Winner was v003, by a small hit-rate gain. v002 matched its parent's hit-rate exactly, as intended. v001 regressed.

## Patterns

- After three cycles the lesson is consistent: workload-a rewards protecting keys that have shown reuse. The ghost hit is itself enough evidence of reuse. Giving re-admitted keys extra credit hurt in cycle 0002, and here a near-zero probation window (tail insertion) also hurt. The current front-of-main, freq-0 re-admission looks close to the right point between those two.
- Tail insertion did not test "require a second reuse signal". Its deadline is the next main insertion, not a span of the access stream, and in steady state that comes almost immediately. Any stricter re-admission rule needs a probation window measured in accesses.
- Main-queue retention is a new axis. It gave a real but modest gain: a cap of 3 already captured most of the frequency signal, because cold ghost re-entries usually supply the eviction victims.
- The indexed port finally exists and is verified as exact. Per-op-time and metadata-size were still not measured, so we can't yet tell whether it helps. The Winner's higher cap makes the linear-structure sweeps longer, so the unmeasured cost keeps growing on the incumbent lineage.

## Open questions

- How much per-op-time does the indexed port save, and how much metadata-size does it add, given that its slab, index and duplicated ghost keys use more memory?
- How long do eviction sweeps get under the higher frequency cap, and does a cap above 7 add any hit-rate?
- Does a re-admission probation window based on access count, somewhere between tail and front, beat front insertion?

## Continuation

Use v003's frequency cap of 7 as the policy baseline, and combine it with v002's indexed structure. The policy and structure changes are independent and should stack. Every variant must report per-op-time and metadata-size, which have now gone unmeasured for three cycles. Don't pursue tail insertion or ghost re-admission credits. Probing the frequency cap further, or decoupling the graduation threshold from the cap, is a reasonable next step. Do it only once the cost of the eviction sweep is measured.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
