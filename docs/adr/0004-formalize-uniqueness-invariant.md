# Formalize the Dash uniqueness invariant (fix vs. formalize)

The Dash cache assumes each key appears in at most one slot (stash, target, or
probing) at any time. This invariant was partially documented but unexplained,
and no runtime check existed to catch a future caller that violated it.

**Decision: formalize, not fix.**

Investigation confirmed that the Caffeine JNI caller only calls `put` after a
`get` miss (return value `== -1`). Because `get` returns on the first matching
slot, any resident key is always returned and never re-inserted. Cross-slot
duplicates are structurally impossible under the current call contract.

Adding an enforcement check — a full Stash→Target→Probing scan on every `put`
— would run on every cache miss with no correctness benefit: the invariant is
already upheld by the caller sequence. The performance cost is not justified.

## What was done

- The `Dash` module doc now states the invariant, explains why it holds under
  the JNI call contract, and documents what breaks if it is violated (phantom
  items invisible to `get`).
- `SegmentCore::contains_key` performs the three-slot existence check.
- The default `Segment::put` calls `debug_assert!(!core.contains_key(&key))`
  before every insertion, so test and debug builds catch any future caller that
  violates the contract. The assert compiles out in release builds — zero
  production overhead.

## Considered options

- **Fix (scan on every put).** Correct but adds an O(bucket_size) scan to every
  cache miss. No correctness gain given the current call contract. Rejected.
- **Formalize with debug assertion.** Documents the contract, catches violations
  in test builds, adds no release overhead. Chosen.
- **Do nothing.** The existing partial comment was insufficient: it did not
  explain why the invariant holds or what breaks if it is violated. Rejected.

## Consequences

- The uniqueness invariant is now the documented caller contract for `Dash::put`
  and `Segment::put`. Any future caller that violates it will panic in debug
  builds immediately at the insertion site.
- The G3 invariant suite's no-phantom-items property is consistent with this
  contract: a phantom can only arise from a duplicate insertion, which the
  assert will catch before it reaches the bucket.
- Release builds are unaffected: `debug_assert!` compiles out with `--release`.
