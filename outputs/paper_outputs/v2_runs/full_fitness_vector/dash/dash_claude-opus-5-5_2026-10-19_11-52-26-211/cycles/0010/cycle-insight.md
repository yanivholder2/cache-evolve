# Cycle 0010 Insight

## Cohort outcome

All three variants branched from cycle-9 v002, the tagged-header latency parent, and all three ran follow-ups that cycle 9 had asked for. v001 shrank the header to fit within one cache line. v002 stacked target-only rescue on top of tags. v003 retried the wider, fewer stash buckets idea. All three core claims were refuted. v001 left behaviour unchanged and get flat. v002 gained a small amount of hit-rate but repeated cycle 8's get regression and made put worse. v003 lost hit-rate and was slower on put and on the get tail. No variant beats the parent overall.

## Patterns

- The tag lever is used up. Shrinking the header did nothing because the hot probe fields already sat in one cache line. Cycle 9's "lower bound" caveat was wrong: tags gave their full gain in cycle 9. The only layout idea left untested is real 64 B alignment of the bucket array. A smaller struct is not enough.
- Rescue's cost is cascades, not scanning. With tags, a fuller target bucket barely costs anything to probe, yet the get-p50 regression stayed about the same size as in cycle 8. Fuller targets send more stash-hit promotions into full buckets, and that triggers demote → stash → rescue chains on both get and put. The pre-registered refutation line ("cost is cascades") was hit.
- Tags lower the cost of each slot but not the number of slots scanned. Making buckets longer (v003's stash, v002's fuller targets) still costs on put shifts, existence checks, victim search and multi-bucket miss tails.
- Hit-rate rises only when keys get new places to land. Pooling existing stash capacity loses hit-rate. Cycle 7's independent hash added little. Together these show stash overflow is not clustered, so stash geometry is now closed.
- The Front/Back insertion asymmetry may be hurting hit-rate. Fresh overflow keys go in at the LRU end and are the next victims, while cold demotees go in at the MRU end and survive. v003 suggests this gets worse as buckets widen.
- The put-p99 baseline is still noisy: v001's put-tail regression has no mechanism behind it.

## Open questions

- If rescue only runs when the target stays below a fill threshold, so promotion targets always keep a hole, does it keep its hit-rate without cascade cost?
- Would placing fresh stash puts at the MRU end, or demotees at the LRU end, raise hit-rate at no latency cost?
- Does `#[repr(align(64))]` on the bucket array move get at all?

## Continuation

Keep cycle-9 v002 as the latency parent. None of this cycle's variants replaces it. The next hit-rate probe should be stash insertion order (fresh keys vs demotees at the LRU/MRU ends): it changes no state shape and adds no scans. Retry rescue only with a fill-threshold or promotion-hole guard that limits cascades. Drop header-size tweaks, stash-geometry pooling and plain rescue ports. Test 64 B bucket alignment only as a cheap side experiment.
