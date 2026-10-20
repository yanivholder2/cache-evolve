# Cross-Cycle Insights

## Recurring wins

- **Restructuring resident ordering pays off.** SLRU segmentation of the resident set (protected vs. probationary) is the only intervention so far that improved hit rate. Changes that operate after admission — on how residents are ranked and protected — appear more effective than changes at the admission boundary itself.

## Recurring failures

- **Admission-policy interventions have backfired consistently.** Both a newcomer admission window and a shorter sketch-aging cadence degraded performance in cycle 0001. The admission window carries a structural capacity penalty (shrinks the main LRU) that this workload punishes before any grace-period benefit can accrue. Sketch aging at too-short an interval collapses frequency distinctions faster than the trace replenishes them, making the admission gate noisier rather than sharper.

## Open questions

- **SLRU split ratio sensitivity:** the 80/20 (protected/probationary) split was arbitrary; 90/10 and 70/30 variants are unprobed and the narrow win leaves room for meaningful ratio-driven gains.
- **Sketch aging sweet spot:** there likely exists a substantially longer reset cadence — or a per-key counter floor — where sketch aging becomes helpful rather than harmful; the threshold has not been located.
- **Compounding SLRU improvements:** adaptive or workload-driven segment resizing may amplify the modest SLRU gain; untested.

## Per-policy notes

- **TinyLFU + SLRU (v003, current baseline):** positive result; SLRU's protection layer is partially redundant with the existing frequency gate but still adds net value. The gain is real but narrow, suggesting the baseline admission gate already screens most one-hit-wonders.
- **TinyLFU + newcomer admission window (v002):** net negative; fixed window capacity cost is not recouped on this trace's access pattern — items do not accumulate sufficient frequency within the window to justify the structural shrinkage of the main cache.
- **TinyLFU + accelerated sketch aging (v001):** net negative; reset interval was calibrated too aggressively for the workload's hot/cold cycle length, degrading gate signal quality.
