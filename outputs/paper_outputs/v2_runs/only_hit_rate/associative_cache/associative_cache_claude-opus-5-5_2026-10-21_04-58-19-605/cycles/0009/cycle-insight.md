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
