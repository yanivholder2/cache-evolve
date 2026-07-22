# Task: cycle-insight-writer

Cycle: 0010
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010

## Winners

- v001
- v003
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\associative_cache_2026-07-21_23-22-35-670\cycles\0010\hypotheses\H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort probed for hit-rate headroom *above* the recovered SLRU
incumbent from three directions — widening per-set associativity (v001),
giving the bucket cross-eviction ghost memory in 2Q style (v002), and
retuning the protected cap toward near-total protection (v003). None gained
on any dimension; the two Winners (v001, v002) qualified only by holding the
metadata floor while every hit-rate bet regressed. This cycle is a cluster
of falsifications: it maps three walls around the current ceiling rather
than breaking through it.

## Patterns

- Set *diversity* beats per-set *depth* at this size. Trading bucket count
  for bucket width (v001) gave each SLRU set more promotion slots but
  coarsened the hash spread, and hit-rate fell — so the ceiling is not a
  bucket-width limit. Widening also surfaced the predicted scan cost as a
  doubled `get` tail without moving the median.
- Cross-eviction memory does not lift hit-rate here. Admitting ghost-hit
  keys as pre-protected (v002) over-inflates the protected set: a
  re-reference is not proof of long-term heat, and blanket protection erodes
  the very reuse-bit discrimination that earned the incumbent its ceiling.
  It also taxed every `put` with linear ghost bookkeeping.
- The protected-cap knob has now been falsified at both extremes. Cycle 6
  ruled out `max_size/2` (too aggressive); v003 rules out `max_size - 1`
  (near-total protection degenerates the two-segment order into LRU, killing
  scan-resistance and lengthening protected-MRU rotations). The optimum sits
  strictly between the ends, not at either wall.
- Off-`Item` metadata hygiene held everywhere this cycle — ghost rings and
  cap knobs all stayed off the measured struct, so the 16-byte floor was
  never at risk. The overlay hazard that recurred in prior cycles did not
  reappear.

## Open questions

- If neither more associativity, more memory, nor more protection lifts
  hit-rate, is two-segment SLRU at diverse small buckets simply the
  structural ceiling for this trace?
- Where does the protected cap actually optimize, now that both extremes are
  falsified — is there a mid-range setting that turns a hold into a gain, or
  is the default already near-optimal?
- Is any remaining headroom a hit-rate question at all, or has the frontier
  fully shifted to reclaiming the incumbent's per-access rotation latency?

## Continuation

Treat the segmented incumbent's hit-rate as at or near its structural
ceiling for this trace: three independent headroom bets (wider
associativity, 2Q ghost memory, near-total protection cap) all regressed, so
stop chasing hit-rate gains through more depth, more memory, or more
protection. Keep the metadata floor and hash-spread diversity as settled —
do not trade bucket count for bucket width again. If the protected cap is
revisited, search the mid-range only; both extremes are now falsified. The
most promising unclaimed lever is latency: the per-access protected rotation
remains pure overhead outside the saturated-bucket case, so next cycle
should pursue lazy/conditional demotion to shed rotation cost while holding
the ceiling, rather than another hit-rate probe.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort turned fully onto the latency frontier that cycle 7 pointed to,
attacking the incumbent's per-access costs from three angles: skip the
below-cap protected rotation (v001), replace the O(n²) reuse-vector rebuild
with an index-preserving move (v002), and re-probe the protected cap at a
mid-range setting (v003). The real win was v002 — collapsing both p99 tails
while holding the best-on-record hit-rate and the 16-byte floor bit-for-bit.
v001 proved its target inert but unmeasurable; v003 confirmed the cap is a
nearly flat lever.

## Patterns

- The incumbent's blown-out tails were never intrinsic to carrying
  scan-resistance — they were a snapshot-and-rebuild artifact. Mirroring the
  policy's own single-element relocation onto the parallel reuse vector
  (v002), instead of reconstructing it by key-search each hit, makes the
  scan-resistance state free on latency as well as metadata. This is the
  cycle's clean, portable latency win.
- Not every per-access move is worth optimizing: the below-cap protected
  rotation is a single-element shift beneath the harness's 100ns timer
  resolution, so removing it (v001) is provably hit-rate-inert yet cannot
  register as a p50/p99 gain. Latency effort must target costly paths
  (per-hit rebuilds, front shifts), not sub-resolution single-element moves.
- The protected-cap knob is now mapped end to end and reads as nearly flat
  across its usable range. Both extremes were falsified in prior cycles; the
  mid-range `3*max_size/4` (v003) moves hit-rate only a hair and pays for it
  in a doubled `get` tail from deeper protected-MRU rotations. This lever is
  effectively exhausted as a hit-rate source.
- Metadata hygiene held throughout — every change stayed off the measured
  `Item`, and the anchored-parent discipline (v002 chose its parent
  explicitly to avoid the 40-byte overlay trap that sank the earlier attempt)
  kept the 16-byte floor intact.

## Open questions

- With the O(n²) rebuild gone, where do the incumbent's remaining tails sit —
  the `insert(0,…)`/`remove(0)` front shifts, or is the path now essentially
  at its latency floor?
- Is there any latency headroom left that the harness can actually resolve,
  or has sub-100ns granularity become the binding constraint on further
  micro-optimization?

## Continuation

Adopt v002's index-preserving reuse-vector move as the incumbent: it
delivers the record hit-rate with clean p99 tails at the 16-byte floor, and
supersedes the rebuild-based path. Stop tuning the protected cap — all three
regions (aggressive, near-total, mid-range) are now explored and it is a
flat lever; treat it as settled at the parent default. Do not pursue
sub-resolution single-element optimizations like v001's rotation skip; they
are correct but unrewardable by the eval harness. If latency is pursued
further, target the costlier front-shift paths (`insert(0)`/`remove(0)`) and
first confirm there is a harness-resolvable gain to capture, rather than
another hit-rate probe — the hit-rate ceiling remains firm.


### Cycle 0009

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
