# Cycle 0004 Insight

## Cohort outcome

This cycle tested whether sketch decay—identified as a breakthrough lever in cycle 0003—could be optimized through decay-frequency tuning and synergy with high-frequency geometry. Three decay variants were deployed: aggressive 250k-operation intervals on the stable depth-3 baseline (v001), conservative 1M-operation intervals on the same baseline (v002), and 500k decay paired with the wider width-8 geometry from cycle 0001 (v003). **All three regressed or showed mixed results.** The aggressive interval (v001) traded the depth-3 baseline's latency advantage for marginal hit-rate gains; the conservative interval (v002) added pure overhead with no hit-rate benefit; and decay paired with wide geometry (v003) created catastrophic tail-latency spikes without hit-rate recovery. The cohort reveals that **sketch decay's benefits are fragile and tightly coupled to specific geometry choices**, not a universally applicable tuning lever.

## Patterns

- **Decay overhead is proportional to sketch size and operation frequency**: v001 added modest latency (+3–7 ns per operation) on the smaller width-4 sketch; v003's decay on width-8 geometry created massive tail-latency degradation (p99 get: 154→243 ns). The O(depth×width) bitwise reset operation is expensive at scale, and checking on every operation exacerbates the cost. This suggests decay is fundamentally misaligned with high-frequency sketches.

- **Decay interval thresholds operate in a tight range with no smooth gradient**: v002's conservative 1M interval rarely triggers on this workload, paying conditional overhead for zero hit-rate benefit—a pure fixed cost. v001's aggressive 250k interval triggers frequently enough to degrade latency visibly. Neither bracket the original cycle-0003 sweet spot of 500k, suggesting that point was either optimal by accident or was the result of orthogonal factors (e.g., workload phase transitions) not present on all traces.

- **Decay does not fix collision or admission problems in wide sketches**: v003 expected that fresher frequency signals would help the width-8 geometry avoid noisy estimates during phase transitions. Instead, the decay mechanism's overhead dominated, and hit-rate degraded (0.7409→0.7405), indicating stale counts were not the limiting factor—collision density and width-based accuracy were. Decay appears orthogonal to, or even antagonistic with, geometry-based tuning.

- **Cycle 0003's decay victory may have been workload-specific or a lucky local optimum**: v003 improved both hit-rate and latency on a narrow baseline (depth-3, width-4, 500k interval). Cycle 0004 shows that result does not generalize: shorter intervals add overhead without benefits, longer intervals are ineffective, and wider geometries amplify the cost. The prior cycle's "breakthrough" appears to have been fragile, confined to a specific parameter space rather than a robust, composable lever.

## Open questions

- Was cycle 0003's v003 (500k decay on depth-3 baseline) a genuine improvement or a statistical artifact? All attempts to re-tune or compose it failed. Should the baseline be re-validated to confirm the decay benefit was real?

- Is there a decay interval sweet spot between 250k and 1M that was never tested? v001 and v002 bracket the space but don't hit the 500k midpoint on the depth-3 baseline. If 500k is truly optimal, what property of the workload (operation volume, phase transition frequency) makes that threshold special?

- Why does decay's overhead scale so poorly with sketch width? v003's tail-latency spike suggests the bitwise right-shift on 32 counters (8×4 = width×depth) is expensive under contention. Could decay be implemented more efficiently (e.g., lazy resets, probabilistic sampling) to reduce per-operation cost?

## Continuation

**Decay as a tuning lever has reached diminishing returns and is brittle across parameter variations.** The cycle 0003 victory was likely a narrow optimum, not a robust pattern. **Stop pursuing decay frequency micro-tuning.** Instead, isolate decay's true benefit: re-validate cycle 0003's v003 on the depth-3 width-4 baseline with 500k interval to establish whether the gain was reproducible and how large it actually was. If genuine, **accept depth-3 width-4 with optional 500k decay as the stable geometry baseline** and shift focus to orthogonal levers: eviction policy variations, hash function choices, adaptive admission thresholds, or workload-dependent parameter schedules. If decay was a false positive, **discard it entirely and focus on unexplored dimensions** (e.g., victim selection, frequency aging strategies not based on periodic resets, or cache-oblivious tuning). Geometry optimization has plateaued; the next frontier is likely outside the sketch parameters.
