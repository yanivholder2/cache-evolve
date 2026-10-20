# Cross-cycle insights

## Recurring wins

- Retention signals that separate re-referenced keys from one-shot inserts gain hit-rate on workload-a. This has held from frequency-based victim selection (C2) through second-chance (C3), the 2-bit re-reference value (C4), and the doorkeeper admission gate (C6).
- Behavior-preserving rewrites of an existing mechanism can recover timing without changing hit-rate. The word-level scan over the per-bucket mask (C5) recovered put median that the bit-by-bit clock walk had lost (C4).
- Keeping reference state per bucket rather than per item saves metadata at a time cost, and that time cost can be partly recovered by the scan form (C4, C5).

## Recurring failures

- Moving items within a bucket (midpoint promotion, midpoint reinsertion) lost hit-rate and added shifting work on the hot path (C1).
- Lengthening the bucket-local ghost ring did not lift hit-rate (C2). Fingerprint prefilters added a second scan and write work with no hit-rate effect (C2).
- Put-side shortcuts to the victim scan keep failing: cached minimum (C3), per-eviction halving merged into the scan (C3), rotating window (C4), early exit on zero counters (C5), narrowing to unreferenced slots (C6). Shortening the scan is not where put cost lives.
- Extra decay on top of the periodic halving erased frequency protection too fast (C3).
- Rewriting the aging step as scan-then-bulk-add made puts slower (C5), so the aging rounds were not the dominant put-tail cost (C4's diagnosis did not hold).
- Single-run get and put tail movements, such as the saturated-counter store skip (C6) and earlier unexplained get shifts (C1, C2, C5), have not been reproducible enough to count as gains.

## Open questions

- Where the put tail actually comes from on full-bucket puts. Aging rounds (C4), the zero-counter scan (C5), and the victim scan length (C6) have each been ruled out as the dominant source.
- Whether the retention gains (frequency, second-chance, 2-bit, doorkeeper) hold on traces other than workload-a and at other cache sizes. Every winning cycle has measured only workload-a.
- Whether the metadata-size probe counts per-bucket and per-slot state. It reported no change in some cycles where state clearly grew (C1, C2), so storage costs are not yet reliably measured.
- Whether the doorkeeper's storage and periodic whole-structure clear can be cut so its put-tail cost goes away without losing the admission effect.
- Whether the mechanisms credited for winners (C3 through C6) are the actual sources of their gains. Overlay code was not readable in several cycles, so attributions rest on hypotheses and diffs.

## Per-policy notes

- **Bucket-local ghost / reinsertion:** no hit-rate lift from midpoint placement or a longer ghost ring. Treat the reinsertion idea as weak on this trace. Do not tune ring length further.
- **Frequency-based victim selection (periodic halving):** the first hit-rate gain on workload-a (C2). Keep the periodic halving schedule. Extra decay, cached minimum, and per-eviction halving have not helped. The victim search still needs cheaper bookkeeping that keeps the full-bucket view.
- **Second-chance / reference bit:** the first result to improve hit-rate and put cost together (C3). Moving the bit into a per-bucket mask keeps hit-rate and saves metadata at a time cost, and the word-level scan partly recovers it (C4, C5). Get-side writes on every hit are still a candidate to cut.
- **Multi-level re-reference (2-bit per slot):** gains hit-rate with a put-tail cost (C4). The bulk-add aging rewrite made it worse (C5). Open whether the gain survives other traces and whether the tail can be cut without losing hit-rate.
- **Doorkeeper admission gate:** gains hit-rate by keeping one-shot keys out of the victim scan (C6). Its storage and periodic clear show up as put-tail and metadata costs. Next step is to shrink storage and spread the clear, holding hit-rate fixed.
