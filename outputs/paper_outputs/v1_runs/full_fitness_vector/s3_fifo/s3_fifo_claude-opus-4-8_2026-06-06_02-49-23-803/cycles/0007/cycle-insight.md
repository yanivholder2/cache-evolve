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
