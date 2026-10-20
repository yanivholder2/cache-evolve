## Recurring wins

- Retention signals that separate re-referenced items from one-shot inserts have produced Winners in three successive cycles: frequency counters (C2), second-chance reference bits (C3), and the multi-level re-reference value (C4). This is the main productive thread on workload-a.
- Eviction decisions that see the full bucket carry the hit-rate gain. Narrowing the victim scan (C4) lost hit-rate, and the frequency protection weakened with it.
- Reference state can be relocated or re-encoded without losing the retention effect: a per-bucket mask kept hit-rate level with its parent (C4), so the signal does not depend on sitting inside each item.

## Recurring failures

- Positional changes inside a bucket have not paid off. Midpoint promotion (C1) and midpoint reinsertion lost hit-rate and added shifting work on the hot path.
- Bucket-local ghost memory has not produced a win. Lengthening the ghost ring did not lift hit-rate (C2), so the reinsertion idea itself looks weak here.
- Fingerprint prefilters on 32-slot buckets added a linear scan on get and write work on put with no hit-rate change (C2).
- Put-side victim-search shortcuts keep failing. A cached minimum was invalidated by get-hit counter updates (C3). A merged scan with per-eviction halving did not help (C3). A miss-driven extra decay erased frequency protection too quickly (C3). A rotating-window scan cut put latency but lost hit-rate (C4).
- The put-side cost of the frequency full-bucket scan (C2) is still unsolved. No variant has removed it without giving up retention.

## Open questions

- The second-chance mechanism (C3) and the re-reference gain (C4) are inferred from diffs and fitness vectors, not confirmed from overlay code. The attribution needs checking before later cycles build on it.
- Do the retention gains hold on traces other than workload-a and at other cache sizes? Every Winner so far is tied to one trace.
- Which reference-state width is worth its cost: one bit per item (C3), a per-bucket mask (C4 v001), or two bits per slot (C4 v003)?
- Can the put-tail cost of the aging rounds (C4) be cut by capping rounds or tracking aging state, without losing hit-rate?
- Does the bucket-width hit-rate gain from C1 survive re-measurement, and is it specific to workload-a's conflict pattern?
- The metadata-size probe has not registered new per-bucket or per-slot state in several cycles (C1, C2). Its readings should not be trusted for storage cost until it counts that state.
- Put and get latency shifts have often been single-run and not explained by the diffs (C1, C2). Re-measure before judging any variant on timing.

## Per-policy notes

- Frequency-based victim selection: gains hit-rate by protecting re-referenced keys. Its full-bucket scan on put is the main cost. Keep the periodic halving schedule as is. Per-eviction halving and extra miss-driven decay have not helped.
- Second-chance reference bit: the most repeated Winner line. It improved hit-rate and put cost together (C3), with a small get-side write on each hit and larger per-item metadata. Next steps are to avoid the get-side write, for example by setting state on only some hits, and to cut state toward one bit.
- Multi-level re-reference value (2 bits per slot): gains hit-rate. Its put tail rises through bounded aging rounds. Attack the aging cost first, keeping hit-rate fixed.
- Per-bucket reference mask: matched parent hit-rate and saved metadata, but slowed both get and put. Keep only as a metadata option, and only if the bit-scan cost can be cut.
- Rotating-window victim search: lower put latency, lower hit-rate. Drop as a put-side fix. Cheaper bookkeeping for the true global minimum is the better direction.
- Bucket-local ghost ring and midpoint reinsertion/promotion: no Winner. Ghost length is closed as an axis. Midpoint placement should be dropped as currently specified.
- Bucket geometry (wider buckets): the only C1 change that raised hit-rate, but by a small margin with a get-path cost from the longer linear scan. Re-measure before building on it.
- Fingerprint prefilter: no hit-rate change and added scan and write cost. Dropped.
