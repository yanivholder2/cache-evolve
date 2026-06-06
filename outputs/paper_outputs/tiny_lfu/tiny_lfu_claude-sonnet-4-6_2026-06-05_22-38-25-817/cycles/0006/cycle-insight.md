# Cycle 0006 Insight

## Cohort outcome

The cohort tested three distinct axes: extending the frequency-victim scan to the protected segment, sharpening CountMinSketch estimates via conservative update, and raising the admission floor. Only the sketch-quality change (v002) produced a gain and was selected winner — the first improvement in several cycles to originate from the frequency-estimation layer rather than from structural cache policy.

## Patterns

- **Sketch noise was still a meaningful drag after five cycles of policy tuning.** Conservative CMS update — incrementing only the rows already at the minimum — reduced collision-inflated frequency estimates and let the existing k=4 victim scan and PROMO_THRESHOLD gate act on sharper signal. The win came with no structural policy change at all, confirming that data-quality improvement was a live frontier the earlier policy work could not surface.
- **The protected segment's LRU tail is frequency-homogeneous.** Mirroring the probationary k=4 victim scan into protected produced zero movement: the PROMO_THRESHOLD gate pre-filters entrants, leaving the LRU end of protected with a narrow frequency spread; the scan rarely finds a colder candidate than the default LRU head. The asymmetry between the two segments is now well-understood and closed.
- **The TinyLFU admission floor is already captured by the existing strict inequality.** Raising the minimum admission frequency to 2 was redundant on a warm Zipfian trace — the edge case it targets (freq-1 item evicting a freq-0 resident) is a cold-start artifact averaged away over a long run. This axis is closed.

## Open questions

- Do other sketch parameters — width, depth, or reset interval — offer further signal-quality gains now that conservative update is the baseline?
- Does conservative update alter how quickly frequencies decay under the aging/reset mechanism, and could that interaction suppress or amplify the gain at different cache-warmth levels?
- Is the conservative-update gain robust across non-Zipfian workloads, or does it depend on the heavy-tail concentration of this trace?

## Continuation

Take v002's conservative CMS update as the new baseline. The highest-value next probe is sketch configuration: width/depth sizing and reset frequency have not been tuned and may limit how much the sharper estimates translate into correct decisions. Protected-segment victim scanning and admission-floor variants are definitively closed. If sketch configuration also disappoints, the remaining structural frontier is frequency decay/aging policy — how quickly the sketch forgets past frequency — rather than any further segment-management change.
