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
