# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010

## Winners

- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-21_04-58-19-605/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Each variant took a different thread from Cycle 0006's continuation. v001 swapped the CLOCK-line ghost ring's stored keys for 16-bit fingerprints, a cost-only change. v002 inserted ghost hits on the LRU line at 3/8 depth instead of 1/4. v003 added a per-bucket miss-frequency sketch as a second admission signal on CLOCK. v001 won by matching its parent's hit rate exactly. H01 held on hit rate, but its real gain (about one-eighth of the ghost memory and a cheaper miss-path compare) still can't be seen, because the evaluation doesn't measure metadata-size or per-op-time. v002 slipped very slightly and v003 regressed clearly, so H02 and H03 were both refuted.

## Patterns

- The ghost ring only needs to answer "was this key evicted recently?" A membership test is enough, so a compact fingerprint ring loses nothing. This is the first change since Cycle 0004 that lowers cost on the settled CLOCK shape without moving hit rate.
- On LRU, ghost-hit insert depth peaks at 1/4. Going deeper gave nothing back, and the curve looks flat there. That was the last single-knob lever left on the LRU line. Depth, ring size, ring admission and insert depth have now all peaked at or before their current settings.
- The admission reward must stay rare. v003 kept the reward capped but widened the signal that grants it. Its small, shared, slow-ageing sketch saturated from hash collisions and ended up granting counter 1 to most newcomers, which in effect turned off the strict one-miss filter. Cycle 0004 found that a capped reward is necessary. This cycle shows it isn't sufficient: the trigger also has to stay selective.
- This is the second cycle running with no hit-rate gain on workload-a. The design family looks saturated on that trace. Seven cycles in, metadata-size and per-op-time are still unrecorded, and v001's win depends entirely on them.

## Open questions

- Would a miss-frequency signal with controlled error help? That would mean more counters, a short window, or a threshold that scales with the window. Or is the 4-entry exact ring already the complete admission signal for workload-a?
- Does fingerprinting the ghost ring work the same way on the LRU line?
- Once costs are measured, does the fingerprint ring (perhaps shrunk to 3 entries) make CLOCK+ghost the clear main line over LRU+ghost?

## Continuation

Adopt cycle/0007/v001 (CLOCK, strict newcomer filter, 4-entry fingerprint ghost ring) as the base going forward. Retire LRU-line insert-depth tuning and treat LRU+ghost as a reference line only. Drop broad frequency-based admission signals. Retry one only if its false-positive rate is bounded by design. Stop running hit-rate-only cycles on workload-a: getting metadata-size and per-op-time recorded, or a second trace or cache size added, is now required before any candidate can be told apart.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants tested a single knob on the cycle/0007/v001 CLOCK base (strict newcomer filter, 4-entry fingerprint ghost ring). v001 raised the first-hit promotion from 1 to 2. v002 recorded ghosts only for keys never hit while resident. v003 lowered the counter ceiling from 3 to 2. All three hypotheses were refuted, and each variant lost a small amount of hit rate. v002 is the listed winner only because its loss was the smallest. Its own insight says the ghost-admission filter gives no real leverage, so it adds no mechanism worth keeping.

## Patterns

- Hit-side protection now has a bracket on both sides. A bigger first-hit step (v001) and a lower ceiling (v003) both lost hit rate, so the uniform +1 step with a ceiling of 3 is locally optimal. Too much protection shields once-touched keys, and too little evicts keys that really are reused. Every GCLOCK knob has now been probed in both directions on workload-a.
- Ghost-ring admission filters lose on both lines, but for different reasons. On LRU (Cycle 0006), excluding never-hit keys hurt. On CLOCK, excluding hit keys hurt, though only barely: with the hand resting on the newcomer, most evictions are already never-hit keys, so the filter removes only a thin slice. Aged-out residents that were hit at some point are worth recording on both lines. The unfiltered 4-entry ring is already complete.
- One reward at the same small size works for both a ghost return and a first hit. Cycle 0004 found that the admission reward must be capped. This cycle finds the same cap applies on the hit side.
- This is the third cycle in a row with no hit-rate gain on workload-a, and the third in which the continuation asked for cost metrics that never arrived. v003's shorter worst-case sweep is a real cost saving that the evaluation still can't see.

## Open questions

- Would raising the ceiling *together with* a larger first-hit step help, so that the extra reward doesn't flatten the frequency range? It's untested, but the trend in this cohort suggests a small effect at best.
- Is ceiling 2 worth taking once per-op-time is measured? Its hit-rate loss is small and its worst-case sweep is a third shorter.
- Do any of the losses in the last three cycles exceed run-to-run noise on a second trace or cache size?

## Continuation

Treat cycle/0007/v001 as final for hit rate on workload-a. The design is fully bracketed: ring size, ring admission, newcomer grace, hit-step size and counter ceiling all peak at their current settings. Drop all single-knob CLOCK hit-rate probes, including ring-admission filters and hit-reward magnitude. The next cycle should not propose another hit-rate-only tweak on workload-a. It should get per-op-time and metadata-size recorded, or add a second trace or cache size, and then re-judge ceiling 2, a 3-entry fingerprint ring and the LRU+ghost reference line on those axes.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle moved away from single-knob hit-rate probes. It tried one structural change and two cost-only changes. H01 added two-choice bucket placement on the CLOCK base. All three of its attempts were rejected at the invariant gate. H02 (v002) shrank the item record by dropping the wall-clock timestamp and narrowing the counter. H03 (v003) ported the fingerprint ghost ring onto the LRU reference line. Both held hit rate exactly, as they predicted. v002 wins because it has the same behaviour as cycle/0007/v001 with a smaller item. As in earlier cycles, that saving is real by construction but the evaluation does not measure it.

## Patterns

- Two-choice placement is ruled out by the rules, not by bad code. The protected slot-legality invariant requires every key to sit in the bucket its hash selects. Any scheme that puts a key in an alternate bucket fails that invariant no matter how it is written. The "which bucket" lever is off-limits in this harness unless the bucket-index function itself changes, which every invariant check would still have to accept.
- Changes that only cut cost keep landing cleanly. These are fingerprint rings (Cycle 0007 and now v003), the item shrink (v002) and the shorter ceiling (Cycle 0008). Each held hit rate because the GCLOCK and LRU+ghost paths never read the state that was removed. The remaining design can be shrunk further without changing behaviour.
- On the LRU line, too, the ghost ring is only used as a membership test. Lessons about which keys a ghost ring captures still differ between the two lines. The representation, though, carries over: both lines can share the same 16-bit fingerprint ring.
- The cost side now has two small trade-offs that no one can measure yet. Fingerprinting adds hashing on the miss path. The logical stamp in v002 adds one shared atomic in `Item::new`, which could cause contention under multithreaded load, but only on policies that are not the winner.
- This is the fourth cycle with no hit-rate gain on workload-a, and the fourth whose continuation asked for cost metrics that never arrived.

## Open questions

- Could a bucket-index change that still passes the invariant check, such as a better-mixing single hash, reduce conflict misses where per-slot rules cannot?
- Once metadata-size and per-op-time are recorded, does the extra hashing from fingerprints outweigh their memory saving? Does the shared stamp atomic matter at all?
- Is anything left to strip from the item or bucket on the CLOCK line, for example a narrower counter, since its range is tiny?

## Continuation

Adopt cycle/0009/v002 (cycle/0007/v001 plus the compact item) as the new base. Treat the fingerprint ring as the shared ghost representation for both lines. Drop all multi-bucket and alternate-placement ideas, because the slot-legality invariant forbids them. Next cycle, either propose only changes that keep each key's single hashed bucket, such as a different bucket-index hash or deeper metadata compaction, or make recording metadata-size and per-op-time the explicit deliverable. Do not propose another hit-rate tweak on workload-a.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
