# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-49-23-803/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

All three variants branched off cycle 0006's main-first-scan incumbent and
converged on a single target: the get-p50 floor, the median scan-to-hit
distance that has resisted every length/content tweak. Three distinct
mechanisms were tried — an O(1) hash index over a key-only resident store,
in-place front-loading of saturated keys, and a dedicated short `hot`
segment. Only the hash index actually lowered get-p50; both
reorder-the-linear-scan approaches missed the median and merely shifted
cost to the tail. Hit-rate stayed bit-identical across all three.

## Patterns

- get-p50 is structural, not spatial. The two variants that kept the linear
  scan and tried to make the median hit *shallow* — by clustering hot keys
  at the head or carving them into a short front segment — both failed at
  their stated target. The median get does not land where Zipfian intuition
  predicts: front-loading gave no median savings, and the segment was
  probed-then-missed often enough that the extra hot-before-main scan made
  p50 worse. Only eliminating the walk entirely (one hash probe) moved the
  floor.
- The hash index was the cycle's strict win: it dropped get-p50, get-p99,
  *and* both put percentiles together, because constant-time resolve also
  collapses the miss and update-in-place paths. This is the first move in
  several cycles to lower median and tail in the same step rather than
  trading between them.
- The two losing variants repeated cycle 0006's tail-redistribution
  pattern: any in-place reorder or segment promotion pays a rare
  O(position) move or added bookkeeping that widens put/get p99 for no
  median return.
- Metadata discipline is now visibly stretched. Every variant held the
  scored `Entry` at 24 bytes, but the winning index — and the losing `hot`
  segment — both grew the unscored `S3Fifo` container. The hash index in
  particular stores every resident key twice (key-queue + entries map), a
  real memory cost the scored summary does not capture.

## Open questions

- Is the hash index a true free win, or does its unscored key-duplication
  (each resident key stored twice) become the binding constraint once
  container memory is scored?
- With the resident lookup now O(1), is any scan left on the hot path to
  shorten — is workload-a now at a joint hit-rate and median-latency floor?
- Could an inline fast hasher's rare resize spike resurface as a put-p99
  regression under different inflow, even though it didn't here?

## Continuation

The lesson is decisive: stop trying to make the *linear* resident scan
shallow — both the in-place front-load (H02) and the segmented hot queue
(H03) missed the median and only widened the tail, because get-p50 is a
structural scan cost, not a hot-key-scatter problem. The productive path is
to **eliminate** the walk, not reorder it: the O(1) hash index over a
key-only store is the new incumbent and the first median+tail win together.
Next cycle should pressure-test it — specifically whether its double
storage of each resident key holds up if container memory enters scoring,
and whether any hot-path scan remains worth removing. Keep treating
hit-rate as closed (held bit-identical again) and keep the scored `Entry`
at 24 bytes, but expect unscored container growth to be the next frontier
of cost.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants branched off cycle 0007's O(1)-get hash-index incumbent
and split their fire across the two remaining live targets: the last linear
scan on the *put* path (the ghost membership walk) and the long-frozen
metadata-size dimension. Two variants eliminated the put scan by different
mechanisms; one finally moved metadata-size off its seven-cycle plateau.
All three were Winners, hit-rate held bit-identical.

## Patterns

- The ghost membership scan was the binding put cost, now confirmed by
  removal. Both put-latency variants crushed put p50/p99 by replacing the
  O(ghost_len) `iter().any` + `retain` rebuild with an O(1) probe — one via
  a dedicated ghost HashSet, one by folding ghost state into the existing
  `entries` map as value-less tombstones. This was the last linear walk on
  any hot path; the resident scan was already gone in 0007.
- The two put fixes embody opposite memory tradeoffs for the *same*
  latency win. The separate set duplicates every ghost key (a second
  key-keyed table); the folded-tombstone variant adds no table but taxes
  the get path with an enum-discriminant match and inflates the resident
  map with ghost tombstones. The scan elimination is settled; how to pay
  for it is the new contested axis.
- Metadata-size finally moved. The `key` field — dead weight ever since
  0007's hash index made the resident key recoverable from the map key —
  was deleted, collapsing the scored layout below its frozen plateau. A
  delayed dividend of the prior cycle's index, and a free win that touched
  no lookup, eviction, or promotion path.
- Container memory is now the explicitly contested frontier the prior
  cycles flagged. Every put variant grew the unscored `S3Fifo` container
  (ghost set, ghost counter, or tombstone-inflated map) even as the scored
  per-entry cost shrank.

## Open questions

- Of the two O(1)-ghost designs, which is net-better once unscored
  container memory is weighed — the duplicated-key HashSet, or the
  tombstone-laden single map that taxes get?
- With both hot-path scans now eliminated and the scored entry shrunk, is
  any latency or memory lever left, or is workload-a at a joint hit-rate /
  median-latency / scored-size floor?
- Do the tombstone designs accumulate stale order-queue / map entries that
  degrade under longer or different inflow than tested here?

## Continuation

The scan frontier is closed: get (0007) and now put (0008) are both O(1),
so stop hunting hot-path linear walks — there are none left. The two
remaining live threads are both about *cost of the same hits*, not more
hits: (1) reconcile the two ghost-O(1) designs against unscored container
memory, since they make opposite tradeoffs for an identical latency win,
and (2) keep probing the now-unfrozen metadata-size dimension for further
dead-field deletions the hash index may have made redundant. Treat
hit-rate as closed (bit-identical at the same value for many cycles
running). Expect the next binding constraint to be container/tombstone
memory, not latency.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants attacked the single live dimension flagged by prior
cycles — scored metadata-size — branching off both O(1)-ghost families
(separate `ghost_set` and folded-tombstone). Two ported the redundant-key
deletion across designs to reach the same shrunken `Entry`; one went
further and relocated the freq counter out of the struct entirely. All
three were Winners, hit-rate bit-identical, latency untouched.

## Patterns

- The redundant `key` field generalized cleanly across both surviving
  ghost designs: dropping it (carried as zero-sized `PhantomData<K>`)
  shrinks the scored `Entry` identically whether ghosts live in a separate
  set or as in-map tombstones. The field was dead weight in every design
  because the resident key already lives in the `entries` map key and the
  order queues — confirming the metadata win is design-independent.
- The apparent metadata floor was alignment padding, not a hard limit.
  After the key drop, `value`+`freq` padded up under `u64` alignment;
  relocating `freq` into the map's value slot beside a value-only `Entry`
  collapsed the scored type below that floor with no change to real
  per-slot memory or hot paths. Scored size is now driveable by field
  *relocation*, not just deletion.
- Every move was measurement-only: no eviction, promotion, or ghost
  probe was touched, so hit-rate and latency held exactly. The cohort
  confirms metadata-size is fully decoupled from the closed hit-rate and
  scan-latency frontiers.

## Open questions

- Now that scored `Entry` is driven to a single aligned word by relocating
  fields off it, does the scored metric still reflect real memory — or has
  the cohort begun optimizing the *measurement* (padding, field location)
  rather than actual footprint?
- The container-memory frontier (separate ghost set vs. tombstone-laden
  single map) was named but not contested this cycle. Is it still the next
  binding constraint, and does freq-relocation shift the real per-slot
  layout in either family's favor?
- With hit-rate, both scans, and now scored `Entry` size all at floors, is
  any scored axis left with genuine headroom?

## Continuation

Field deletion *and* relocation have now exhausted the scored `Entry` type
— it is down to a single word and the "floor" was shown to be padding, so
stop chasing further per-entry shrinkage; the remaining slack is
measurement artifact, not memory. Pivot the next cycle to the unscored
container-memory frontier that has been flagged for three cycles running
but never directly tested: reconcile the separate-`ghost_set` vs.
folded-tombstone families on real footprint, since they make opposite
tradeoffs for identical scored results. Treat hit-rate, scan-latency, and
scored metadata-size all as closed (bit-identical / floored). If no scored
axis has real headroom left, the honest next question is whether the
scoring captures the real cost the container designs differ on.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
