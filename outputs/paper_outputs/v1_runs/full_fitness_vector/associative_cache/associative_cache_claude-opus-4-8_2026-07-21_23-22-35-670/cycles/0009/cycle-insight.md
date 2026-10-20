# Cycle 0009 Insight

## Cohort outcome

The cohort pushed on both remaining frontiers off cycle 8's index-preserving
incumbent: two latency bets (a cheap integer hash to shed SipHash setup —
v001; a scalar bitmask for the reuse state — v002) and one last hit-rate
probe (a two-hit admission threshold — v003). The clean win was v002 —
folding the parallel `Vec<bool>` into a single `u64` mask collapsed the last
per-hit heap shift and the eviction scan into register bit-ops, holding the
record hit-rate and 16-byte floor bit-for-bit. v001's hash swap was correct
but unresolvable; v003 regressed hit-rate.

## Patterns

- Latency is only capturable when the target lives on a heap-vector path,
  not on a fixed per-op constant. Folding the reuse state into a scalar
  bitmask (v002) removed a real O(bucket) shift and registered; swapping
  SipHash for a one-multiply mixer (v001) targeted a tens-of-nanosecond
  constant that sits below the harness's ~100ns timer, so it bought nothing
  measurable and even let coarse buckets push tails the wrong way. Hashing
  is not the observable lever on this harness.
- The in-residency protection threshold is now falsified as a hit-rate
  source. Demanding a second hit before granting protection (v003) does not
  filter enough scan noise to clear the ceiling — the same bar that rejects
  one-hit-wonders also delays protection for genuinely hot keys until a scan
  can evict them, so it regresses. One-hit binary protection sits at or above
  the sweet spot for this trace.
- Scan-resistance is now free on every dimension at once — metadata (proven
  cycle 8), and now latency (v002) — because under small associativity the
  whole reuse set fits in a scalar the CPU manipulates branch-free. The mask
  splice mirrors the policy's single-element relocation exactly, so
  representation changes stay behaviourally inert.

## Open questions

- With the reuse vector gone scalar, are the incumbent's remaining tails now
  purely the policy's own `items` front-shifts (`insert(0)`/`remove(0)`), and
  is there a harness-resolvable gain left there at all?
- Has the frontier now fully exhausted — hit-rate ceiling firm across every
  swept lever, latency down to items-vector shifts and timer granularity?

## Continuation

Adopt v002's `u64` reuse-mask as the incumbent: it delivers the record
hit-rate with the collapsed `put` p99 tail at the 16-byte floor, superseding
the `Vec<bool>` path. Stop probing hit-rate through in-residency protection
thresholds — one-hit is confirmed at/above the sweet spot and the two-hit bar
regresses; combined with prior cycles, the hit-rate ceiling should be treated
as firm. Do not chase the hash constant or any other fixed per-op cost below
the ~100ns timer — only heap-vector paths are resolvable. If latency is
pursued further, aim solely at the `items` front-shifts and first confirm a
harness-resolvable gain exists before spending a variant on it.
