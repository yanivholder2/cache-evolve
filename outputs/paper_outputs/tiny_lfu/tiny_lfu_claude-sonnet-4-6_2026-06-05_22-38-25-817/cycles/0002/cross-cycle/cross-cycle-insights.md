## Recurring wins

- **Resident-ordering changes beat admission-policy changes.** The only two winners across both cycles (SLRU segmentation, then mild SLRU ratio tightening) restructured how admitted items are ordered and protected; every variant that intervened at admission time or at the frequency gate itself failed to improve hit rate.
- **Accumulated frequency counts are signal, not noise.** The workload's access distribution is stable across the full replay window, so approaches that preserve the frequency sketch unmodified consistently outperform those that periodically reset or degrade it.

## Recurring failures

- **Sketch aging** — harmful at every cadence tested (short in cycle 0001, 10× longer in cycle 0002). The trace's frequency distribution is stationary; halving counters destroys valid signal faster than stale counts accumulate, making the admission gate noisier in both cases.
- **Capacity-splitting admission windows** — permanently reserving a probationary or newcomer segment shrinks the main resident set, and the capacity loss is felt immediately before any grace-period benefit can materialize. This trace does not produce items that accumulate frequency fast enough inside a window to recoup the structural cost.
- **Parametric SLRU ratio tuning** — gains from moving the protected/probationary split diminish rapidly; the workload's hot set was well-contained at 80 % protected, so 90/10 added negligible value. Further ratio adjustment is unlikely to yield meaningful headroom.

## Open questions

- What structural additions — ghost/victim caches, recently-evicted key feedback for re-admission biasing — could shift hit rate on a stable Zipfian workload where SLRU-on-TinyLFU is near a local optimum?
- Can in-segment ordering within the protected SLRU tier be improved (e.g., frequency-ordered rather than LRU-ordered), and does that unlock headroom the ratio tuning didn't?
- Is there a frequency-promotion threshold high enough (e.g., 5–10) to meaningfully reduce protected-segment churn, or does this trace simply not produce enough borderline-frequency contention for a threshold gate to matter?
- At what sketch aging cadence — or with what per-key counter floor — does counter decay shift from harmful to neutral or helpful? The current evidence spans only two cadences; both were harmful, but the search space is not exhausted.

## Per-policy notes

- **SLRU (TinyLFU baseline):** The structural choice that consistently provides the framework for winning variants. 90/10 split is the current marginal best but gains over 80/20 are negligible. Further ratio tuning is deprioritized; next improvements should be structural additions on top of this configuration.
- **Sketch aging variants:** Uniformly harmful across cycles 0001 and 0002 at every tested cadence. Deprioritized until a mechanism is found that preserves hot-key signal while selectively aging cold keys.
- **Admission window / newcomer buffer:** Carried a structural capacity penalty immediately punished by this trace. No variant using this lever has improved hit rate; deprioritized.
- **Frequency-gated promotion threshold:** Tested only at a low threshold value (2); landed in a behavioral dead zone — borderline-frequency items were too rare or promoted themselves naturally without the gate. Worth one revisit at a substantially higher value, but not as a standalone variant.
