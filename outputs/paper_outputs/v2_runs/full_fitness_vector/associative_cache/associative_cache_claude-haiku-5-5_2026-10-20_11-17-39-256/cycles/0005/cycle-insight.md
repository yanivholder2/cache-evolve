# Cycle 0005 Insight

## Cohort outcome

The cohort made three small, mostly put-side changes: an early exit in the frequency victim scan (v001), a word-level bucket scan that replaces the per-slot clock walk on top of the per-bucket mask (v002), and a bulk-add rewrite of the 2-bit re-reference aging (v003). All three kept hit-rate and metadata unchanged; the only movement was in timing, and it was mixed.

## Patterns

- Hit-rate held across the cohort, so these changes are behavior-preserving. Each one is an implementation-level rewrite of an existing mechanism, not a new retention policy.
- The per-bucket mask follow-up (v002) recovered put median time that the bit-by-bit clock walk had lost in Cycle 0004, which supports the idea that the scan form, not the mask itself, was the source of that regression. Put tail did not improve, though.
- The frequency early exit (v001) gave only a small put gain, which suggests a zero-counter slot is rarely present when a put scans the bucket. The scan usually runs to completion, so this shortcut is not where the put-side cost lives.
- The aging rewrite (v003) made the put path slower. The scan-then-bulk-add form appears to add full-bucket passes on the common full-bucket put, so the aging rounds were not the dominant put-tail cost. Cycle 0004 pointed at them; this cycle's result argues against that diagnosis.
- Get-side timing moved in v002 with no change to the get path, and the same kind of unexplained get movement appeared in earlier cycles. Treat get-side gains as noise until re-measured.
- Mechanisms here are inferred from diffs and variant insights, not confirmed from overlay code, because overlays were not readable this cycle.

## Open questions

- Where does the put tail actually come from on full-bucket puts, if neither the aging rounds nor the zero-counter scan dominates it?
- How often is a zero-counter slot present at put time, measured directly? This decides whether the early exit is worth keeping at all.
- Does the word-level scan's put-median gain hold across runs, and can its p99 rise be traced to the same code path?
- Is the get-side movement in v002 real or layout-related?

## Continuation

Next cycle should keep the per-bucket mask and the word-level scan (v002) as the put-side base, and profile the victim search on full-bucket puts before changing it again. Drop the aging rewrite (v003) as a put-tail fix; the scan-then-bulk-add form is a net loss. Drop the zero-counter early exit (v001) as an axis unless a measurement shows it fires often enough to matter.
</content>
