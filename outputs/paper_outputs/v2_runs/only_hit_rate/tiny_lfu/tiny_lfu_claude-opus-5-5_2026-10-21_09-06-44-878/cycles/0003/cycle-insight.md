# Cycle 0003 Insight

## Cohort outcome

All three variants built on cycle-0002 v001 (sampled victims), and each changed one lever. v001 admitted the newcomer on warm ties. v002 widened the victim sample. v003 widened the count-min sketch. Only v003 won. Its gain was clearly larger than any earlier policy tweak, while both contest/sampling levers lost hit-rate.

## Patterns

- The real bottleneck was estimate quality, not the contest rule. Cleaning up collision noise helped. Admitting on ties and searching for victims more deeply both hurt, because they act on estimates that are misleading in two ways: they are inflated by collisions, and they never age, so they reward lifetime counts.
- Strict `>` with keep-victim-on-tie is now tested and should stay. Tie-admission mostly swapped one cold key for another. The get-miss-then-put replay bumps the sketch twice per new key, so a fixed estimate threshold cannot single out one-hit wonders.
- When estimates never age, the lowest estimate marks the youngest resident, not the coldest one. Widening the victim sample therefore evicts recent, still-useful keys. Victim sampling looks exhausted as a lever.
- Part of the "stale-high" inflation blamed on missing aging in earlier cycles was actually collision overcount. Earlier aging results were confounded by this noise.
- For the third cycle running, per-op-time and metadata-size went unscored. v003's larger sketch has a real memory cost that has not been priced yet.

## Open questions

- Now that collision noise is reduced, does very-long-period aging, or doorkeeper-style dedup of the double get/put increment, finally help? Or does lifetime frequency still win on workload-a?
- Is there more to gain from an even wider sketch or a different depth, or has it already plateaued? What is the memory/latency trade-off?

## Continuation

Build on v003, the wider sketch, and treat it as the new baseline for every estimate-related comparison. Next, target estimate meaning: try a doorkeeper or dedup of the get-then-put double count, re-test long-period aging against this baseline, and sweep sketch width/depth while metadata-size is reported. Drop tie-admission and wider victim sampling. Keep insisting on a run that scores per-op-time and metadata-size.
